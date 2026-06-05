mod audio;
mod config;
mod cover;
mod db;
mod icloud;
mod llm;
mod profiles;
mod write;

use tauri::Manager;

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
            // 初始化 SQLite 并注入托管状态（节目档案库等持久化数据）
            let database = db::init().map_err(std::io::Error::other)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::read_audio_metadata,
            icloud::check_icloud_status,
            icloud::start_icloud_download,
            llm::parse_filenames,
            llm::generate_filename_rule,
            llm::match_covers,
            cover::scan_cover_candidates,
            cover::read_image_data_url,
            profiles::list_show_profiles,
            profiles::save_show_profile,
            profiles::delete_show_profile,
            write::write_metadata,
            write::reset_files,
            write::list_snapshot_paths,
            config::write_text_file,
            config::read_text_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
