use crate::model::Tool;
use serde::{Deserialize, Serialize};

const CORNER_DEPTH: f64 = 0.14;
const EDGE_DEPTH: f64 = 0.10;
const EDGE_HALF_SPAN: f64 = 0.13;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TrackpadHotspot {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl TrackpadHotspot {
    pub const ALL: [Self; 8] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Right,
        Self::BottomRight,
        Self::Bottom,
        Self::BottomLeft,
        Self::Left,
    ];

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    pub const fn index(self) -> usize {
        match self {
            Self::TopLeft => 0,
            Self::Top => 1,
            Self::TopRight => 2,
            Self::Right => 3,
            Self::BottomRight => 4,
            Self::Bottom => 5,
            Self::BottomLeft => 6,
            Self::Left => 7,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::Top => "Top edge",
            Self::TopRight => "Top right",
            Self::Right => "Right edge",
            Self::BottomRight => "Bottom right",
            Self::Bottom => "Bottom edge",
            Self::BottomLeft => "Bottom left",
            Self::Left => "Left edge",
        }
    }

    pub fn contains(self, x: f64, y: f64) -> bool {
        let x = x.clamp(0.0, 1.0);
        let y = y.clamp(0.0, 1.0);
        match self {
            Self::TopLeft => x <= CORNER_DEPTH && y <= CORNER_DEPTH,
            Self::Top => y <= EDGE_DEPTH && (x - 0.5).abs() <= EDGE_HALF_SPAN,
            Self::TopRight => x >= 1.0 - CORNER_DEPTH && y <= CORNER_DEPTH,
            Self::Right => x >= 1.0 - EDGE_DEPTH && (y - 0.5).abs() <= EDGE_HALF_SPAN,
            Self::BottomRight => x >= 1.0 - CORNER_DEPTH && y >= 1.0 - CORNER_DEPTH,
            Self::Bottom => y >= 1.0 - EDGE_DEPTH && (x - 0.5).abs() <= EDGE_HALF_SPAN,
            Self::BottomLeft => x <= CORNER_DEPTH && y >= 1.0 - CORNER_DEPTH,
            Self::Left => x <= EDGE_DEPTH && (y - 0.5).abs() <= EDGE_HALF_SPAN,
        }
    }

    pub fn at(x: f64, y: f64) -> Option<Self> {
        Self::ALL.into_iter().find(|hotspot| hotspot.contains(x, y))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotspotTrigger {
    Touch,
    #[default]
    LongPress,
    Click,
}

impl HotspotTrigger {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "touch" => Some(Self::Touch),
            "long-press" => Some(Self::LongPress),
            "click" => Some(Self::Click),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Touch => "touch",
            Self::LongPress => "long-press",
            Self::Click => "click",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HotspotAction {
    #[default]
    None,
    Select,
    Pen,
    Eraser,
    Arrow,
    Rectangle,
    Ellipse,
    Text,
    Undo,
    Redo,
    Delete,
    Send,
}

impl HotspotAction {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "none" => Some(Self::None),
            "select" => Some(Self::Select),
            "pen" => Some(Self::Pen),
            "eraser" => Some(Self::Eraser),
            "arrow" => Some(Self::Arrow),
            "rectangle" => Some(Self::Rectangle),
            "ellipse" => Some(Self::Ellipse),
            "text" => Some(Self::Text),
            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "delete" => Some(Self::Delete),
            "send" => Some(Self::Send),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Select => "select",
            Self::Pen => "pen",
            Self::Eraser => "eraser",
            Self::Arrow => "arrow",
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Text => "text",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Delete => "delete",
            Self::Send => "send",
        }
    }

    pub const fn tool(self) -> Option<Tool> {
        match self {
            Self::Select => Some(Tool::Select),
            Self::Pen => Some(Tool::Pen),
            Self::Eraser => Some(Tool::Eraser),
            Self::Arrow => Some(Tool::Arrow),
            Self::Rectangle => Some(Tool::Rectangle),
            Self::Ellipse => Some(Tool::Ellipse),
            Self::Text => Some(Tool::Text),
            Self::None | Self::Undo | Self::Redo | Self::Delete | Self::Send => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct HotspotBinding {
    pub trigger: HotspotTrigger,
    pub action: HotspotAction,
}

pub const fn default_hotspot_bindings() -> [HotspotBinding; 8] {
    [HotspotBinding {
        trigger: HotspotTrigger::LongPress,
        action: HotspotAction::None,
    }; 8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eight_hotspots_are_detected() {
        let samples = [
            ((0.02, 0.02), TrackpadHotspot::TopLeft),
            ((0.5, 0.02), TrackpadHotspot::Top),
            ((0.98, 0.02), TrackpadHotspot::TopRight),
            ((0.98, 0.5), TrackpadHotspot::Right),
            ((0.98, 0.98), TrackpadHotspot::BottomRight),
            ((0.5, 0.98), TrackpadHotspot::Bottom),
            ((0.02, 0.98), TrackpadHotspot::BottomLeft),
            ((0.02, 0.5), TrackpadHotspot::Left),
        ];

        for ((x, y), expected) in samples {
            assert_eq!(TrackpadHotspot::at(x, y), Some(expected));
        }
    }

    #[test]
    fn drawing_area_does_not_resolve_to_a_hotspot() {
        assert_eq!(TrackpadHotspot::at(0.5, 0.5), None);
        assert_eq!(TrackpadHotspot::at(0.25, 0.08), None);
        assert_eq!(TrackpadHotspot::at(0.08, 0.25), None);
    }

    #[test]
    fn action_ids_and_tool_mapping_round_trip() {
        for action in [
            HotspotAction::None,
            HotspotAction::Select,
            HotspotAction::Pen,
            HotspotAction::Eraser,
            HotspotAction::Arrow,
            HotspotAction::Rectangle,
            HotspotAction::Ellipse,
            HotspotAction::Text,
            HotspotAction::Undo,
            HotspotAction::Redo,
            HotspotAction::Delete,
            HotspotAction::Send,
        ] {
            assert_eq!(HotspotAction::from_id(action.id()), Some(action));
        }
        assert_eq!(HotspotAction::Pen.tool(), Some(Tool::Pen));
        assert_eq!(HotspotAction::Send.tool(), None);
    }
}
