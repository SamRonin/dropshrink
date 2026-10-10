use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use tokio::process::Child;
use tokio::sync::Notify;

use crate::detect::FfmpegStatus;
use crate::jobs::JobKind;
use crate::jobs::Preset;
use crate::settings::Settings;

/// Immutable description of a piece of work, kept for retries.
#[derive(Debug, Clone)]
pub struct JobSpec {
    pub id: String,
    pub input_path: String,
    pub file_name: String,
    pub kind: JobKind,
    pub preset: Preset,
    pub input_size: u64,
}

/// Mutable per-job control data (cancellation + running child handle).
pub struct JobControl {
    pub spec: JobSpec,
    pub cancel: Arc<AtomicBool>,
    pub child: Option<Arc<tokio::sync::Mutex<Child>>>,
}

pub struct SlotState {
    pub running: u32,
    pub max: u32,
}

/// Global application state managed by Tauri. All mutable fields use interior
/// mutability; guards must never be held across `.await` points.
pub struct AppState {
    pub ffmpeg: RwLock<Option<FfmpegStatus>>,
    pub settings: Mutex<Settings>,
    pub slot: Mutex<SlotState>,
    pub slot_notify: Notify,
    pub jobs: Mutex<HashMap<String, JobControl>>,
    pub settings_path: Option<PathBuf>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        let max = settings.max_concurrent;
        Self {
            ffmpeg: RwLock::new(None),
            settings: Mutex::new(settings),
            slot: Mutex::new(SlotState { running: 0, max }),
            slot_notify: Notify::new(),
            jobs: Mutex::new(HashMap::new()),
            settings_path: None,
        }
    }

    /// Await a free encoding slot. `max` can change live via settings.
    pub async fn acquire_slot(&self) {
        loop {
            let notified = self.slot_notify.notified();
            {
                let mut slot = self.slot.lock().unwrap();
                if slot.running < slot.max {
                    slot.running += 1;
                    return;
                }
            }
            notified.await;
        }
    }

    pub fn release_slot(&self) {
        {
            let mut slot = self.slot.lock().unwrap();
            slot.running = slot.running.saturating_sub(1);
        }
        self.slot_notify.notify_one();
    }
}

/// Releases the encoding slot on drop, so every early-return stays correct.
pub struct SlotGuard<'a>(pub &'a AppState);

impl Drop for SlotGuard<'_> {
    fn drop(&mut self) {
        self.0.release_slot();
    }
}
