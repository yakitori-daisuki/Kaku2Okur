use crate::settings::RelativeWindowFrame;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::listen_invocation;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeliveryTarget(i64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierKeyState {
    pub primary_left: bool,
    pub primary_right: bool,
    pub alternate_left: bool,
    pub alternate_right: bool,
    pub shift_left: bool,
    pub shift_right: bool,
}

impl ModifierKeyState {
    pub const fn from_mask(mask: u32) -> Self {
        Self {
            primary_left: mask & (1 << 0) != 0,
            primary_right: mask & (1 << 1) != 0,
            alternate_left: mask & (1 << 2) != 0,
            alternate_right: mask & (1 << 3) != 0,
            shift_left: mask & (1 << 4) != 0,
            shift_right: mask & (1 << 5) != 0,
        }
    }
}

impl DeliveryTarget {
    pub fn is_available(self) -> bool {
        self.0 != 0
    }
}

pub fn initialize_application() {
    #[cfg(target_os = "macos")]
    macos::initialize_application();
}

pub fn system_dark_mode() -> bool {
    #[cfg(target_os = "macos")]
    return macos::system_dark_mode();
    #[cfg(target_os = "windows")]
    return windows::system_dark_mode();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    false
}

pub fn invocation_access_granted() -> bool {
    #[cfg(target_os = "macos")]
    return macos::invocation_access_granted();
    #[cfg(not(target_os = "macos"))]
    true
}

pub fn request_invocation_access() -> bool {
    #[cfg(target_os = "macos")]
    return macos::request_invocation_access();
    #[cfg(not(target_os = "macos"))]
    true
}

pub fn open_invocation_access_settings() -> bool {
    #[cfg(target_os = "macos")]
    return macos::open_invocation_access_settings();
    #[cfg(not(target_os = "macos"))]
    false
}

pub fn modifier_key_state() -> Option<ModifierKeyState> {
    #[cfg(target_os = "macos")]
    return Some(macos::modifier_key_state());
    #[cfg(target_os = "windows")]
    return Some(windows::modifier_key_state());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    None
}

pub fn capture_delivery_target() -> DeliveryTarget {
    #[cfg(target_os = "macos")]
    return DeliveryTarget(macos::capture_delivery_target());
    #[cfg(target_os = "windows")]
    return DeliveryTarget(windows::capture_delivery_target());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    DeliveryTarget::default()
}

pub fn restore_delivery_target(target: DeliveryTarget) -> bool {
    if !target.is_available() {
        return false;
    }
    #[cfg(target_os = "macos")]
    return macos::restore_delivery_target(target.0);
    #[cfg(target_os = "windows")]
    return windows::restore_delivery_target(target.0);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    false
}

pub fn configure_and_place_window(
    window: &slint::Window,
    saved: Option<RelativeWindowFrame>,
) -> Option<String> {
    let handle = raw_window_handle(window)?;
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        return macos::configure_and_place(handle.ns_view.as_ptr(), saved);
    }
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(handle) = handle {
        return windows::configure_and_place(handle.hwnd.get(), saved);
    }
    None
}

pub fn show_window(window: &slint::Window) {
    let _ = window.show();
    let Some(handle) = raw_window_handle(window) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        macos::show_window(handle.ns_view.as_ptr());
    }
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(handle) = handle {
        windows::show_window(handle.hwnd.get());
    }
}

pub fn preferred_window_size(
    window: &slint::Window,
    saved: Option<RelativeWindowFrame>,
) -> Option<(f32, f32)> {
    let handle = raw_window_handle(window)?;
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        return macos::preferred_window_size(handle.ns_view.as_ptr(), saved);
    }
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(handle) = handle {
        return windows::preferred_window_size(handle.hwnd.get(), saved, window.scale_factor());
    }
    None
}

pub fn hide_window(window: &slint::Window) {
    let _ = window.hide();
}

pub fn request_window_drag(window: &slint::Window, x: f32, y: f32) {
    let Some(handle) = raw_window_handle(window) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        macos::request_window_drag(handle.ns_view.as_ptr(), x, y);
    }
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(handle) = handle {
        windows::request_window_drag(handle.hwnd.get());
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (x, y);
}

pub fn update_window_drag(window: &slint::Window, x: f32, y: f32) {
    let Some(handle) = raw_window_handle(window) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        macos::update_window_drag(handle.ns_view.as_ptr(), x, y);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (handle, x, y);
}

pub fn end_window_drag(window: &slint::Window, x: f32, y: f32) {
    let Some(handle) = raw_window_handle(window) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        macos::end_window_drag(handle.ns_view.as_ptr(), x, y);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (handle, x, y);
}

pub fn current_relative_frame(window: &slint::Window) -> Option<(String, RelativeWindowFrame)> {
    let handle = raw_window_handle(window)?;
    #[cfg(target_os = "macos")]
    if let RawWindowHandle::AppKit(handle) = handle {
        return macos::current_relative_frame(handle.ns_view.as_ptr());
    }
    #[cfg(target_os = "windows")]
    if let RawWindowHandle::Win32(handle) = handle {
        return windows::current_relative_frame(handle.hwnd.get());
    }
    None
}

pub fn install_trackpad_capture(
    window: &slint::Window,
    callback: extern "C" fn(f64, f64, i32, usize) -> bool,
    dismiss_callback: extern "C" fn(),
    finish_text_callback: extern "C" fn(),
) -> bool {
    #[cfg(target_os = "macos")]
    {
        let Some(RawWindowHandle::AppKit(handle)) = raw_window_handle(window) else {
            return false;
        };
        macos::install_trackpad_capture(
            handle.ns_view.as_ptr(),
            callback,
            dismiss_callback,
            finish_text_callback,
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (window, callback, dismiss_callback, finish_text_callback);
        false
    }
}

fn raw_window_handle(window: &slint::Window) -> Option<RawWindowHandle> {
    window
        .window_handle()
        .window_handle()
        .ok()
        .map(|handle| handle.as_raw())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifier_mask_keeps_left_and_right_keys_separate() {
        let state = ModifierKeyState::from_mask((1 << 1) | (1 << 2) | (1 << 5));

        assert!(!state.primary_left);
        assert!(state.primary_right);
        assert!(state.alternate_left);
        assert!(!state.alternate_right);
        assert!(!state.shift_left);
        assert!(state.shift_right);
    }
}
