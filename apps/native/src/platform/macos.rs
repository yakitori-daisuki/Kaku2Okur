use crate::settings::RelativeWindowFrame;
use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct NativeFrame {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

extern "C" {
    fn k2o_set_accessory_application();
    fn k2o_system_dark_mode() -> bool;
    fn k2o_invocation_access_granted() -> bool;
    fn k2o_request_invocation_access() -> bool;
    fn k2o_open_invocation_access_settings() -> bool;
    fn k2o_modifier_key_state() -> u32;
    fn k2o_listen_invocation(
        callback: extern "C" fn(*mut c_void, u32, u16, bool),
        context: *mut c_void,
    ) -> bool;
    fn k2o_frontmost_application_pid() -> i64;
    fn k2o_activate_application(pid: i64) -> bool;
    fn k2o_configure_window(view: *mut c_void);
    fn k2o_show_window(view: *mut c_void);
    fn k2o_request_window_drag(view: *mut c_void, x: f64, y: f64);
    fn k2o_update_window_drag(view: *mut c_void, x: f64, y: f64);
    fn k2o_end_window_drag(view: *mut c_void, x: f64, y: f64);
    fn k2o_pointer_display_id() -> u64;
    fn k2o_place_window(
        view: *mut c_void,
        display_id: u64,
        has_saved_frame: bool,
        frame: NativeFrame,
    ) -> bool;
    fn k2o_preferred_window_size(
        view: *mut c_void,
        has_saved_frame: bool,
        frame: NativeFrame,
        width: *mut f64,
        height: *mut f64,
    ) -> bool;
    fn k2o_current_relative_frame(
        view: *mut c_void,
        display_id: *mut u64,
        frame: *mut NativeFrame,
    ) -> bool;
    fn k2o_install_trackpad_capture(
        view: *mut c_void,
        callback: extern "C" fn(f64, f64, i32, usize) -> bool,
        dismiss_callback: extern "C" fn(),
        finish_text_callback: extern "C" fn(),
    );
}

pub fn initialize_application() {
    unsafe { k2o_set_accessory_application() }
}

pub fn system_dark_mode() -> bool {
    unsafe { k2o_system_dark_mode() }
}

pub fn invocation_access_granted() -> bool {
    unsafe { k2o_invocation_access_granted() }
}

pub fn request_invocation_access() -> bool {
    unsafe { k2o_request_invocation_access() }
}

pub fn open_invocation_access_settings() -> bool {
    unsafe { k2o_open_invocation_access_settings() }
}

pub fn modifier_key_state() -> super::ModifierKeyState {
    super::ModifierKeyState::from_mask(unsafe { k2o_modifier_key_state() })
}

pub fn listen_invocation<F>(mut callback: F) -> Result<(), String>
where
    F: FnMut(u32, u16, bool),
{
    extern "C" fn forward<F: FnMut(u32, u16, bool)>(
        context: *mut c_void,
        mask: u32,
        key: u16,
        down: bool,
    ) {
        // The native run loop is synchronous and releases the callback before returning.
        let callback = unsafe { &mut *context.cast::<F>() };
        callback(mask, key, down);
    }
    let started = unsafe { k2o_listen_invocation(forward::<F>, (&mut callback as *mut F).cast()) };
    if started {
        Ok(())
    } else {
        Err("macOS keyboard event tap could not start or was stopped".into())
    }
}

pub fn capture_delivery_target() -> i64 {
    unsafe { k2o_frontmost_application_pid() }
}

pub fn restore_delivery_target(pid: i64) -> bool {
    unsafe { k2o_activate_application(pid) }
}

pub fn configure_and_place(
    view: *mut c_void,
    saved: Option<RelativeWindowFrame>,
) -> Option<String> {
    unsafe { k2o_configure_window(view) };
    let display_id = unsafe { k2o_pointer_display_id() };
    let frame = saved.map(NativeFrame::from).unwrap_or_default();
    let placed = unsafe { k2o_place_window(view, display_id, saved.is_some(), frame) };
    placed.then(|| display_id.to_string())
}

pub fn show_window(view: *mut c_void) {
    unsafe { k2o_show_window(view) }
}

pub fn preferred_window_size(
    view: *mut c_void,
    saved: Option<RelativeWindowFrame>,
) -> Option<(f32, f32)> {
    let mut width = 0.0;
    let mut height = 0.0;
    let frame = saved.map(NativeFrame::from).unwrap_or_default();
    let success =
        unsafe { k2o_preferred_window_size(view, saved.is_some(), frame, &mut width, &mut height) };
    success.then_some((width as f32, height as f32))
}

pub fn request_window_drag(view: *mut c_void, x: f32, y: f32) {
    unsafe { k2o_request_window_drag(view, x as f64, y as f64) }
}

pub fn update_window_drag(view: *mut c_void, x: f32, y: f32) {
    unsafe { k2o_update_window_drag(view, x as f64, y as f64) }
}

pub fn end_window_drag(view: *mut c_void, x: f32, y: f32) {
    unsafe { k2o_end_window_drag(view, x as f64, y as f64) }
}

pub fn current_relative_frame(view: *mut c_void) -> Option<(String, RelativeWindowFrame)> {
    let mut display_id = 0;
    let mut frame = NativeFrame::default();
    let success = unsafe { k2o_current_relative_frame(view, &mut display_id, &mut frame) };
    success.then(|| (display_id.to_string(), frame.into()))
}

pub fn install_trackpad_capture(
    view: *mut c_void,
    callback: extern "C" fn(f64, f64, i32, usize) -> bool,
    dismiss_callback: extern "C" fn(),
    finish_text_callback: extern "C" fn(),
) -> bool {
    unsafe { k2o_install_trackpad_capture(view, callback, dismiss_callback, finish_text_callback) };
    true
}

impl From<RelativeWindowFrame> for NativeFrame {
    fn from(value: RelativeWindowFrame) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            width: value.width as f64,
            height: value.height as f64,
        }
    }
}

impl From<NativeFrame> for RelativeWindowFrame {
    fn from(value: NativeFrame) -> Self {
        Self {
            x: value.x as f32,
            y: value.y as f32,
            width: value.width as f32,
            height: value.height as f32,
        }
    }
}
