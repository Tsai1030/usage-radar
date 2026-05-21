mod scheduler;
mod settings;
mod sources;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition,
};

use settings::AppSettings;
use sources::{claude::ClaudeSource, codex::CodexSource, UsageSnapshot, UsageSource};

#[tauri::command]
fn get_initial_snapshots() -> Vec<UsageSnapshot> {
    let mut codex = CodexSource::default();
    let mut claude = ClaudeSource::default();
    vec![codex.poll(), claude.poll()]
}

#[tauri::command]
fn get_settings() -> AppSettings {
    settings::load()
}

#[tauri::command]
fn update_settings(settings_in: AppSettings) -> Result<(), String> {
    settings::save(&settings_in).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    // Settings is inline in the main card now — just show the window and emit
    // an event the frontend listens for to switch into settings mode.
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        let _ = win.emit("toggle-settings", true);
    }
    Ok(())
}

fn position_top_right(win: &tauri::WebviewWindow) -> tauri::Result<()> {
    if let Some(monitor) = win.current_monitor()?.or(win.primary_monitor()?) {
        let monitor_size = monitor.size();
        let scale = monitor.scale_factor();
        let win_size = win.outer_size()?;
        let margin = (16.0 * scale) as i32;
        let x = monitor_size.width as i32 - win_size.width as i32 - margin;
        let y = margin;
        win.set_position(PhysicalPosition::new(x, y))?;
    }
    Ok(())
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        match win.is_visible() {
            Ok(true) => {
                let _ = win.hide();
            }
            _ => {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let win = app
                .get_webview_window("main")
                .expect("main window is defined in tauri.conf.json");
            let _ = position_top_right(&win);
            let _ = win.show();

            let show_item = MenuItemBuilder::with_id("show", "Show").build(app)?;
            let hide_item = MenuItemBuilder::with_id("hide", "Hide").build(app)?;
            let settings_item =
                MenuItemBuilder::with_id("settings", "Settings…").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&show_item, &hide_item, &settings_item, &quit_item])
                .build()?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().expect("icon configured").clone())
                .tooltip("Usage Radar")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "settings" => {
                        let _ = open_settings(app.clone());
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            scheduler::spawn(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_snapshots,
            get_settings,
            update_settings,
            open_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
