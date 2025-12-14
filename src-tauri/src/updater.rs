use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;
use tracing::{info, warn, error};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Track if an update has been downloaded and is ready to install
static UPDATE_READY: AtomicBool = AtomicBool::new(false);

/// Event payload for update available notification
#[derive(Clone, serde::Serialize)]
pub struct UpdateAvailableEvent {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// Event payload for update downloaded notification
#[derive(Clone, serde::Serialize)]
pub struct UpdateDownloadedEvent {
    pub version: String,
}

/// Event payload for download progress
#[derive(Clone, serde::Serialize)]
pub struct UpdateProgressEvent {
    pub downloaded: u64,
    pub total: Option<u64>,
}

/// Check for updates on startup
pub async fn check_for_updates(app: AppHandle) -> Result<(), String> {
    info!("Checking for updates...");

    let updater = app.updater().map_err(|e| {
        warn!("Failed to get updater: {}", e);
        e.to_string()
    })?;

    match updater.check().await {
        Ok(Some(update)) => {
            let version = update.version.clone();
            let notes = update.body.clone();
            let date = update.date.map(|d| d.to_string());

            info!("Update available: v{}", version);

            // Emit event so frontend knows update is available
            let _ = app.emit("update:available", UpdateAvailableEvent {
                version: version.clone(),
                notes: notes.clone(),
                date,
            });

            // Download in background
            info!("Downloading update v{}...", version);

            let app_for_progress = app.clone();
            let downloaded_bytes = Arc::new(AtomicU64::new(0));
            let downloaded_bytes_clone = downloaded_bytes.clone();

            let bytes = update.download(
                move |chunk_length, content_length| {
                    let total_downloaded = downloaded_bytes_clone.fetch_add(chunk_length as u64, Ordering::SeqCst) + chunk_length as u64;
                    // Emit progress events
                    let _ = app_for_progress.emit("update:progress", UpdateProgressEvent {
                        downloaded: total_downloaded,
                        total: content_length,
                    });
                },
                || {
                    info!("Update download complete");
                }
            ).await;

            match bytes {
                Ok(update_bytes) => {
                    info!("Download complete, installing update...");

                    // Install the update (stages files for next restart)
                    if let Err(e) = update.install(update_bytes) {
                        error!("Failed to install update: {}", e);
                        return Err(e.to_string());
                    }

                    // Mark that update is ready
                    UPDATE_READY.store(true, Ordering::SeqCst);

                    // Emit downloaded event
                    let _ = app.emit("update:downloaded", UpdateDownloadedEvent {
                        version: version.clone(),
                    });

                    // Send native notification
                    notify_update_available(&app, &version, notes.as_deref())?;

                    info!("Update v{} installed and ready - restart to apply", version);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to download update: {}", e);
                    Err(e.to_string())
                }
            }
        }
        Ok(None) => {
            info!("No updates available, running latest version");
            Ok(())
        }
        Err(e) => {
            // Don't treat update check failures as fatal errors
            warn!("Update check failed: {}", e);
            Ok(())
        }
    }
}

/// Check if an update is downloaded and ready to install
pub fn is_update_ready() -> bool {
    UPDATE_READY.load(Ordering::SeqCst)
}

/// Send native notification about available update
fn notify_update_available(app: &AppHandle, version: &str, notes: Option<&str>) -> Result<(), String> {
    let title = "Update Available";
    const MAX_NOTIFICATION_LENGTH: usize = 100;
    let body = match notes {
        Some(n) if !n.is_empty() => {
            // Truncate notes if too long for notification (UTF-8 safe)
            let truncated: String = n.chars().take(MAX_NOTIFICATION_LENGTH).collect();
            let truncated = if n.chars().count() > MAX_NOTIFICATION_LENGTH {
                format!("{}...", truncated)
            } else {
                truncated
            };
            format!("Claudius v{} is ready.\n{}", version, truncated)
        }
        _ => format!("Claudius v{} is ready to install. Restart to update.", version),
    };

    match app.notification()
        .builder()
        .title(title)
        .body(&body)
        .sound("default")
        .show()
    {
        Ok(_) => {
            info!("Update notification sent");
            Ok(())
        }
        Err(e) => {
            warn!("Failed to send update notification: {}", e);
            // Don't fail the overall process just because notification failed
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_available_event_serialization() {
        let event = UpdateAvailableEvent {
            version: "1.0.0".to_string(),
            notes: Some("Bug fixes and improvements".to_string()),
            date: Some("2025-01-01".to_string()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"notes\":\"Bug fixes and improvements\""));
        assert!(json.contains("\"date\":\"2025-01-01\""));
    }

    #[test]
    fn test_update_available_event_with_null_fields() {
        let event = UpdateAvailableEvent {
            version: "1.0.0".to_string(),
            notes: None,
            date: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"notes\":null"));
        assert!(json.contains("\"date\":null"));
    }

    #[test]
    fn test_update_downloaded_event_serialization() {
        let event = UpdateDownloadedEvent {
            version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
    }

    #[test]
    fn test_update_progress_event_serialization() {
        let event = UpdateProgressEvent {
            downloaded: 1024,
            total: Some(4096),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"downloaded\":1024"));
        assert!(json.contains("\"total\":4096"));
    }

    #[test]
    fn test_update_progress_event_unknown_total() {
        let event = UpdateProgressEvent {
            downloaded: 1024,
            total: None,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"downloaded\":1024"));
        assert!(json.contains("\"total\":null"));
    }

    #[test]
    fn test_is_update_ready_initial_state() {
        // Reset to known state
        UPDATE_READY.store(false, Ordering::SeqCst);
        assert!(!is_update_ready());
    }

    #[test]
    fn test_is_update_ready_after_update() {
        UPDATE_READY.store(true, Ordering::SeqCst);
        assert!(is_update_ready());
        // Reset for other tests
        UPDATE_READY.store(false, Ordering::SeqCst);
    }
}
