use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;
use tracing::{info, warn, error};
use std::process::Command;

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
    info!("Sending research complete notification (count: {}, sound: {})", count, enable_sound);

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
    let mut builder = app.notification()
        .builder()
        .title(title)
        .body(&body);

    if enable_sound {
        builder = builder.sound("default");
    }

    match builder.show() {
        Ok(_) => {
            info!("Tauri notification sent successfully");
        }
        Err(e) => {
            warn!("Tauri notification failed: {}", e);
        }
    }

    // Also try native macOS notification as fallback (more reliable in dev mode)
    #[cfg(target_os = "macos")]
    {
        let sound_option = if enable_sound { "sound name \"Glass\"" } else { "" };
        let script = format!(
            r#"display notification "{}" with title "{}" {}"#,
            escape_applescript(&body),
            escape_applescript(title),
            sound_option
        );

        match Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Native macOS notification sent successfully");
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
                            matches!(new_state, tauri_plugin_notification::PermissionState::Granted)
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
