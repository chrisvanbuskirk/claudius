#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;
mod scheduler;
mod tray;
mod notifications;
mod research;
mod research_state;
mod mcp_client;
mod research_log;

use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use scheduler::Scheduler;

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
        .manage(Arc::new(Scheduler::new()))
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
            // API Key commands (stored securely in OS keychain)
            commands::get_api_key,
            commands::set_api_key,
            commands::has_api_key,
            commands::clear_api_key,
            // Research commands
            commands::trigger_research,
            commands::run_research_now,
            // Scheduler commands
            get_scheduler_status,
            start_scheduler,
            stop_scheduler,
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
            get_next_run_time,
            // Research log commands
            commands::get_research_logs,
            commands::get_actionable_errors,
            // Research state control commands
            commands::cancel_research,
            commands::reset_research_state,
            commands::get_research_status,
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

            // Start scheduler with saved preferences
            let scheduler = app.state::<Arc<Scheduler>>();
            let scheduler_clone = scheduler.inner().clone();
            let app_handle_for_scheduler = app_handle.clone();

            // Spawn async task to start scheduler
            tauri::async_runtime::spawn(async move {
                // Set app handle on scheduler for notifications
                scheduler_clone.set_app_handle(app_handle_for_scheduler).await;

                // Load schedule from preferences
                match commands::get_settings() {
                    Ok(settings) => {
                        let schedule = settings.schedule_cron;
                        tracing::info!("Starting scheduler with schedule: {}", schedule);

                        if let Err(e) = scheduler_clone.start(&schedule).await {
                            tracing::error!("Failed to start scheduler: {}", e);
                        } else {
                            tracing::info!("Scheduler started successfully");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Could not load settings, using default schedule: {}", e);
                        // Use default schedule
                        if let Err(e) = scheduler_clone.start("0 6 * * *").await {
                            tracing::error!("Failed to start scheduler with default: {}", e);
                        }
                    }
                }
            });

            Ok(())
        })
        // Handle window events
        .on_window_event(|window, event| {
            match event {
                // Hide main window on close instead of quitting (keeps scheduler running)
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

/// Get the current scheduler status
#[tauri::command]
async fn get_scheduler_status(
    scheduler: tauri::State<'_, Arc<Scheduler>>
) -> Result<serde_json::Value, String> {
    let is_running = scheduler.is_running().await;
    let schedule = scheduler.get_schedule().await;

    Ok(serde_json::json!({
        "running": is_running,
        "schedule": schedule
    }))
}

/// Start the scheduler with the given cron expression
#[tauri::command]
async fn start_scheduler(
    scheduler: tauri::State<'_, Arc<Scheduler>>,
    cron_expr: String
) -> Result<(), String> {
    scheduler.start(&cron_expr).await
}

/// Stop the scheduler
#[tauri::command]
async fn stop_scheduler(
    scheduler: tauri::State<'_, Arc<Scheduler>>
) -> Result<(), String> {
    scheduler.stop().await;
    Ok(())
}

/// Get the next scheduled run time
#[tauri::command]
async fn get_next_run_time(
    scheduler: tauri::State<'_, Arc<Scheduler>>
) -> Result<Option<String>, String> {
    Ok(scheduler.get_next_run_time().await)
}
