use crate::settings::RelativeWindowFrame;
use std::ffi::c_void;
use windows_sys::Win32::{
    Foundation::{ERROR_SUCCESS, HWND, POINT, RECT},
    Graphics::Gdi::{
        EnumDisplayDevicesW, GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, DISPLAY_DEVICEW,
        HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
    },
    System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
    UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, ReleaseCapture, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_RCONTROL, VK_RMENU,
        VK_RSHIFT,
    },
    UI::WindowsAndMessaging::{
        GetCursorPos, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, SendMessageW,
        SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow,
        EDD_GET_DEVICE_INTERFACE_NAME, GWL_EXSTYLE, GWL_STYLE, HTCAPTION, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOZORDER, SW_SHOW, WM_NCLBUTTONDOWN, WS_CAPTION, WS_EX_APPWINDOW,
        WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
    },
};

pub fn modifier_key_state() -> super::ModifierKeyState {
    let pressed = |key| unsafe { GetAsyncKeyState(key as i32) } & i16::MIN != 0;
    super::ModifierKeyState {
        primary_left: pressed(VK_LCONTROL),
        primary_right: pressed(VK_RCONTROL),
        alternate_left: pressed(VK_LMENU),
        alternate_right: pressed(VK_RMENU),
        shift_left: pressed(VK_LSHIFT),
        shift_right: pressed(VK_RSHIFT),
    }
}

pub fn system_dark_mode() -> bool {
    let subkey = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
    let value_name = wide("AppsUseLightTheme");
    let mut value = 1_u32;
    let mut value_size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut value as *mut u32).cast::<c_void>(),
            &mut value_size,
        )
    };
    status == ERROR_SUCCESS && value == 0
}

pub fn capture_delivery_target() -> i64 {
    unsafe { GetForegroundWindow() as i64 }
}

pub fn restore_delivery_target(token: i64) -> bool {
    unsafe { SetForegroundWindow(token as HWND) != 0 }
}

pub fn configure_and_place(raw_hwnd: isize, saved: Option<RelativeWindowFrame>) -> Option<String> {
    let hwnd = raw_hwnd as HWND;
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            ((style & !(WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX)) | WS_THICKFRAME)
                as isize,
        );
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ((ex_style & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW) as isize,
        );
    }

    let mut cursor = POINT::default();
    unsafe { GetCursorPos(&mut cursor) };
    let monitor = unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST) };
    let (id, work) = monitor_info(monitor)?;
    let work_width = (work.right - work.left).max(1) as f32;
    let work_height = (work.bottom - work.top).max(1) as f32;
    let frame = saved.unwrap_or(RelativeWindowFrame {
        x: 0.25,
        y: 0.25,
        width: 0.5,
        height: 0.5,
    });
    let mut current = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut current) };
    let width = (current.right - current.left).max(1);
    let height = (current.bottom - current.top).max(1);
    let x = (work.left + (frame.x * work_width) as i32)
        .clamp(work.left, (work.right - width).max(work.left));
    let y = (work.top + (frame.y * work_height) as i32)
        .clamp(work.top, (work.bottom - height).max(work.top));
    unsafe {
        SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            x,
            y,
            width,
            height,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
    Some(id)
}

pub fn show_window(raw_hwnd: isize) {
    let hwnd = raw_hwnd as HWND;
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
    }
}

pub fn preferred_window_size(
    raw_hwnd: isize,
    saved: Option<RelativeWindowFrame>,
    scale_factor: f32,
) -> Option<(f32, f32)> {
    let hwnd = raw_hwnd as HWND;
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let (_, work) = monitor_info(monitor)?;
    let work_width = (work.right - work.left).max(1) as f32;
    let work_height = (work.bottom - work.top).max(1) as f32;
    let frame = saved.unwrap_or(RelativeWindowFrame {
        x: 0.25,
        y: 0.25,
        width: 0.5,
        height: 0.5,
    });
    let scale_factor = scale_factor.clamp(1.0, 4.0);
    let min_width = (680.0 * scale_factor).min(work_width);
    let min_height = (420.0 * scale_factor).min(work_height);
    let width = (work_width * frame.width.clamp(0.15, 1.0)).clamp(min_width, work_width);
    let height = (work_height * frame.height.clamp(0.15, 1.0)).clamp(min_height, work_height);
    Some((width / scale_factor, height / scale_factor))
}

pub fn request_window_drag(raw_hwnd: isize) {
    unsafe {
        ReleaseCapture();
        SendMessageW(raw_hwnd as HWND, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
    }
}

pub fn current_relative_frame(raw_hwnd: isize) -> Option<(String, RelativeWindowFrame)> {
    let hwnd = raw_hwnd as HWND;
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let (id, work) = monitor_info(monitor)?;
    let mut frame = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut frame) } == 0 {
        return None;
    }
    let work_width = (work.right - work.left).max(1) as f32;
    let work_height = (work.bottom - work.top).max(1) as f32;
    let width = ((frame.right - frame.left) as f32 / work_width).clamp(0.0, 1.0);
    let height = ((frame.bottom - frame.top) as f32 / work_height).clamp(0.0, 1.0);
    Some((
        id,
        RelativeWindowFrame {
            x: ((frame.left - work.left) as f32 / work_width).clamp(0.0, (1.0 - width).max(0.0)),
            y: ((frame.top - work.top) as f32 / work_height).clamp(0.0, (1.0 - height).max(0.0)),
            width,
            height,
        },
    ))
}

fn monitor_info(monitor: HMONITOR) -> Option<(String, RECT)> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    if unsafe {
        GetMonitorInfoW(
            monitor,
            (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>(),
        )
    } == 0
    {
        return None;
    }
    let adapter_name = utf16_z(&info.szDevice);
    let id = persistent_monitor_id(&info.szDevice).unwrap_or(adapter_name);
    Some((id, info.monitorInfo.rcWork))
}

fn persistent_monitor_id(adapter_name: &[u16; 32]) -> Option<String> {
    let mut device = DISPLAY_DEVICEW {
        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..DISPLAY_DEVICEW::default()
    };
    if unsafe {
        EnumDisplayDevicesW(
            adapter_name.as_ptr(),
            0,
            &mut device,
            EDD_GET_DEVICE_INTERFACE_NAME,
        )
    } == 0
    {
        return None;
    }
    let id = utf16_z(&device.DeviceID);
    (!id.is_empty()).then_some(id)
}

fn utf16_z(value: &[u16]) -> String {
    let end = value
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(value.len());
    String::from_utf16_lossy(&value[..end])
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
