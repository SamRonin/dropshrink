use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::jobs::{JobKind, JobStatus, JobUpdate};
use crate::presets;
use crate::probe;
use crate::progress::ProgressParser;
use crate::state::{AppState, SlotGuard};

const ERROR_FFMPEG_MISSING: &str = "error.ffmpeg_missing";
const ERROR_UNSUPPORTED: &str = "error.unsupported_file";
const ERROR_CANCELED: &str = "error.canceled";
const ERROR_ENCODE_FAILED: &str = "error.encode_failed";

const WARNING_HW_FALLBACK: &str = "warning.hw_fallback";
const WARNING_OUTPUT_REMOVED: &str = "warning.output_removed";

pub fn emit(app: &AppHandle, update: JobUpdate) {
    let _ = app.emit("job-update", update);
}

/// Schedule a job on the async runtime (the queue picks it up via slots).
pub fn spawn(app: AppHandle, id: String) {
    tauri::async_runtime::spawn(async move {
        run_job(app, id).await;
    });
}

fn unique_output_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let mut candidate = dir.join(format!("ds-{stem}.{ext}"));
    let mut counter = 1u32;
    while candidate.exists() {
        candidate = dir.join(format!("ds-{stem}-{counter}.{ext}"));
        counter += 1;
    }
    candidate
}

/// Result of one ffmpeg encode attempt.
struct EncodeOutcome {
    exit_code: Option<i32>,
    stderr_tail: Vec<String>,
}

impl EncodeOutcome {
    fn success(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Spawn ffmpeg once, wire up progress + stderr readers, wait for it to exit.
async fn run_encode(
    app: &AppHandle,
    state: &AppState,
    id: &str,
    ffmpeg_path: &str,
    args: &[String],
    total_secs: Option<f64>,
) -> EncodeOutcome {
    let mut child = match tokio::process::Command::new(ffmpeg_path)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            return EncodeOutcome {
                exit_code: None,
                stderr_tail: vec![format!("spawn_failed: {err}")],
            };
        }
    };

    // Take the pipes first, then share the child itself for cancellation.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child = Arc::new(tokio::sync::Mutex::new(child));

    // Register the running child so cancel_job can kill it.
    {
        let mut jobs = state.jobs.lock().unwrap();
        if let Some(control) = jobs.get_mut(id) {
            control.child = Some(Arc::clone(&child));
        }
    }

    // --- Progress reader (stdout: `-progress pipe:1` key=value lines) --------
    let progress_handle = tauri::async_runtime::spawn({
        let app = app.clone();
        let id = id.to_string();
        let mut parser = ProgressParser::new(total_secs);
        async move {
            if let Some(stdout) = stdout {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let Some(tick) = parser.feed(&line) else {
                        continue;
                    };
                    emit(
                        &app,
                        JobUpdate {
                            progress: Some(tick.percent),
                            eta_secs: Some(tick.eta_secs),
                            fps: Some(tick.fps),
                            kbps: Some(tick.kbps),
                            ..JobUpdate::new(&id)
                        },
                    );
                    if tick.finished {
                        break;
                    }
                }
            }
        }
    });

    // --- stderr collector (last lines become the error message) --------------
    let stderr_handle = tauri::async_runtime::spawn(async move {
        let mut tail: Vec<String> = Vec::new();
        if let Some(stderr) = stderr {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tail.len() >= 12 {
                    tail.remove(0);
                }
                tail.push(line);
            }
        }
        tail
    });

    let status = {
        let mut child_lock = child.lock().await;
        child_lock.wait().await
    };
    let exit_code = status.ok().and_then(|s| s.code());

    // Release the child handle in the registry.
    {
        let mut jobs = state.jobs.lock().unwrap();
        if let Some(control) = jobs.get_mut(id) {
            control.child = None;
        }
    }

    let _ = progress_handle.await;
    let stderr_tail = stderr_handle.await.unwrap_or_default();

    EncodeOutcome {
        exit_code,
        stderr_tail,
    }
}

