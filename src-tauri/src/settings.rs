use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tauri::Manager;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebCodec {
    H264,
    Vp9,
}

/// Persisted user preferences. Mirrors `AppSettings` in `src/types.ts`.
/// The container-level `default` keeps old settings files forward-compatible:
/// missing fields are filled in from `Settings::default()` on load.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub max_concurrent: u32,
    pub web_codec: WebCodec,
    pub output_dir: Option<String>,
    pub strip_metadata: bool,
    /// Encode with a hardware encoder when one is detected and working.
    pub hw_accel: bool,
    /// Optional user-provided directory that contains `ffmpeg`/`ffprobe`.
    /// Lowest lookup priority (sidecar → PATH → common → custom).
    pub custom_ffmpeg_dir: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        let cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self {
            max_concurrent: (cores / 2).clamp(1, 8) as u32,
            web_codec: WebCodec::H264,
            output_dir: None,
            strip_metadata: true,
            hw_accel: true,
            custom_ffmpeg_dir: None,
        }
    }
}

pub fn settings_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

pub fn load(app: &tauri::AppHandle) -> Settings {
    let Some(path) = settings_path(app) else {
        return Settings::default();
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(path: &PathBuf, settings: &Settings) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}
