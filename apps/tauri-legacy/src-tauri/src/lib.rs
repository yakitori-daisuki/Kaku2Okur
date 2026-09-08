mod core;
mod delivery;
mod invocation;
#[cfg(target_os = "macos")]
mod mac_panel;
mod placement;
#[cfg(target_os = "macos")]
mod trackpad;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .manage(core::AppState::default())
        .plugin(invocation::global_shortcut_plugin())
        .invoke_handler(tauri::generate_handler![
            core::commit_scene,
            core::current_scene,
            core::set_invocation_gesture,
            delivery::deliver_image,
            delivery::discard_session,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                mac_panel::configure(app.handle())?;
            }
            tray::configure(app)?;
            invocation::register_conventional_shortcut(app.handle())?;
            invocation::start_dual_modifier_listener(app.handle().clone());
            if let Some(window) = app.get_webview_window("main") {
                placement::place_on_pointer_display(&window)?;
                #[cfg(target_os = "macos")]
                trackpad::configure(app.handle(), &window)?;
            }
            #[cfg(debug_assertions)]
            placement::show_session(app.handle());
            Ok(())
        });

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running Kaku2Okur");
}
