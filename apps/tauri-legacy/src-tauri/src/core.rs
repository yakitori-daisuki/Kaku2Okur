use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", rename_all_fields = "camelCase")]
pub enum CanvasObject {
    Pen {
        id: String,
        stroke: String,
        stroke_width: f64,
        points: Vec<f64>,
    },
    Arrow {
        id: String,
        stroke: String,
        stroke_width: f64,
        points: [f64; 4],
    },
    Rectangle {
        id: String,
        stroke: String,
        stroke_width: f64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Ellipse {
        id: String,
        stroke: String,
        stroke_width: f64,
        x: f64,
        y: f64,
        radius_x: f64,
        radius_y: f64,
    },
    Text {
        id: String,
        stroke: String,
        stroke_width: f64,
        x: f64,
        y: f64,
        width: f64,
        text: String,
        font_size: f64,
    },
}

#[derive(Debug)]
pub struct SketchHistory {
    snapshots: Vec<Vec<CanvasObject>>,
    index: usize,
}

impl Default for SketchHistory {
    fn default() -> Self {
        Self {
            snapshots: vec![Vec::new()],
            index: 0,
        }
    }
}

impl SketchHistory {
    pub fn current(&self) -> Vec<CanvasObject> {
        self.snapshots[self.index].clone()
    }

    pub fn commit(&mut self, objects: Vec<CanvasObject>) {
        if self.snapshots[self.index] == objects {
            return;
        }
        self.snapshots.truncate(self.index + 1);
        self.snapshots.push(objects);
        self.index += 1;
    }

    pub fn clear(&mut self) {
        self.snapshots = vec![Vec::new()];
        self.index = 0;
    }
}

#[derive(Debug)]
pub struct AppState {
    pub history: Mutex<SketchHistory>,
    dual_modifier_enabled: AtomicBool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            history: Mutex::new(SketchHistory::default()),
            dual_modifier_enabled: AtomicBool::new(true),
        }
    }
}

impl AppState {
    pub fn dual_modifier_enabled(&self) -> bool {
        self.dual_modifier_enabled.load(Ordering::Relaxed)
    }

    pub fn set_dual_modifier_enabled(&self, enabled: bool) {
        self.dual_modifier_enabled.store(enabled, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationGesture {
    DualModifier,
    Conventional,
}

#[tauri::command]
pub fn commit_scene(state: tauri::State<'_, AppState>, objects: Vec<CanvasObject>) {
    state.history.lock().commit(objects);
}

#[tauri::command]
pub fn current_scene(state: tauri::State<'_, AppState>) -> Vec<CanvasObject> {
    state.history.lock().current()
}

#[tauri::command]
pub fn set_invocation_gesture(
    state: tauri::State<'_, AppState>,
    gesture: InvocationGesture,
) {
    state.set_dual_modifier_enabled(matches!(gesture, InvocationGesture::DualModifier));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rectangle(id: &str) -> CanvasObject {
        CanvasObject::Rectangle {
            id: id.to_owned(),
            stroke: "#000000".to_owned(),
            stroke_width: 3.0,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }
    }

    #[test]
    fn commit_truncates_a_replaced_branch() {
        let mut history = SketchHistory::default();
        history.commit(vec![rectangle("one")]);
        history.clear();
        history.commit(vec![rectangle("two")]);
        assert_eq!(history.current(), vec![rectangle("two")]);
        assert_eq!(history.snapshots.len(), 2);
    }
}
