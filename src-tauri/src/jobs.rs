use serde::{Deserialize, Serialize};

/// Compression preset chosen in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    Telegram,
    Web,
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobKind {
    Video,
    Image,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Probing,
    Processing,
    Done,
    Error,
    Canceled,
}

/// A queue item as seen by the frontend. Mirrors `Job` in `src/types.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub input_path: String,
    pub output_path: Option<String>,
    pub file_name: String,
    pub kind: JobKind,
    pub preset: Preset,
    pub input_size: u64,
    pub status: JobStatus,
    pub progress: Option<f32>,
    pub eta_secs: Option<u64>,
    pub fps: Option<f32>,
    pub kbps: Option<u64>,
    pub output_size: Option<u64>,
    pub error: Option<String>,
    /// Non-fatal note for the UI (e.g. "fell back to CPU", "output removed").
    /// An i18n `warning.*` key.
    pub warning: Option<String>,
}

/// Live update emitted on the `job-update` channel. `None` fields are skipped
/// so the frontend can distinguish "no change" (absent) from "clear" (`null`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobUpdate {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<JobStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<Option<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eta_secs: Option<Option<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<Option<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kbps: Option<Option<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_size: Option<Option<u64>>,
    /// `Some(None)` serializes to `null` → the frontend clears a previous warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<Option<String>>,
    /// `Some(None)` serializes to `null` → the frontend clears a previous error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Option<String>>,
}

impl JobUpdate {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: None,
            progress: None,
            eta_secs: None,
            fps: None,
            kbps: None,
            output_path: None,
            output_size: None,
            warning: None,
            error: None,
        }
    }
}
