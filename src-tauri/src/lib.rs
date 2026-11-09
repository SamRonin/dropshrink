mod commands;
mod detect;
mod jobs;
mod open_files;
mod presets;
mod probe;
mod progress;
mod runner;
mod settings;
mod state;

use tauri::Manager;

pub fn run() {
    let app = tauri::Builder::default()
        // Must be registered first: a second launch (e.g. "Open with" while
        // the app is already running) hands its files to this instance.
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            open_files::queue(app, open_files::from_args(args));
            open_files::focus_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let stored = settings::load(app.handle());
            let mut state = state::AppState::new(stored);
            state.settings_path = settings::settings_path(app.handle());
            app.manage(state);

            // Windows and Linux pass the files as command-line arguments.
            open_files::queue(app.handle(), open_files::from_args(std::env::args()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::detect_ffmpeg,
            commands::redetect_ffmpeg,
            commands::enqueue_files,
            commands::take_pending_open,
            commands::cancel_job,
            commands::retry_job,
            commands::get_settings,
            commands::update_settings,
            commands::reveal_path,
            commands::video_preview,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        // macOS delivers "Open with" / double-click files as Opened events.
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        if let tauri::RunEvent::Opened { urls } = &event {
            let paths = urls
                .iter()
                .filter_map(|url| url.to_file_path().ok())
                .map(|path| path.to_string_lossy().into_owned())
                .collect();
            open_files::queue(app_handle, paths);
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        let _ = (app_handle, event);
    });
}
