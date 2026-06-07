use tauri::{AppHandle, Manager, PhysicalPosition, Position, WebviewUrl, WebviewWindowBuilder, Window};

use crate::features::config::{read_config, save_position, WindowPosition};

#[tauri::command]
pub fn get_window_position(window: Window) -> Result<WindowPosition, String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    Ok(WindowPosition {
        x: position.x,
        y: position.y,
    })
}

#[tauri::command]
pub fn set_window_position(app: AppHandle, window: Window, position: WindowPosition) -> Result<(), String> {
    window
        .set_position(Position::Physical(PhysicalPosition {
            x: position.x,
            y: position.y,
        }))
        .map_err(|error| error.to_string())?;
    save_position(&app, position)?;
    Ok(())
}

fn present_settings_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    let _ = window.unminimize();
    window
        .center()
        .map_err(|error| error.to_string())?;
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        return present_settings_window(&window);
    }

    let window = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("index.html".into()))
        .title("uwu companion settings")
        .inner_size(560.0, 640.0)
        .resizable(true)
        .decorations(true)
        .visible(false)
        .build()
        .map_err(|error| error.to_string())?;

    present_settings_window(&window)
}

#[tauri::command]
pub fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        window.hide().map_err(|error| error.to_string())?;
        let _ = window.set_always_on_top(false);
    }

    Ok(())
}

#[tauri::command]
pub fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

pub fn restore_main_window_position(app: &AppHandle) {
    let Ok(config) = read_config(app) else {
        return;
    };

    let Some(position) = config.position else {
        return;
    };

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_position(Position::Physical(PhysicalPosition {
            x: position.x,
            y: position.y,
        }));
    }
}