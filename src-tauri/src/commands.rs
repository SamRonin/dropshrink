use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::detect::{self, FfmpegStatus};
use crate::jobs::{Job, JobKind, JobStatus, JobUpdate, Preset};
use crate::probe;
use crate::runner;
use crate::settings::{self, Settings};
use crate::state::{AppState, JobControl, JobSpec};

const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "mov", "webm", "m4v", "avi", "wmv", "flv", "ts", "mpg", "mpeg", "3gp",
];
const IMAGE_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "tif", "tiff", "gif", "avif"];

fn extension_kind(path: &str) -> Option<JobKind> {
    let ext = Path::new(path)
        .extension()?
        .to_string_lossy()
        .to_lowercase();
    if VIDEO_EXTS.contains(&ext.as_str()) {
        Some(JobKind::Video)
    } else if IMAGE_EXTS.contains(&ext.as_str()) {
        Some(JobKind::Image)
    } else {
        None
    }
}

fn job_from_spec(spec: &JobSpec) -> Job {
    Job {
        id: spec.id.clone(),
        input_path: spec.input_path.clone(),
        output_path: None,
        file_name: spec.file_name.clone(),
        kind: spec.kind,
        preset: spec.preset,
        input_size: spec.input_size,
        status: JobStatus::Queued,
        progress: None,
        eta_secs: None,
        fps: None,
        kbps: None,
        output_size: None,
        error: None,
        warning: None,
    }
}

