use crate::{
    model::{Rgba, Tool},
    trackpad::{default_hotspot_bindings, HotspotBinding},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackgroundPreference {
    #[default]
    System,
    White,
    Black,
    Custom,
}

impl BackgroundPreference {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "system" => Some(Self::System),
            "white" => Some(Self::White),
            "black" => Some(Self::Black),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::White => "white",
            Self::Black => "black",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SendBackgroundPolicy {
    #[default]
    Visible,
    White,
    Black,
    Custom,
}

impl SendBackgroundPolicy {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "visible" => Some(Self::Visible),
            "white" => Some(Self::White),
            "black" => Some(Self::Black),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Visible => "visible",
            Self::White => "white",
            Self::Black => "black",
            Self::Custom => "custom",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationGesture {
    DualModifier,
    DoubleModifier,
    #[default]
    DualAlternate,
    DoubleAlternate,
    Conventional,
}

impl InvocationGesture {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "dual-modifier" => Some(Self::DualModifier),
            "double-modifier" => Some(Self::DoubleModifier),
            "dual-alternate" => Some(Self::DualAlternate),
            "double-alternate" => Some(Self::DoubleAlternate),
            "conventional" => Some(Self::Conventional),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::DualModifier => "dual-modifier",
            Self::DoubleModifier => "double-modifier",
            Self::DualAlternate => "dual-alternate",
            Self::DoubleAlternate => "double-alternate",
            Self::Conventional => "conventional",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct RelativeWindowFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct Settings {
    pub canvas_background: BackgroundPreference,
    pub custom_canvas_color: String,
    pub send_background: SendBackgroundPolicy,
    pub custom_send_color: String,
    pub invocation_gesture: InvocationGesture,
    pub invocation_gesture_configured: bool,
    pub stroke_color: String,
    pub stroke_color_automatic: bool,
    pub stroke_width: f32,
    pub auto_shape_enabled: bool,
    pub font_size: f32,
    pub last_tool: Tool,
    pub trackpad_mode_default: bool,
    pub trackpad_hotspots: [HotspotBinding; 8],
    pub display_frames: HashMap<String, RelativeWindowFrame>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            canvas_background: BackgroundPreference::System,
            custom_canvas_color: "#ffffff".into(),
            send_background: SendBackgroundPolicy::Visible,
            custom_send_color: "#ffffff".into(),
            invocation_gesture: InvocationGesture::DualAlternate,
            invocation_gesture_configured: false,
            stroke_color: "#1f2328".into(),
            stroke_color_automatic: true,
            stroke_width: 3.0,
            auto_shape_enabled: true,
            font_size: 24.0,
            last_tool: Tool::Pen,
            trackpad_mode_default: false,
            trackpad_hotspots: default_hotspot_bindings(),
            display_frames: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        fs::read_to_string(path)
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .map(Self::migrate)
            .unwrap_or_default()
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(path) = settings_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    pub fn canvas_color(&self, system_dark: bool) -> Rgba {
        match self.canvas_background {
            BackgroundPreference::System if system_dark => [17, 19, 21, 255],
            BackgroundPreference::System | BackgroundPreference::White => [255, 255, 255, 255],
            BackgroundPreference::Black => [17, 19, 21, 255],
            BackgroundPreference::Custom => {
                parse_hex_color(&self.custom_canvas_color).unwrap_or([255, 255, 255, 255])
            }
        }
    }

    pub fn send_color(&self, visible: Rgba) -> Rgba {
        match self.send_background {
            SendBackgroundPolicy::Visible => visible,
            SendBackgroundPolicy::White => [255, 255, 255, 255],
            SendBackgroundPolicy::Black => [17, 19, 21, 255],
            SendBackgroundPolicy::Custom => {
                parse_hex_color(&self.custom_send_color).unwrap_or([255, 255, 255, 255])
            }
        }
    }

    pub fn stroke_color(&self) -> Rgba {
        parse_hex_color(&self.stroke_color).unwrap_or([31, 35, 40, 255])
    }

    pub fn effective_stroke_color(&self, canvas: Rgba) -> Rgba {
        if !self.stroke_color_automatic {
            return self.stroke_color();
        }
        let luminance =
            0.2126 * canvas[0] as f32 + 0.7152 * canvas[1] as f32 + 0.0722 * canvas[2] as f32;
        if luminance < 150.0 {
            [255, 255, 255, 255]
        } else {
            [31, 35, 40, 255]
        }
    }

    fn migrate(mut self) -> Self {
        if !self.invocation_gesture_configured
            && self.invocation_gesture == InvocationGesture::DualModifier
        {
            self.invocation_gesture = InvocationGesture::DualAlternate;
        }
        self
    }
}

pub fn parse_hex_color(value: &str) -> Option<Rgba> {
    let value = value.strip_prefix('#').unwrap_or(value);
    match value.len() {
        6 => Some([
            u8::from_str_radix(&value[0..2], 16).ok()?,
            u8::from_str_radix(&value[2..4], 16).ok()?,
            u8::from_str_radix(&value[4..6], 16).ok()?,
            255,
        ]),
        8 => Some([
            u8::from_str_radix(&value[0..2], 16).ok()?,
            u8::from_str_radix(&value[2..4], 16).ok()?,
            u8::from_str_radix(&value[4..6], 16).ok()?,
            u8::from_str_radix(&value[6..8], 16).ok()?,
        ]),
        _ => None,
    }
}

fn settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join("Library/Application Support/Kaku2Okur/settings.json"))
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = std::env::var_os("APPDATA")?;
        Some(PathBuf::from(app_data).join("Kaku2Okur/settings.json"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(config).join("kaku2okur/settings.json"));
        }
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config/kaku2okur/settings.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_preserves_background_policies() {
        let settings = Settings {
            canvas_background: BackgroundPreference::Custom,
            custom_canvas_color: "#123456".into(),
            send_background: SendBackgroundPolicy::White,
            auto_shape_enabled: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let decoded: Settings = serde_json::from_str(&json).unwrap();

        assert_eq!(settings, decoded);
        assert_eq!(decoded.canvas_color(false), [0x12, 0x34, 0x56, 255]);
        assert_eq!(decoded.send_color([0, 0, 0, 255]), [255, 255, 255, 255]);
        assert!(!decoded.auto_shape_enabled);
    }

    #[test]
    fn parses_rgb_and_rgba_hex_values() {
        assert_eq!(parse_hex_color("#ffffff"), Some([255, 255, 255, 255]));
        assert_eq!(parse_hex_color("10203040"), Some([16, 32, 48, 64]));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn automatic_stroke_color_contrasts_with_the_canvas() {
        let settings = Settings::default();
        assert_eq!(
            settings.effective_stroke_color([17, 19, 21, 255]),
            [255, 255, 255, 255]
        );
        assert_eq!(
            settings.effective_stroke_color([255, 255, 255, 255]),
            [31, 35, 40, 255]
        );
    }

    #[test]
    fn default_invocation_uses_both_option_or_alt_keys() {
        assert_eq!(
            Settings::default().invocation_gesture,
            InvocationGesture::DualAlternate
        );
    }

    #[test]
    fn migrates_the_previous_unconfigured_invocation_default() {
        let settings = Settings {
            invocation_gesture: InvocationGesture::DualModifier,
            invocation_gesture_configured: false,
            ..Settings::default()
        }
        .migrate();

        assert_eq!(
            settings.invocation_gesture,
            InvocationGesture::DualAlternate
        );
    }

    #[test]
    fn preserves_an_explicit_command_or_control_invocation_choice() {
        let settings = Settings {
            invocation_gesture: InvocationGesture::DualModifier,
            invocation_gesture_configured: true,
            ..Settings::default()
        }
        .migrate();

        assert_eq!(settings.invocation_gesture, InvocationGesture::DualModifier);
    }
}
