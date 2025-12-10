#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod tray;
mod notifications;
mod research;
mod research_state;
mod mcp_client;
mod research_log;

use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

fn main() {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance tries to launch, focus the existing main window
            tracing::info!("Second instance detected, focusing existing window");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            // Briefing commands
            commands::get_briefings,
            commands::get_briefing,
            commands::get_briefing_by_id,
            commands::get_todays_briefings,
            commands::search_briefings,
            // Feedback commands
            commands::add_feedback,
            commands::submit_feedback,
            // Topic commands
            commands::get_topics,
            commands::add_topic,
            commands::update_topic,
            commands::delete_topic,
            commands::reorder_topics,
            // MCP server commands
            commands::get_mcp_servers,
            commands::toggle_mcp_server,
            commands::add_mcp_server,
            commands::update_mcp_server,
            commands::remove_mcp_server,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            // Notification commands
            commands::request_notification_permission,
            // API Key commands (stored securely in OS keychain)
            commands::get_api_key,
            commands::set_api_key,
            commands::has_api_key,
            commands::clear_api_key,
            // Research commands
            commands::trigger_research,
            commands::run_research_now,
            // Legacy interest commands (for CLI compatibility)
            commands::get_interests,
            commands::add_interest,
            commands::remove_interest,
            commands::get_preferences,
            commands::update_preferences,
            // Window control commands (for popover)
            commands::open_main_window,
            commands::open_settings_window,
            commands::hide_popover,
            // Research log commands
            commands::get_research_logs,
            commands::get_actionable_errors,
            // Research state control commands
            commands::cancel_research,
            commands::reset_research_state,
            commands::get_research_status,
            // CLI installation commands
            commands::get_cli_status,
            commands::install_cli,
            commands::uninstall_cli,
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Initialize database
            db::init_database(&app_handle)?;

            // Initialize system tray (kept for Windows/Linux where it works better)
            if let Err(e) = tray::init_tray(&app_handle) {
                tracing::error!("Failed to initialize tray: {}", e);
            }

            // Register global shortcut: Cmd+Shift+B (macOS) or Ctrl+Shift+B (Windows/Linux)
            #[cfg(target_os = "macos")]
            let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyB);
            #[cfg(not(target_os = "macos"))]
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyB);

            let app_handle_for_shortcut = app_handle.clone();
            if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                // Only respond to key press, not release (prevents double-trigger)
                if event.state != ShortcutState::Pressed {
                    return;
                }
                tracing::info!("Global shortcut triggered (key pressed)");
                if let Some(window) = app_handle_for_shortcut.get_webview_window("main") {
                    match window.is_visible() {
                        Ok(true) => {
                            tracing::info!("Main window visible, hiding");
                            let _ = window.hide();
                        }
                        Ok(false) => {
                            tracing::info!("Main window hidden, showing");
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        Err(e) => {
                            tracing::error!("Failed to check window visibility: {}", e);
                        }
                    }
                }
            }) {
                tracing::error!("Failed to register global shortcut: {}", e);
            } else {
                tracing::info!("Global shortcut registered: Cmd/Ctrl+Shift+B");
            }

            Ok(())
        })
        // Handle window events
        .on_window_event(|window, event| {
            match event {
                // Hide main window on close instead of quitting (keeps tray icon active)
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label() == "main" {
                        tracing::info!("Main window close requested, hiding instead");
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    // Settings and popover windows can close normally
                }
                // Hide popover when it loses focus
                tauri::WindowEvent::Focused(focused) => {
                    if window.label() == "popover" && !focused {
                        tracing::info!("Popover lost focus, hiding");
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
