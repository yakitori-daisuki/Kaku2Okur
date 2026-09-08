use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::placement::show_session;

pub fn configure(app: &mut tauri::App) -> tauri::Result<()> {
    let new_sketch = MenuItem::with_id(app, "new-sketch", "New Sketch", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Kaku2Okur", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&new_sketch, &quit])?;
    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Kaku2Okur")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "new-sketch" => show_session(app),
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
                show_session(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    #[cfg(target_os = "macos")]
    {
        builder = builder.icon_as_template(true);
    }

    builder.build(app)?;
    Ok(())
}

