#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod scheduler;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_briefings,
            commands::get_briefing,
            commands::search_briefings,
            commands::add_feedback,
            commands::get_interests,
            commands::add_interest,
            commands::remove_interest,
            commands::get_preferences,
            commands::update_preferences,
            commands::trigger_research,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();
            // Initialize database
            db::init_database(&app_handle)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
