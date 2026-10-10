use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::detect::{self, FfmpegStatus};
use crate::jobs::{Job, JobKind, JobStatus, JobUpdate, Preset};
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
    let status = detect::detect(&app).await;
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
        let status = detect::detect(&app).await;
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
                        error: Some("error.canceled".into()),
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
                job_from_spec(&control.spec)
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
        app.opener().reveal_item_in_dir(target.to_path_buf())
    } else {
        app.opener()
            .open_path(target.to_string_lossy().to_string(), None::<String>)
    };
    result.map_err(|e| e.to_string())
}
