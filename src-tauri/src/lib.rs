mod commands;
mod detect;
mod jobs;
mod presets;
mod probe;
mod progress;
mod runner;
mod settings;
mod state;

use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let stored = settings::load(app.handle());
            let mut state = state::AppState::new(stored);
            state.settings_path = settings::settings_path(app.handle());
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_ffmpeg,
            commands::redetect_ffmpeg,
            commands::enqueue_files,
            commands::cancel_job,
            commands::retry_job,
            commands::get_settings,
            commands::update_settings,
            commands::reveal_path,
            commands::video_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
