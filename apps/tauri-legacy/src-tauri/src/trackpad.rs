use serde::Serialize;
use std::ffi::c_void;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, WebviewWindow};

static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackpadTouch {
    x: f64,
    y: f64,
    phase: i32,
    touch_count: usize,
}

extern "C" {
    fn k2o_install_trackpad_capture(
        view: *mut c_void,
        callback: extern "C" fn(f64, f64, i32, usize),
    );
}

extern "C" fn handle_touch(x: f64, y: f64, phase: i32, touch_count: usize) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit(
            "trackpad-touch",
            TrackpadTouch {
                x,
                y,
                phase,
                touch_count,
            },
        );
    }
}

pub fn configure(app: &AppHandle, window: &WebviewWindow) -> tauri::Result<()> {
    let _ = APP_HANDLE.set(app.clone());
    let view = window.ns_view()? as *mut c_void;
    unsafe {
        k2o_install_trackpad_capture(view, handle_touch);
    }
    Ok(())
}