#[tauri::command]
pub async fn detect_ffmpeg(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<FfmpegStatus, String> {
    {
        let cached = state.ffmpeg.read().unwrap();
        if let Some(status) = cached.as_ref() {
            return Ok(status.clone());
        }
    }
    let custom = state.settings.lock().unwrap().custom_ffmpeg_dir.clone();
    let status = detect::detect(&app, custom.as_deref()).await;
    *state.ffmpeg.write().unwrap() = Some(status.clone());
    Ok(status)
}

/// Re-run detection from scratch — used after the user changes the custom
/// FFmpeg folder so the cached status is invalidated.
#[tauri::command]
pub async fn redetect_ffmpeg(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<FfmpegStatus, String> {
    let custom = state.settings.lock().unwrap().custom_ffmpeg_dir.clone();
    let status = detect::detect(&app, custom.as_deref()).await;
    *state.ffmpeg.write().unwrap() = Some(status.clone());
    Ok(status)
}

#[tauri::command]
pub async fn enqueue_files(
    app: AppHandle,
    state: State<'_, AppState>,
    paths: Vec<String>,
    preset: Preset,
) -> Result<Vec<Job>, String> {
    // Make sure FFmpeg is detected at least once before any job runs.
    if state.ffmpeg.read().unwrap().is_none() {
        let custom = state.settings.lock().unwrap().custom_ffmpeg_dir.clone();
        let status = detect::detect(&app, custom.as_deref()).await;
        *state.ffmpeg.write().unwrap() = Some(status);
    }

    let mut created: Vec<Job> = Vec::new();
    for path in paths {
        let Some(kind) = extension_kind(&path) else {
            return Err("error.unsupported_file".into());
        };
        let file = Path::new(&path);
        if !file.is_file() {
            return Err(format!("file_not_found: {path}"));
        }
        let input_size = std::fs::metadata(file).map(|m| m.len()).unwrap_or(0);
        let file_name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());
        let id = Uuid::new_v4().to_string();
        let spec = JobSpec {
            id: id.clone(),
            input_path: path,
            file_name,
            kind,
            preset,
            input_size,
        };
        let job = job_from_spec(&spec);
        state.jobs.lock().unwrap().insert(
            id,
            JobControl {
                spec,
                cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
                child: None,
            },
        );
        created.push(job);
    }

    for job in &created {
        runner::spawn(app.clone(), job.id.clone());
    }
    Ok(created)
}

#[tauri::command]
pub async fn cancel_job(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let control = {
        let jobs = state.jobs.lock().unwrap();
        jobs.get(&id)
            .map(|c| (Arc::clone(&c.cancel), c.child.clone()))
    };
    if let Some((cancel, child)) = control {
        cancel.store(true, Ordering::SeqCst);
        match child {
            Some(child) => {
                // Running: kill the ffmpeg process; the runner reports "canceled".
                let mut child_lock = child.lock().await;
                let _ = child_lock.start_kill();
            }
            None => {
                // Still queued: reflect the new state immediately for a snappy UI.
                runner::emit(
                    &app,
                    JobUpdate {
                        status: Some(JobStatus::Canceled),
                        error: Some(Some("error.canceled".into())),
                        ..JobUpdate::new(&id)
                    },
                );
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn retry_job(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<Job>, String> {
    let job = {
        let mut jobs = state.jobs.lock().unwrap();
        match jobs.get_mut(&id) {
            Some(control) => {
                control.cancel.store(false, Ordering::SeqCst);
                control.child = None;
                Some(job_from_spec(&control.spec))
            }
            None => None,
        }
    };

    if let Some(job) = &job {
        runner::emit(
            &app,
            JobUpdate {
                status: Some(JobStatus::Queued),
                progress: Some(None),
                eta_secs: Some(None),
                fps: Some(None),
                kbps: Some(None),
                output_size: Some(None),
                warning: Some(None),
                error: Some(None),
                ..JobUpdate::new(&id)
            },
        );
        runner::spawn(app.clone(), job.id.clone());
    }
    Ok(job)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<(), String> {
    {
        let mut current = state.settings.lock().unwrap();
        *current = settings.clone();
        let mut slot = state.slot.lock().unwrap();
        slot.max = settings.max_concurrent.clamp(1, 16);
    }
    // Wake waiting jobs so a raised limit applies immediately.
    state.slot_notify.notify_one();

    if let Some(path) = &state.settings_path {
        settings::save(path, &settings);
    }
    Ok(())
}

#[tauri::command]
pub async fn reveal_path(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let target = Path::new(&path);
    if !target.exists() {
        return Err(format!("path_not_found: {path}"));
    }
    let result = if target.is_file() {
        app.opener().reveal_item_in_dir(target)
    } else {
        app.opener()
            .open_path(target.to_string_lossy().to_string(), None::<String>)
    };
    result.map_err(|e| e.to_string())
}

/// Frames + metadata for the preview & diff modal (videos only).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPreview {
    pub input_first: String,
    pub input_middle: String,
    pub output_first: Option<String>,
    pub output_middle: Option<String>,
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

fn fnv1a_hash(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

async fn extract_frame(
    ffmpeg: &str,
    input: &str,
    at_secs: f64,
    out: &Path,
) -> Result<(), String> {
    let output = tokio::process::Command::new(ffmpeg)
        .args(["-hide_banner", "-v", "error", "-nostdin", "-y"])
        .arg("-ss")
        .arg(format!("{at_secs:.3}"))
        .args(["-i", input, "-frames:v", "1", "-q:v", "3"])
        .arg(out)
        .output()
        .await
        .map_err(|e| format!("frame_spawn_failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let last = stderr.lines().last().unwrap_or("unknown error").trim();
        return Err(format!("frame_failed: {last}"));
    }
    Ok(())
}

/// Extract the first and middle frames of a video (and of its compressed
/// output when available) so the UI can diff them side by side.
#[tauri::command]
pub async fn video_preview(
    app: AppHandle,
    state: State<'_, AppState>,
    input: String,
    output: Option<String>,
) -> Result<VideoPreview, String> {
    let status = state.ffmpeg.read().unwrap().clone();
    let Some(status) = status.filter(|s| s.ffmpeg_path.is_some() && s.ffprobe_path.is_some())
    else {
        return Err("error.ffmpeg_missing".into());
    };
    let ffmpeg = status.ffmpeg_path.clone().unwrap_or_default();
    let ffprobe = status.ffprobe_path.clone().unwrap_or_default();

    let media = probe::probe(&ffprobe, Path::new(&input))
        .await
        .map_err(|e| format!("probe_failed: {e}"))?;
    let duration = media.duration_secs.unwrap_or(0.0).max(0.0);

    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("cache_dir_failed: {e}"))?
        .join("previews");
    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("cache_dir_failed: {e}"))?;

    // Deterministic names: re-opening the same file overwrites its previews.
    let key = fnv1a_hash(&input).to_string();
    let input_first = cache_dir.join(format!("{key}-in-first.jpg"));
    let input_middle = cache_dir.join(format!("{key}-in-mid.jpg"));

    extract_frame(&ffmpeg, &input, 0.0, &input_first).await?;
    let middle_at = if duration > 0.1 { duration / 2.0 } else { 0.0 };
    extract_frame(&ffmpeg, &input, middle_at, &input_middle).await?;

    // Same frames from the compressed output, when it exists on disk.
    let output_path = output.map(PathBuf::from).filter(|p| p.is_file());
    let (output_first, output_middle) = match &output_path {
        Some(out) => {
            let out_first = cache_dir.join(format!("{key}-out-first.jpg"));
            let out_middle = cache_dir.join(format!("{key}-out-mid.jpg"));
            let out_str = out.to_string_lossy().to_string();
            match extract_frame(&ffmpeg, &out_str, 0.0, &out_first).await {
                Ok(()) => {
                    let mid = extract_frame(&ffmpeg, &out_str, middle_at, &out_middle).await;
                    let first = Some(out_first.to_string_lossy().to_string());
                    let middle = mid.ok().map(|_| out_middle.to_string_lossy().to_string());
                    (first, middle)
                }
                Err(_) => (None, None),
            }
        }
        None => (None, None),
    };

    Ok(VideoPreview {
        input_first: input_first.to_string_lossy().to_string(),
        input_middle: input_middle.to_string_lossy().to_string(),
        output_first,
        output_middle,
        duration_secs: media.duration_secs,
        width: media.width,
        height: media.height,
    })
}
