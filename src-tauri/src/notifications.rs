use std::process::Command;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tracing::{error, info, warn};

/// Escape a string for safe use in AppleScript.
/// Escapes backslashes, double quotes, and newlines.
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Send a notification that research is complete.
pub fn notify_research_complete(
    app: &AppHandle,
    count: usize,
    enable_sound: bool,
) -> Result<(), String> {
    info!(
        "Sending research complete notification (count: {}, sound: {})",
        count, enable_sound
    );

    let title = "Research Complete";
    let body = if count == 1 {
        "1 new briefing ready!".to_string()
    } else {
        format!("{} new briefings ready!", count)
    };

    // Check permission state first
    match app.notification().permission_state() {
        Ok(state) => {
            info!("Notification permission state: {:?}", state);
        }
        Err(e) => {
            warn!("Could not check notification permission: {}", e);
        }
    }

    // Try Tauri notification first
    // Note: On macOS, the app icon is automatically used from the bundle in production
    // In dev mode, notifications may show a generic icon
    let mut builder = app.notification().builder().title(title).body(&body);

    if enable_sound {
        builder = builder.sound("default");
    }

    // Try to set icon path (helps in dev mode on some platforms)
    #[cfg(target_os = "macos")]
    {
        // Use the app's icon.icns from the icons directory
        if let Ok(resource_dir) = app.path().resource_dir() {
            let icon_path = resource_dir.join("icons").join("icon.icns");
            if icon_path.exists() {
                builder = builder.icon(icon_path.to_string_lossy().to_string());
            }
        }
    }

    let tauri_success = match builder.show() {
        Ok(_) => {
            info!("Tauri notification sent successfully");
            true
        }
        Err(e) => {
            warn!("Tauri notification failed: {}", e);
            false
        }
    };

    // Only use AppleScript fallback if Tauri notification failed
    // Note: AppleScript notifications show Script Editor icon, not app icon
    #[cfg(target_os = "macos")]
    if !tauri_success {
        let sound_option = if enable_sound {
            "sound name \"Glass\""
        } else {
            ""
        };
        let script = format!(
            r#"display notification "{}" with title "{}" {}"#,
            escape_applescript(&body),
            escape_applescript(title),
            sound_option
        );

        match Command::new("osascript").arg("-e").arg(&script).output() {
            Ok(output) => {
                if output.status.success() {
                    info!("Native macOS notification sent (fallback)");
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Native macOS notification failed: {}", stderr);
                }
            }
            Err(e) => {
                warn!("Failed to run osascript: {}", e);
            }
        }
    }

    Ok(())
}

/// Send a notification for research errors.
pub fn notify_research_error(app: &AppHandle, error_message: &str) -> Result<(), String> {
    warn!("Sending research error notification: {}", error_message);

    app.notification()
        .builder()
        .title("Research Failed")
        .body(error_message)
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Check if notifications are allowed.
pub async fn check_notification_permission(app: &AppHandle) -> bool {
    match app.notification().permission_state() {
        Ok(state) => {
            match state {
                tauri_plugin_notification::PermissionState::Granted => true,
                tauri_plugin_notification::PermissionState::Denied => {
                    warn!("Notifications are denied");
                    false
                }
                tauri_plugin_notification::PermissionState::Prompt
                | tauri_plugin_notification::PermissionState::PromptWithRationale => {
                    info!("Notification permission needs to be requested");
                    // Request permission
                    match app.notification().request_permission() {
                        Ok(new_state) => {
                            matches!(
                                new_state,
                                tauri_plugin_notification::PermissionState::Granted
                            )
                        }
                        Err(e) => {
                            error!("Failed to request notification permission: {}", e);
                            false
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to check notification permission: {}", e);
            false
        }
    }
}
