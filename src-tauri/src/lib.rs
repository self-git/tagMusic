mod audio;
mod icloud;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::read_audio_metadata,
            icloud::check_icloud_status,
            icloud::start_icloud_download
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
