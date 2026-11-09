//! Files handed to DropShrink by the operating system ("Open with", a file
//! passed on the command line, or macOS `Opened` events).

use std::path::Path;

use tauri::{AppHandle, Emitter, Manager};

use crate::commands::extension_kind;
use crate::state::AppState;

/// Payload-less signal telling the frontend to call `take_pending_open`.
/// The pending queue is the single source of truth, so a file is never
/// handled twice even if the signal and the initial drain overlap.
pub const OPEN_FILES_EVENT: &str = "open-files-pending";

/// Keeps only real, supported media files.
fn supported(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|p| Path::new(p).is_file() && extension_kind(p).is_some())
        .collect()
}

/// Paths from process arguments. The first argument is the executable itself.
pub fn from_args<I>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    supported(args.into_iter().skip(1).collect())
}

/// Queue files for the UI and notify it.
pub fn queue(app: &AppHandle, paths: Vec<String>) {
    let paths = supported(paths);
    if paths.is_empty() {
        return;
    }
    if let Some(state) = app.try_state::<AppState>() {
        state.pending_open.lock().unwrap().extend(paths);
    }
    let _ = app.emit(OPEN_FILES_EVENT, ());
}

/// Bring the main window to the front (used when a second instance is launched).
pub fn focus_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}
