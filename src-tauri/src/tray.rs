use tauri::{
    tray::TrayIconBuilder,
    menu::{Menu, MenuItem},
    Manager, AppHandle,
    image::Image,
};
use tauri_plugin_positioner::{Position, WindowExt};
use tracing::{info, warn};

/// Load the tray icon from embedded bytes.
fn load_tray_icon() -> Image<'static> {
    // Dedicated tray icon (22x22 PNG for macOS menu bar)
    // This is embedded at compile time from src-tauri/icons/tray/icon.png
    let tray_icon_bytes = include_bytes!("../icons/tray/icon.png");

    let img = image::load_from_memory(tray_icon_bytes)
        .expect("Failed to load embedded tray icon");
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    info!("Loaded tray icon: {}x{}", width, height);
    Image::new_owned(rgba.into_raw(), width, height)
}

/// Initialize the system tray icon and event handlers.
pub fn init_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let icon = load_tray_icon();

    // Create a menu for right-click only
    let show_item = MenuItem::with_id(app, "show", "Show Popover", true, None::<&str>)?;
    let open_app = MenuItem::with_id(app, "open_app", "Open Full App", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &open_app, &quit])?;

    // NOTE: Due to Tauri 2.0 bug #11413, on_tray_icon_event doesn't receive Click events on macOS.
    // Workaround: Show menu on left click so users can access popover via menu.
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true) // macOS: treat as template image (adapts to light/dark menu bar)
        .tooltip("Claudius - Research Assistant")
        .menu(&menu)
        .show_menu_on_left_click(true) // Show menu on left click as workaround for macOS click bug
        .on_menu_event(|app, event| {
            info!("Tray menu event: {:?}", event.id.as_ref());
            match event.id.as_ref() {
                "show" => {
                    toggle_popover(app);
                }
                "open_app" => {
                    show_main_window(app);
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    info!("Tray icon initialized successfully");
    Ok(())
}

/// Toggle the popover window visibility.
fn toggle_popover(app: &AppHandle) {
    info!("Tray clicked - toggling popover");
    if let Some(window) = app.get_webview_window("popover") {
        match window.is_visible() {
            Ok(true) => {
                info!("Popover is visible, hiding it");
                if let Err(e) = window.hide() {
                    warn!("Failed to hide popover: {}", e);
                }
            }
            Ok(false) => {
                info!("Popover is hidden, showing it");
                // Position the popover in the top-right area near the tray
                if let Err(e) = window.move_window(Position::TopRight) {
                    warn!("Failed to position popover: {}", e);
                }
                if let Err(e) = window.show() {
                    warn!("Failed to show popover: {}", e);
                }
                if let Err(e) = window.set_focus() {
                    warn!("Failed to focus popover: {}", e);
                }
                info!("Popover shown and focused");
            }
            Err(e) => {
                warn!("Failed to check popover visibility: {}", e);
            }
        }
    } else {
        warn!("Popover window not found");
    }
}

/// Show the main window and hide the popover.
pub fn show_main_window(app: &AppHandle) {
    // Show main window
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        info!("Opened main window");
    }

    // Hide popover
    if let Some(popover) = app.get_webview_window("popover") {
        let _ = popover.hide();
    }
}

/// Show the settings window and hide the popover.
pub fn show_settings_window(app: &AppHandle) {
    // Show settings window
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        info!("Opened settings window");
    }

    // Hide popover
    if let Some(popover) = app.get_webview_window("popover") {
        let _ = popover.hide();
    }
}

/// Hide the popover window.
pub fn hide_popover(app: &AppHandle) {
    if let Some(popover) = app.get_webview_window("popover") {
        let _ = popover.hide();
    }
}