async fn run_job(app: AppHandle, id: String) {
    // Clone before borrowing state so the handle can move into sub-tasks.
    let task_app = app.clone();
    let state = app.state::<AppState>();

    // Snapshot the job spec + cancel flag without holding the lock across awaits.
    let (spec, cancel) = {
        let jobs = state.jobs.lock().unwrap();
        match jobs.get(&id) {
            Some(control) => (control.spec.clone(), std::sync::Arc::clone(&control.cancel)),
            None => return,
        }
    };
    cancel.store(false, Ordering::SeqCst);

    // Fast path: canceled while still queued (never even waited for a slot).
    if cancel.load(Ordering::SeqCst) {
        emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Canceled),
                error: Some(Some(ERROR_CANCELED.into())),
                ..JobUpdate::new(&id)
            },
        );
        return;
    }

    state.acquire_slot().await;
    let _slot_guard = SlotGuard(&state);

    // Another cancel may have arrived while waiting for a slot.
    if cancel.load(Ordering::SeqCst) {
        emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Canceled),
                error: Some(Some(ERROR_CANCELED.into())),
                ..JobUpdate::new(&id)
            },
        );
        return;
    }

    // --- FFmpeg availability -------------------------------------------------
    let ffmpeg_status = {
        let guard = state.ffmpeg.read().unwrap();
        guard.clone()
    };
    let Some(ffmpeg_status) = ffmpeg_status.filter(|s| s.ffmpeg_path.is_some()) else {
        emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Error),
                error: Some(Some(ERROR_FFMPEG_MISSING.into())),
                ..JobUpdate::new(&id)
            },
        );
        return;
    };

    let settings = state.settings.lock().unwrap().clone();

    // Reset any stale error/progress/warning from a previous attempt.
    emit(
        &app,
        JobUpdate {
            status: Some(JobStatus::Probing),
            progress: Some(Some(0.0)),
            error: Some(None),
            warning: Some(None),
            output_size: Some(None),
            ..JobUpdate::new(&id)
        },
    );

    // --- Probe (videos only) -------------------------------------------------
    let mut media = None;
    if spec.kind == JobKind::Video {
        let Some(ffprobe) = ffmpeg_status.ffprobe_path.clone() else {
            emit(
                &app,
                JobUpdate {
                    status: Some(JobStatus::Error),
                    error: Some(Some(ERROR_FFMPEG_MISSING.into())),
                    ..JobUpdate::new(&id)
                },
            );
            return;
        };
        match probe::probe(&ffprobe, Path::new(&spec.input_path)).await {
            Ok(info) => media = Some(info),
            Err(err) => {
                let message = if err.contains("unsupported") {
                    ERROR_UNSUPPORTED.to_string()
                } else {
                    format!("{ERROR_ENCODE_FAILED}: {err}")
                };
                emit(
                    &app,
                    JobUpdate {
                        status: Some(JobStatus::Error),
                        error: Some(Some(message)),
                        ..JobUpdate::new(&id)
                    },
                );
                return;
            }
        }
    }

    // --- Compute the output path --------------------------------------------
    let ext = presets::output_ext(
        spec.preset,
        spec.kind,
        settings.web_codec,
        ffmpeg_status.has_vp9,
    );
    let stem = spec
        .file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| spec.file_name.clone());
    let dir: PathBuf = match settings.output_dir.as_ref() {
        Some(custom) => PathBuf::from(custom),
        None => Path::new(&spec.input_path)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    let output = unique_output_path(&dir, &stem, ext);
    let output_str = output.to_string_lossy().to_string();

    emit(
        &app,
        JobUpdate {
            status: Some(JobStatus::Processing),
            output_path: Some(output_str.clone()),
            ..JobUpdate::new(&id)
        },
    );

    // --- Encode loop: HW first (when enabled), one CPU retry on HW failure ---
    let mut settings_eff = settings.clone();
    let mut warning: Option<String> = None;
    let total_secs = media.as_ref().and_then(|m| m.duration_secs);
    let ffmpeg_path = ffmpeg_status.ffmpeg_path.clone().unwrap_or_default();

    let outcome = loop {
        let plan = if spec.kind == JobKind::Video {
            presets::plan_video(
                spec.preset,
                &settings_eff,
                &ffmpeg_status,
                &spec.input_path,
                &output_str,
                media.as_ref().expect("video jobs are always probed"),
            )
        } else {
            presets::EncodePlan {
                args: presets::build_image_args(
                    spec.preset,
                    &spec.input_path,
                    &output_str,
                    settings_eff.strip_metadata,
                ),
                hw_encoder: None,
            }
        };

        let outcome = run_encode(
            &task_app,
            &state,
            &id,
            &ffmpeg_path,
            &plan.args,
            total_secs,
        )
        .await;

        // Killed via cancel_job → report "canceled" regardless of exit code.
        if cancel.load(Ordering::SeqCst) {
            break outcome;
        }
        if outcome.success() {
            break outcome;
        }

        // Hardware encoder failed → fall back to the CPU path exactly once.
        if plan.hw_encoder.is_some() && settings_eff.hw_accel {
            settings_eff.hw_accel = false;
            warning = Some(WARNING_HW_FALLBACK.to_string());
            emit(
                &task_app,
                JobUpdate {
                    status: Some(JobStatus::Processing),
                    progress: Some(Some(0.0)),
                    error: Some(None),
                    warning: Some(warning.clone()),
                    ..JobUpdate::new(&id)
                },
            );
            continue;
        }

        break outcome;
    };

    drop(_slot_guard);

    // --- Final status ---------------------------------------------------------
    if cancel.load(Ordering::SeqCst) {
        // Clean up a partially written output file.
        let _ = std::fs::remove_file(&output);
        emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Canceled),
                error: Some(Some(ERROR_CANCELED.into())),
                warning: Some(warning),
                eta_secs: Some(None),
                fps: Some(None),
                kbps: Some(None),
                ..JobUpdate::new(&id)
            },
        );
        return;
    }

    if !outcome.success() {
        let _ = std::fs::remove_file(&output);
        let tail: Vec<String> = outcome
            .stderr_tail
            .iter()
            .rev()
            .take(4)
            .rev()
            .cloned()
            .collect();
        let message = if tail.is_empty() {
            format!("{ERROR_ENCODE_FAILED} (exit = {:?})", outcome.exit_code)
        } else {
            format!("{ERROR_ENCODE_FAILED}: {}", tail.join("\n"))
        };
        emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Error),
                error: Some(Some(message)),
                warning: Some(warning),
                eta_secs: Some(None),
                fps: Some(None),
                kbps: Some(None),
                ..JobUpdate::new(&id)
            },
        );
        return;
    }

    // --- Size guard: never keep an output that got *bigger* ------------------
    let mut output_size = std::fs::metadata(&output).ok().map(|m| m.len());
    if let Some(size) = output_size {
        if spec.input_size > 0 && size >= spec.input_size {
            // Compression gained nothing → remove the file and tell the user.
            let _ = std::fs::remove_file(&output);
            output_size = None;
            warning = Some(match warning {
                Some(prev) => format!("{prev}\n{WARNING_OUTPUT_REMOVED}"),
                None => WARNING_OUTPUT_REMOVED.to_string(),
            });
        }
    }

    emit(
        &app,
        JobUpdate {
            status: Some(JobStatus::Done),
            progress: Some(Some(100.0)),
            eta_secs: Some(None),
            fps: Some(None),
            kbps: Some(None),
            output_size: Some(output_size),
            warning: Some(warning),
            error: Some(None),
            ..JobUpdate::new(&id)
        },
    );
}
