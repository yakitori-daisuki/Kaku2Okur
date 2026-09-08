use arboard::{Clipboard, ImageData};
use base64::Engine;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use image::ImageReader;
use serde::Serialize;
use std::{borrow::Cow, io::Cursor, thread, time::Duration};
use crate::core::AppState;
use crate::placement::hide_session;

enum ClipboardSnapshot {
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    },
    Text(String),
    Empty,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryOutcome {
    status: &'static str,
    message: Option<String>,
}

fn decode_png(data_url: &str) -> Result<(usize, usize, Vec<u8>), String> {
    let encoded = data_url
        .split_once(',')
        .map(|(_, payload)| payload)
        .ok_or_else(|| "Invalid image payload".to_owned())?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| error.to_string())?;
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| error.to_string())?
        .to_rgba8();
    let (width, height) = image.dimensions();
    Ok((width as usize, height as usize, image.into_raw()))
}

fn capture_clipboard(clipboard: &mut Clipboard) -> ClipboardSnapshot {
    if let Ok(image) = clipboard.get_image() {
        return ClipboardSnapshot::Image {
            width: image.width,
            height: image.height,
            bytes: image.bytes.into_owned(),
        };
    }
    if let Ok(text) = clipboard.get_text() {
        return ClipboardSnapshot::Text(text);
    }
    ClipboardSnapshot::Empty
}

fn restore_clipboard(
    clipboard: &mut Clipboard,
    snapshot: ClipboardSnapshot,
) -> Result<(), String> {
    match snapshot {
        ClipboardSnapshot::Image {
            width,
            height,
            bytes,
        } => clipboard
            .set_image(ImageData {
                width,
                height,
                bytes: Cow::Owned(bytes),
            })
            .map_err(|error| error.to_string()),
        ClipboardSnapshot::Text(text) => clipboard.set_text(text).map_err(|error| error.to_string()),
        ClipboardSnapshot::Empty => clipboard.clear().map_err(|error| error.to_string()),
    }
}

fn dispatch_paste() -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|error| error.to_string())?;
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Direction::Press)
        .map_err(|error| error.to_string())?;
    let click_result = enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|error| error.to_string());
    let release_result = enigo
        .key(modifier, Direction::Release)
        .map_err(|error| error.to_string());
    click_result.and(release_result)
}

fn deliver_blocking(data_url: String) -> Result<DeliveryOutcome, String> {
    let (width, height, bytes) = decode_png(&data_url)?;
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    let previous = capture_clipboard(&mut clipboard);
    clipboard
        .set_image(ImageData {
            width,
            height,
            bytes: Cow::Owned(bytes),
        })
        .map_err(|error| error.to_string())?;

    thread::sleep(Duration::from_millis(140));
    if let Err(error) = dispatch_paste() {
        return Ok(DeliveryOutcome {
            status: "clipboardFallback",
            message: Some(error),
        });
    }

    thread::sleep(Duration::from_millis(520));
    restore_clipboard(&mut clipboard, previous)?;
    Ok(DeliveryOutcome {
        status: "dispatched",
        message: None,
    })
}

#[tauri::command]
pub async fn deliver_image(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    data_url: String,
) -> Result<DeliveryOutcome, String> {
    hide_session(&app)?;

    let outcome = tauri::async_runtime::spawn_blocking(move || deliver_blocking(data_url))
        .await
        .map_err(|error| error.to_string())??;
    if outcome.status == "dispatched" {
        state.history.lock().clear();
    }
    Ok(outcome)
}

#[tauri::command]
pub fn discard_session(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.history.lock().clear();
    hide_session(&app)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_png_data_url() {
        let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([0, 0, 0, 0]));
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let encoded = base64::engine::general_purpose::STANDARD.encode(png.into_inner());
        let data_url = format!("data:image/png;base64,{encoded}");
        let (width, height, bytes) = decode_png(&data_url).unwrap();
        assert_eq!((width, height), (1, 1));
        assert_eq!(bytes.len(), 4);
    }
}
