use rdev::{listen, Event, EventType, Key};
use std::thread;
use tauri::Manager;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

use crate::{core::AppState, placement::show_session};

pub fn conventional_shortcut() -> Shortcut {
    #[cfg(target_os = "macos")]
    let modifiers = Modifiers::META | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let modifiers = Modifiers::CONTROL | Modifiers::SHIFT;
    Shortcut::new(Some(modifiers), Code::Space)
}

pub fn register_conventional_shortcut(app: &tauri::AppHandle) -> Result<(), String> {
    app.global_shortcut()
        .register(conventional_shortcut())
        .map_err(|error| error.to_string())
}

pub fn global_shortcut_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let shortcut = conventional_shortcut();
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, triggered, event| {
            if triggered == &shortcut && event.state() == ShortcutState::Pressed {
                show_session(app);
            }
        })
        .build()
}

pub fn start_dual_modifier_listener(app: tauri::AppHandle) {
    thread::spawn(move || {
        let mut left_down = false;
        let mut right_down = false;
        let mut armed = true;
        let callback = move |event: Event| {
            #[cfg(target_os = "macos")]
            let (left_key, right_key) = (Key::MetaLeft, Key::MetaRight);
            #[cfg(not(target_os = "macos"))]
            let (left_key, right_key) = (Key::ControlLeft, Key::ControlRight);

            match event.event_type {
                EventType::KeyPress(key) if key == left_key => left_down = true,
                EventType::KeyPress(key) if key == right_key => right_down = true,
                EventType::KeyRelease(key) if key == left_key => left_down = false,
                EventType::KeyRelease(key) if key == right_key => right_down = false,
                _ => {}
            }

            if left_down && right_down && armed {
                armed = false;
                let enabled = app
                    .try_state::<AppState>()
                    .map(|state| state.dual_modifier_enabled())
                    .unwrap_or(true);
                if enabled {
                    show_session(&app);
                }
            }
            if !left_down && !right_down {
                armed = true;
            }
        };

        if let Err(error) = listen(callback) {
            log::warn!("Dual-modifier listener unavailable: {error:?}");
        }
    });
}
