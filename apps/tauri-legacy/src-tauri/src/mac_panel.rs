use tauri::{AppHandle, Manager};
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};

tauri_panel! {
    panel!(CanvasPanel {
        config: {
            can_become_key_window: true,
            can_become_main_window: false,
            becomes_key_only_if_needed: true,
            is_floating_panel: true
        }
    })
}

pub fn configure(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| tauri::Error::WindowNotFound)?;
    let panel = window.to_panel::<CanvasPanel>()?;
    panel.set_level(PanelLevel::Floating.value());
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().resizable().into());
    panel.set_collection_behavior(
        CollectionBehavior::new()
            .full_screen_auxiliary()
            .can_join_all_spaces()
            .into(),
    );
    panel.set_hides_on_deactivate(false);
    panel.set_works_when_modal(true);
    panel.set_floating_panel(true);
    panel.set_has_shadow(true);
    panel.set_corner_radius(8.0);
    Ok(())
}

pub fn show(app: &AppHandle) -> bool {
    match app.get_webview_panel("main") {
        Ok(panel) => {
            panel.show_and_make_key();
            true
        }
        Err(error) => {
            log::warn!("Could not show NSPanel: {error:?}");
            false
        }
    }
}

pub fn hide(app: &AppHandle) -> bool {
    match app.get_webview_panel("main") {
        Ok(panel) => {
            panel.hide();
            true
        }
        Err(error) => {
            log::warn!("Could not hide NSPanel: {error:?}");
            false
        }
    }
}
