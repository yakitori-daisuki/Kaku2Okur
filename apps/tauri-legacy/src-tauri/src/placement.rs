use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewWindow};

pub fn place_on_pointer_display(window: &WebviewWindow) -> tauri::Result<()> {
    let pointer = window.cursor_position()?;
    let monitors = window.available_monitors()?;
    let monitor = monitors.iter().find(|monitor| {
        let origin = monitor.position();
        let size = monitor.size();
        pointer.x >= origin.x as f64
            && pointer.x < (origin.x + size.width as i32) as f64
            && pointer.y >= origin.y as f64
            && pointer.y < (origin.y + size.height as i32) as f64
    });

    if let Some(monitor) = monitor {
        let origin = monitor.position();
        let available = monitor.size();
        let width = (available.width / 2).clamp(680, 1100);
        let height = (available.height / 2).clamp(420, 760);
        let x = origin.x + (available.width.saturating_sub(width) / 2) as i32;
        let y = origin.y + (available.height.saturating_sub(height) / 2) as i32;
        window.set_size(Size::Physical(PhysicalSize::new(width, height)))?;
        window.set_position(Position::Physical(PhysicalPosition::new(x, y)))?;
    } else {
        window.center()?;
    }
    Ok(())
}

pub fn show_session(app: &tauri::AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(error) = place_on_pointer_display(&window) {
        log::warn!("Could not position canvas: {error}");
    }
    #[cfg(target_os = "macos")]
    if !crate::mac_panel::show(app) {
        return;
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Err(error) = window.show() {
            log::warn!("Could not show canvas: {error}");
            return;
        }
        if let Err(error) = window.set_focus() {
            log::warn!("Could not focus canvas: {error}");
        }
    }
    let _ = window.emit("session-opened", ());
}

pub fn hide_session(app: &tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    if crate::mac_panel::hide(app) {
        return Ok(());
    }

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Canvas window is unavailable".to_owned())?;
    window.hide().map_err(|error| error.to_string())
}
