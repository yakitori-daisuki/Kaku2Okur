use crate::platform::{self, DeliveryTarget};
use arboard::{Clipboard, ImageData};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::{borrow::Cow, thread, time::Duration};
use tiny_skia::Pixmap;

enum ClipboardSnapshot {
    Image {
        width: usize,
        height: usize,
        bytes: Vec<u8>,
    },
    Text(String),
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Dispatched,
    ClipboardFallback(String),
}

pub fn deliver(image: Pixmap, target: DeliveryTarget) -> DeliveryOutcome {
    match deliver_inner(image, target) {
        Ok(outcome) => outcome,
        Err(error) => DeliveryOutcome::ClipboardFallback(error),
    }
}

fn deliver_inner(image: Pixmap, target: DeliveryTarget) -> Result<DeliveryOutcome, String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    let previous = capture_clipboard(&mut clipboard);
    set_clipboard_image(&mut clipboard, &image)?;

    if !platform::restore_delivery_target(target) {
        return Ok(DeliveryOutcome::ClipboardFallback(
            "The previous application could not be restored".into(),
        ));
    }

    thread::sleep(Duration::from_millis(140));
    if let Err(error) = dispatch_paste() {
        return Ok(DeliveryOutcome::ClipboardFallback(error));
    }

    thread::sleep(Duration::from_millis(520));
    restore_clipboard(&mut clipboard, previous)?;
    Ok(DeliveryOutcome::Dispatched)
}

fn set_clipboard_image(clipboard: &mut Clipboard, image: &Pixmap) -> Result<(), String> {
    clipboard
        .set_image(ImageData {
            width: image.width() as usize,
            height: image.height() as usize,
            bytes: Cow::Borrowed(image.data()),
        })
        .map_err(|error| error.to_string())
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

fn restore_clipboard(clipboard: &mut Clipboard, snapshot: ClipboardSnapshot) -> Result<(), String> {
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
        ClipboardSnapshot::Text(text) => {
            clipboard.set_text(text).map_err(|error| error.to_string())
        }
        ClipboardSnapshot::Empty => clipboard.clear().map_err(|error| error.to_string()),
    }
}

fn dispatch_paste() -> Result<(), String> {
    // Check permission without requesting a system dialog on every delivery.
    // In particular, macOS may no longer trust an updated ad-hoc signed build
    // even while the older application remains listed in System Settings.
    // On failure deliver_inner keeps the image available for manual paste.
    let settings = Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    };
    let mut enigo = Enigo::new(&settings).map_err(|error| error.to_string())?;
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    enigo
        .key(modifier, Direction::Press)
        .map_err(|error| error.to_string())?;
    let paste_result = enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|error| error.to_string());
    let release_result = enigo
        .key(modifier, Direction::Release)
        .map_err(|error| error.to_string());
    paste_result.and(release_result)
}
