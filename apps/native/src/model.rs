use serde::{Deserialize, Serialize};

pub type Rgba = [u8; 4];

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    pub fn from_points(a: Point, b: Point) -> Self {
        Self {
            min: Point::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    pub fn from_point(point: Point) -> Self {
        Self {
            min: point,
            max: point,
        }
    }

    pub fn width(self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn center(self) -> Point {
        Point::new(
            (self.min.x + self.max.x) * 0.5,
            (self.min.y + self.max.y) * 0.5,
        )
    }

    pub fn expanded(self, amount: f32) -> Self {
        Self {
            min: Point::new(self.min.x - amount, self.min.y - amount),
            max: Point::new(self.max.x + amount, self.max.y + amount),
        }
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            min: Point::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Point::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum CanvasObject {
    Pen {
        id: u64,
        color: Rgba,
        width: f32,
        points: Vec<Point>,
    },
    Arrow {
        id: u64,
        color: Rgba,
        width: f32,
        start: Point,
        end: Point,
    },
    Rectangle {
        id: u64,
        color: Rgba,
        width: f32,
        origin: Point,
        size: Point,
    },
    Ellipse {
        id: u64,
        color: Rgba,
        width: f32,
        center: Point,
        radius: Point,
    },
    Text {
        id: u64,
        color: Rgba,
        origin: Point,
        text: String,
        font_size: f32,
    },
}

impl CanvasObject {
    pub fn id(&self) -> u64 {
        match self {
            Self::Pen { id, .. }
            | Self::Arrow { id, .. }
            | Self::Rectangle { id, .. }
            | Self::Ellipse { id, .. }
            | Self::Text { id, .. } => *id,
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Self::Pen { width, points, .. } => points
                .iter()
                .copied()
                .map(Bounds::from_point)
                .reduce(Bounds::union)
                .unwrap_or_default()
                .expanded(*width * 0.5),
            Self::Arrow {
                width, start, end, ..
            } => Bounds::from_points(*start, *end).expanded(12.0 + *width),
            Self::Rectangle {
                width,
                origin,
                size,
                ..
            } => Bounds::from_points(*origin, *origin + *size).expanded(*width * 0.5),
            Self::Ellipse {
                width,
                center,
                radius,
                ..
            } => Bounds::from_points(
                Point::new(center.x - radius.x.abs(), center.y - radius.y.abs()),
                Point::new(center.x + radius.x.abs(), center.y + radius.y.abs()),
            )
            .expanded(*width * 0.5),
            Self::Text {
                origin,
                text,
                font_size,
                ..
            } => Bounds::from_points(
                *origin,
                *origin + crate::text::measure_text(text, *font_size),
            ),
        }
    }

    pub fn translate(&mut self, delta: Point) {
        match self {
            Self::Pen { points, .. } => {
                for point in points {
                    *point = *point + delta;
                }
            }
            Self::Arrow { start, end, .. } => {
                *start = *start + delta;
                *end = *end + delta;
            }
            Self::Rectangle { origin, .. } | Self::Text { origin, .. } => {
                *origin = *origin + delta;
            }
            Self::Ellipse { center, .. } => *center = *center + delta,
        }
    }

    pub fn scale_from(&mut self, anchor: Point, scale_x: f32, scale_y: f32) {
        let transform = |point: Point| {
            Point::new(
                anchor.x + (point.x - anchor.x) * scale_x,
                anchor.y + (point.y - anchor.y) * scale_y,
            )
        };
        match self {
            Self::Pen { width, points, .. } => {
                for point in points {
                    *point = transform(*point);
                }
                *width *= ((scale_x.abs() + scale_y.abs()) * 0.5).max(0.2);
            }
            Self::Arrow {
                width, start, end, ..
            } => {
                *start = transform(*start);
                *end = transform(*end);
                *width *= ((scale_x.abs() + scale_y.abs()) * 0.5).max(0.2);
            }
            Self::Rectangle {
                width,
                origin,
                size,
                ..
            } => {
                let end = transform(*origin + *size);
                *origin = transform(*origin);
                *size = end - *origin;
                *width *= ((scale_x.abs() + scale_y.abs()) * 0.5).max(0.2);
            }
            Self::Ellipse {
                width,
                center,
                radius,
                ..
            } => {
                *center = transform(*center);
                radius.x *= scale_x.abs();
                radius.y *= scale_y.abs();
                *width *= ((scale_x.abs() + scale_y.abs()) * 0.5).max(0.2);
            }
            Self::Text {
                origin, font_size, ..
            } => {
                *origin = transform(*origin);
                *font_size *= ((scale_x.abs() + scale_y.abs()) * 0.5).max(0.2);
            }
        }
    }

    pub fn hit_test(&self, point: Point, tolerance: f32) -> bool {
        match self {
            Self::Pen { points, width, .. } => points.windows(2).any(|line| {
                distance_to_segment(point, line[0], line[1]) <= tolerance + *width * 0.5
            }),
            Self::Arrow {
                start, end, width, ..
            } => distance_to_segment(point, *start, *end) <= tolerance + *width * 0.5,
            Self::Rectangle { .. } | Self::Ellipse { .. } | Self::Text { .. } => {
                self.bounds().expanded(tolerance).contains(point)
            }
        }
    }

    pub fn is_meaningful(&self) -> bool {
        match self {
            Self::Pen { points, .. } => points.len() >= 2,
            Self::Arrow { start, end, .. } => start.distance_to(*end) >= 3.0,
            Self::Rectangle { size, .. } => size.x.abs() >= 3.0 && size.y.abs() >= 3.0,
            Self::Ellipse { radius, .. } => radius.x.abs() >= 1.5 && radius.y.abs() >= 1.5,
            Self::Text { text, .. } => !text.trim().is_empty(),
        }
    }
}

fn distance_to_segment(point: Point, start: Point, end: Point) -> f32 {
    let line = end - start;
    let length_squared = line.x * line.x + line.y * line.y;
    if length_squared <= f32::EPSILON {
        return point.distance_to(start);
    }
    let relative = point - start;
    let t = ((relative.x * line.x + relative.y * line.y) / length_squared).clamp(0.0, 1.0);
    point.distance_to(Point::new(start.x + line.x * t, start.y + line.y * t))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RecognizedPenShape {
    Line { start: Point, end: Point },
    Rectangle { origin: Point, size: Point },
    Ellipse { center: Point, radius: Point },
}

fn recognize_pen_shape(points: &[Point]) -> Option<RecognizedPenShape> {
    let (&start, &end) = (points.first()?, points.last()?);
    if points.len() < 3 {
        return None;
    }

    let path_length: f32 = points
        .windows(2)
        .map(|pair| pair[0].distance_to(pair[1]))
        .sum();
    if path_length < 24.0 {
        return None;
    }

    let bounds = points
        .iter()
        .copied()
        .map(Bounds::from_point)
        .reduce(Bounds::union)?;
    let diagonal = bounds.min.distance_to(bounds.max).max(1.0);
    let chord = start.distance_to(end);
    let max_line_error = points
        .iter()
        .map(|point| distance_to_segment(*point, start, end))
        .fold(0.0_f32, f32::max);
    if chord >= 24.0 && chord / path_length >= 0.9 && max_line_error <= (diagonal * 0.06).max(3.0) {
        return Some(RecognizedPenShape::Line { start, end });
    }

    if start.distance_to(end) > (diagonal * 0.24).max(18.0) {
        return None;
    }

    let width = bounds.width();
    let height = bounds.height();
    if width < 24.0 || height < 24.0 || path_length < diagonal * 2.0 {
        return None;
    }

    let center = bounds.center();
    let radius = Point::new(width * 0.5, height * 0.5);
    let mut rectangle_error = 0.0;
    let mut ellipse_error = 0.0;
    let mut side_hits = [0_usize; 4];
    let mut quadrant_hits = [false; 4];

    for point in points {
        let left = ((point.x - bounds.min.x) / width).abs();
        let right = ((bounds.max.x - point.x) / width).abs();
        let top = ((point.y - bounds.min.y) / height).abs();
        let bottom = ((bounds.max.y - point.y) / height).abs();
        rectangle_error += left.min(right).min(top).min(bottom);
        if left <= 0.12 {
            side_hits[0] += 1;
        }
        if right <= 0.12 {
            side_hits[1] += 1;
        }
        if top <= 0.12 {
            side_hits[2] += 1;
        }
        if bottom <= 0.12 {
            side_hits[3] += 1;
        }

        let nx = (point.x - center.x) / radius.x;
        let ny = (point.y - center.y) / radius.y;
        ellipse_error += ((nx * nx + ny * ny).sqrt() - 1.0).abs();
        let quadrant = usize::from(nx >= 0.0) + usize::from(ny >= 0.0) * 2;
        quadrant_hits[quadrant] = true;
    }

    let count = points.len() as f32;
    rectangle_error /= count;
    ellipse_error /= count;
    let minimum_side_hits = (points.len() / 24).max(2);
    let rectangle_covered = side_hits.into_iter().all(|hits| hits >= minimum_side_hits);
    let ellipse_covered = quadrant_hits.into_iter().all(|hit| hit);
    let rectangle_perimeter = 2.0 * (width + height);
    let h = ((radius.x - radius.y).powi(2) / (radius.x + radius.y).powi(2)).clamp(0.0, 1.0);
    let ellipse_perimeter = std::f32::consts::PI
        * (radius.x + radius.y)
        * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()));

    let rectangle_valid =
        rectangle_covered && rectangle_error <= 0.11 && path_length <= rectangle_perimeter * 1.7;
    let ellipse_valid =
        ellipse_covered && ellipse_error <= 0.18 && path_length <= ellipse_perimeter * 1.8;

    if rectangle_valid && (!ellipse_valid || rectangle_error / 0.11 <= ellipse_error / 0.18) {
        Some(RecognizedPenShape::Rectangle {
            origin: bounds.min,
            size: Point::new(width, height),
        })
    } else if ellipse_valid {
        Some(RecognizedPenShape::Ellipse { center, radius })
    } else {
        None
    }
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
    pub fn current(&self) -> &[CanvasObject] {
        &self.snapshots[self.index]
    }

    pub fn commit(&mut self, objects: Vec<CanvasObject>) {
        if self.current() == objects {
            return;
        }
        self.snapshots.truncate(self.index + 1);
        self.snapshots.push(objects);
        self.index += 1;
    }

    pub fn undo(&mut self) -> Option<&[CanvasObject]> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        Some(self.current())
    }

    pub fn redo(&mut self) -> Option<&[CanvasObject]> {
        if self.index + 1 >= self.snapshots.len() {
            return None;
        }
        self.index += 1;
        Some(self.current())
    }

    pub fn can_undo(&self) -> bool {
        self.index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.index + 1 < self.snapshots.len()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.snapshots.push(Vec::new());
        self.index = 0;
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tool {
    Select,
    #[default]
    Pen,
    Eraser,
    Arrow,
    Rectangle,
    Ellipse,
    Text,
}

impl Tool {
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "select" => Some(Self::Select),
            "pen" => Some(Self::Pen),
            "eraser" => Some(Self::Eraser),
            "arrow" => Some(Self::Arrow),
            "rectangle" => Some(Self::Rectangle),
            "ellipse" => Some(Self::Ellipse),
            "text" => Some(Self::Text),
            _ => None,
        }
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Select => "select",
            Self::Pen => "pen",
            Self::Eraser => "eraser",
            Self::Arrow => "arrow",
            Self::Rectangle => "rectangle",
            Self::Ellipse => "ellipse",
            Self::Text => "text",
        }
    }
}

#[derive(Clone, Debug)]
enum Interaction {
    None,
    Creating,
    Erasing {
        before: Vec<CanvasObject>,
    },
    Moving {
        id: u64,
        pointer_origin: Point,
        original: CanvasObject,
    },
    Resizing {
        id: u64,
        anchor: Point,
        original_bounds: Bounds,
        original: CanvasObject,
    },
}

impl Default for Interaction {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointerOutcome {
    None,
    BeginText(Point),
}

#[derive(Debug, Default)]
pub struct SketchDocument {
    history: SketchHistory,
    objects: Vec<CanvasObject>,
    draft: Option<CanvasObject>,
    selected: Option<u64>,
    interaction: Interaction,
    next_id: u64,
}

impl SketchDocument {
    pub fn objects(&self) -> &[CanvasObject] {
        &self.objects
    }

    pub fn display_objects(&self) -> Vec<CanvasObject> {
        let mut objects = self.objects.clone();
        if let Some(draft) = self.draft.clone() {
            objects.push(draft);
        }
        objects
    }

    pub fn selected(&self) -> Option<u64> {
        self.selected
    }

    pub fn selection_bounds(&self) -> Option<Bounds> {
        let selected = self.selected?;
        self.objects
            .iter()
            .find(|object| object.id() == selected)
            .map(CanvasObject::bounds)
    }

    pub fn meaningful_bounds(&self) -> Option<Bounds> {
        self.objects
            .iter()
            .map(CanvasObject::bounds)
            .reduce(Bounds::union)
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn text_draggable_at(&self, tool: Tool, point: Point) -> bool {
        matches!(tool, Tool::Pen | Tool::Text | Tool::Select)
            && self
                .objects
                .iter()
                .rev()
                .find(|object| object.hit_test(point, 6.0))
                .is_some_and(|object| matches!(object, CanvasObject::Text { .. }))
    }

    /// Pointer input can grab text without changing the chosen drawing tool.
    /// Keyboard text entry continues to use Tool::Text directly.
    pub fn pointer_tool(&self, tool: Tool, point: Point) -> Tool {
        if self.text_draggable_at(tool, point) {
            Tool::Select
        } else {
            tool
        }
    }

    pub fn pointer_down(
        &mut self,
        tool: Tool,
        point: Point,
        color: Rgba,
        stroke_width: f32,
    ) -> PointerOutcome {
        self.cancel_active();
        match tool {
            Tool::Pen => {
                let id = self.allocate_id();
                self.draft = Some(CanvasObject::Pen {
                    id,
                    color,
                    width: stroke_width,
                    points: vec![point],
                });
                self.interaction = Interaction::Creating;
            }
            Tool::Arrow => {
                let id = self.allocate_id();
                self.draft = Some(CanvasObject::Arrow {
                    id,
                    color,
                    width: stroke_width,
                    start: point,
                    end: point,
                });
                self.interaction = Interaction::Creating;
            }
            Tool::Rectangle => {
                let id = self.allocate_id();
                self.draft = Some(CanvasObject::Rectangle {
                    id,
                    color,
                    width: stroke_width,
                    origin: point,
                    size: Point::default(),
                });
                self.interaction = Interaction::Creating;
            }
            Tool::Ellipse => {
                let id = self.allocate_id();
                self.draft = Some(CanvasObject::Ellipse {
                    id,
                    color,
                    width: stroke_width,
                    center: point,
                    radius: Point::default(),
                });
                self.interaction = Interaction::Creating;
            }
            Tool::Eraser => {
                self.interaction = Interaction::Erasing {
                    before: self.objects.clone(),
                };
                self.erase_at(point);
            }
            Tool::Select => self.begin_selection_interaction(point),
            Tool::Text => {
                self.selected = None;
                return PointerOutcome::BeginText(point);
            }
        }
        PointerOutcome::None
    }

    pub fn pointer_move(&mut self, point: Point) {
        match self.interaction.clone() {
            Interaction::Creating => {
                if let Some(draft) = &mut self.draft {
                    match draft {
                        CanvasObject::Pen { points, .. } => {
                            if points
                                .last()
                                .is_none_or(|last| last.distance_to(point) >= 0.8)
                            {
                                points.push(point);
                            }
                        }
                        CanvasObject::Arrow { end, .. } => *end = point,
                        CanvasObject::Rectangle { origin, size, .. } => *size = point - *origin,
                        CanvasObject::Ellipse { center, radius, .. } => {
                            *radius = Point::new(point.x - center.x, point.y - center.y)
                        }
                        CanvasObject::Text { .. } => {}
                    }
                }
            }
            Interaction::Erasing { .. } => self.erase_at(point),
            Interaction::Moving {
                id,
                pointer_origin,
                mut original,
            } => {
                original.translate(point - pointer_origin);
                self.replace_object(id, original);
            }
            Interaction::Resizing {
                id,
                anchor,
                original_bounds,
                mut original,
            } => {
                let scale_x = ((point.x - anchor.x) / original_bounds.width().max(1.0)).max(0.05);
                let scale_y = ((point.y - anchor.y) / original_bounds.height().max(1.0)).max(0.05);
                original.scale_from(anchor, scale_x, scale_y);
                self.replace_object(id, original);
            }
            Interaction::None => {}
        }
    }

    pub fn pointer_up(&mut self, point: Point) {
        if !matches!(self.interaction, Interaction::Erasing { .. }) {
            self.pointer_move(point);
        }
        self.finish_pointer_interaction();
    }

    pub fn pointer_up_current(&mut self) {
        self.finish_pointer_interaction();
    }

    pub fn snap_active_pen_to_shape(&mut self) -> bool {
        if !matches!(self.interaction, Interaction::Creating) {
            return false;
        }
        let Some(CanvasObject::Pen {
            id,
            color,
            width,
            points,
        }) = self.draft.as_ref()
        else {
            return false;
        };
        let Some(shape) = recognize_pen_shape(points) else {
            return false;
        };
        self.draft = Some(match shape {
            RecognizedPenShape::Line { start, end } => CanvasObject::Pen {
                id: *id,
                color: *color,
                width: *width,
                points: vec![start, end],
            },
            RecognizedPenShape::Rectangle { origin, size } => CanvasObject::Rectangle {
                id: *id,
                color: *color,
                width: *width,
                origin,
                size,
            },
            RecognizedPenShape::Ellipse { center, radius } => CanvasObject::Ellipse {
                id: *id,
                color: *color,
                width: *width,
                center,
                radius,
            },
        });
        true
    }

    fn finish_pointer_interaction(&mut self) {
        match std::mem::take(&mut self.interaction) {
            Interaction::Creating => {
                if let Some(draft) = self.draft.take().filter(CanvasObject::is_meaningful) {
                    self.selected = Some(draft.id());
                    self.objects.push(draft);
                    self.commit();
                }
            }
            Interaction::Erasing { before } => {
                if before != self.objects {
                    self.commit();
                }
            }
            Interaction::Moving { original, .. } | Interaction::Resizing { original, .. } => {
                if self
                    .objects
                    .iter()
                    .find(|object| object.id() == original.id())
                    != Some(&original)
                {
                    self.commit();
                }
            }
            Interaction::None => {}
        }
        self.draft = None;
    }

    pub fn commit_text(&mut self, origin: Point, text: String, color: Rgba, font_size: f32) {
        if text.trim().is_empty() {
            return;
        }
        let object = CanvasObject::Text {
            id: self.allocate_id(),
            color,
            origin,
            text,
            font_size,
        };
        self.selected = Some(object.id());
        self.objects.push(object);
        self.commit();
    }

    pub fn cancel_active(&mut self) -> bool {
        let interaction = std::mem::take(&mut self.interaction);
        self.draft = None;
        match interaction {
            Interaction::Moving { id, original, .. }
            | Interaction::Resizing { id, original, .. } => {
                self.replace_object(id, original);
                true
            }
            Interaction::Erasing { before } => {
                self.objects = before;
                true
            }
            Interaction::Creating => true,
            Interaction::None => false,
        }
    }

    pub fn clear_selection(&mut self) -> bool {
        self.selected.take().is_some()
    }

    pub fn delete_selection(&mut self) -> bool {
        let Some(selected) = self.selected.take() else {
            return false;
        };
        let old_len = self.objects.len();
        self.objects.retain(|object| object.id() != selected);
        if self.objects.len() != old_len {
            self.commit();
            return true;
        }
        false
    }

    pub fn undo(&mut self) -> bool {
        self.cancel_active();
        let Some(objects) = self.history.undo().map(<[CanvasObject]>::to_vec) else {
            return false;
        };
        self.objects = objects;
        self.selected = None;
        true
    }

    pub fn redo(&mut self) -> bool {
        self.cancel_active();
        let Some(objects) = self.history.redo().map(<[CanvasObject]>::to_vec) else {
            return false;
        };
        self.objects = objects;
        self.selected = None;
        true
    }

    pub fn clear(&mut self) {
        self.objects.clear();
        self.draft = None;
        self.selected = None;
        self.interaction = Interaction::None;
        self.history.clear();
    }

    fn begin_selection_interaction(&mut self, point: Point) {
        const HANDLE_RADIUS: f32 = 10.0;
        if let Some(selected) = self.selected {
            if let Some(original) = self
                .objects
                .iter()
                .find(|object| object.id() == selected)
                .cloned()
            {
                let bounds = original.bounds();
                if point.distance_to(bounds.max) <= HANDLE_RADIUS {
                    self.interaction = Interaction::Resizing {
                        id: selected,
                        anchor: bounds.min,
                        original_bounds: bounds,
                        original,
                    };
                    return;
                }
            }
        }

        let hit = self
            .objects
            .iter()
            .rev()
            .find(|object| object.hit_test(point, 6.0))
            .cloned();
        if let Some(original) = hit {
            let id = original.id();
            self.selected = Some(id);
            self.interaction = Interaction::Moving {
                id,
                pointer_origin: point,
                original,
            };
        } else {
            self.selected = None;
            self.interaction = Interaction::None;
        }
    }

    fn erase_at(&mut self, point: Point) {
        let hit = self
            .objects
            .iter()
            .rposition(|object| object.hit_test(point, 9.0));
        if let Some(index) = hit {
            let removed = self.objects.remove(index);
            if self.selected == Some(removed.id()) {
                self.selected = None;
            }
        }
    }

    fn replace_object(&mut self, id: u64, replacement: CanvasObject) {
        if let Some(object) = self.objects.iter_mut().find(|object| object.id() == id) {
            *object = replacement;
        }
    }

    fn allocate_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    fn commit(&mut self) {
        self.history.commit(self.objects.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Rgba = [0, 0, 0, 255];

    fn rectangle(id: u64) -> CanvasObject {
        CanvasObject::Rectangle {
            id,
            color: BLACK,
            width: 2.0,
            origin: Point::default(),
            size: Point::new(10.0, 10.0),
        }
    }

    #[test]
    fn undo_and_redo_walk_committed_snapshots() {
        let mut history = SketchHistory::default();
        history.commit(vec![rectangle(1)]);
        history.commit(vec![rectangle(1), rectangle(2)]);

        assert_eq!(history.undo().unwrap(), &[rectangle(1)]);
        assert_eq!(history.redo().unwrap(), &[rectangle(1), rectangle(2)]);
    }

    #[test]
    fn a_new_commit_replaces_the_redo_branch() {
        let mut history = SketchHistory::default();
        history.commit(vec![rectangle(1)]);
        history.commit(vec![rectangle(1), rectangle(2)]);
        history.undo();
        history.commit(vec![rectangle(3)]);

        assert!(!history.can_redo());
        assert_eq!(history.current(), &[rectangle(3)]);
    }

    #[test]
    fn drawing_and_undo_are_one_history_step() {
        let mut sketch = SketchDocument::default();
        sketch.pointer_down(Tool::Pen, Point::new(10.0, 10.0), BLACK, 3.0);
        sketch.pointer_move(Point::new(20.0, 20.0));
        sketch.pointer_up(Point::new(30.0, 20.0));

        assert_eq!(sketch.objects().len(), 1);
        assert!(sketch.undo());
        assert!(sketch.objects().is_empty());
        assert!(sketch.redo());
        assert_eq!(sketch.objects().len(), 1);
    }

    #[test]
    fn eraser_removes_the_topmost_hit_object() {
        let mut sketch = SketchDocument::default();
        sketch.commit_text(Point::new(10.0, 10.0), "first".into(), BLACK, 20.0);
        sketch.commit_text(Point::new(10.0, 10.0), "second".into(), BLACK, 20.0);

        sketch.pointer_down(Tool::Eraser, Point::new(20.0, 20.0), BLACK, 3.0);
        sketch.pointer_up(Point::new(20.0, 20.0));

        assert_eq!(sketch.objects().len(), 1);
        match &sketch.objects()[0] {
            CanvasObject::Text { text, .. } => assert_eq!(text, "first"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn selection_can_move_an_object() {
        let mut sketch = SketchDocument::default();
        sketch.commit_text(Point::new(10.0, 10.0), "move".into(), BLACK, 20.0);
        sketch.pointer_down(Tool::Select, Point::new(20.0, 20.0), BLACK, 3.0);
        sketch.pointer_move(Point::new(40.0, 50.0));
        sketch.pointer_up(Point::new(40.0, 50.0));

        match &sketch.objects()[0] {
            CanvasObject::Text { origin, .. } => assert_eq!(*origin, Point::new(30.0, 40.0)),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn text_can_be_grabbed_with_pen_or_text_and_moved_in_one_undo_step() {
        for tool in [Tool::Pen, Tool::Text] {
            let mut sketch = SketchDocument::default();
            sketch.commit_text(Point::new(10.0, 10.0), "日本語の文字".into(), BLACK, 20.0);
            let original = sketch.objects()[0].clone();
            let grab = Point::new(original.bounds().max.x - 2.0, 15.0);
            let pointer_tool = sketch.pointer_tool(tool, grab);
            assert_eq!(pointer_tool, Tool::Select);
            sketch.pointer_down(pointer_tool, grab, BLACK, 3.0);
            sketch.pointer_move(grab + Point::new(20.0, 30.0));
            sketch.pointer_up(grab + Point::new(40.0, 50.0));
            let mut moved = original.clone();
            moved.translate(Point::new(40.0, 50.0));
            assert_eq!(sketch.objects(), &[moved.clone()]);
            assert!(sketch.undo());
            assert_eq!(sketch.objects(), &[original]);
            assert!(sketch.redo());
            assert_eq!(sketch.objects(), &[moved]);
        }
    }

    #[test]
    fn direct_text_drag_can_be_cancelled_and_does_not_override_other_tools() {
        let mut sketch = SketchDocument::default();
        sketch.commit_text(Point::new(10.0, 10.0), "日本語".into(), BLACK, 20.0);
        let original = sketch.objects()[0].clone();
        let grab = Point::new(15.0, 15.0);
        for tool in [Tool::Eraser, Tool::Arrow, Tool::Rectangle, Tool::Ellipse] {
            assert_eq!(sketch.pointer_tool(tool, grab), tool);
        }
        assert_eq!(
            sketch.pointer_tool(Tool::Pen, Point::new(500.0, 500.0)),
            Tool::Pen
        );
        sketch.pointer_down(sketch.pointer_tool(Tool::Pen, grab), grab, BLACK, 3.0);
        sketch.pointer_move(Point::new(100.0, 100.0));
        assert!(sketch.cancel_active());
        assert_eq!(sketch.objects(), &[original]);
        // Typing over existing text must still start a separate text entry.
        assert_eq!(
            sketch.pointer_down(Tool::Text, grab, BLACK, 3.0),
            PointerOutcome::BeginText(grab)
        );
    }

    #[test]
    fn geometric_tools_commit_editable_objects() {
        let mut sketch = SketchDocument::default();
        for (tool, start, end) in [
            (Tool::Arrow, Point::new(10.0, 10.0), Point::new(80.0, 40.0)),
            (
                Tool::Rectangle,
                Point::new(100.0, 20.0),
                Point::new(180.0, 90.0),
            ),
            (
                Tool::Ellipse,
                Point::new(220.0, 60.0),
                Point::new(270.0, 110.0),
            ),
        ] {
            sketch.pointer_down(tool, start, BLACK, 3.0);
            sketch.pointer_up(end);
        }

        assert_eq!(sketch.objects().len(), 3);
        assert!(matches!(sketch.objects()[0], CanvasObject::Arrow { .. }));
        assert!(matches!(
            sketch.objects()[1],
            CanvasObject::Rectangle { .. }
        ));
        assert!(matches!(sketch.objects()[2], CanvasObject::Ellipse { .. }));
    }

    #[test]
    fn selection_resize_is_one_undoable_step() {
        let mut sketch = SketchDocument::default();
        sketch.pointer_down(Tool::Rectangle, Point::new(10.0, 10.0), BLACK, 2.0);
        sketch.pointer_up(Point::new(50.0, 30.0));
        let original = sketch.objects()[0].bounds();

        sketch.pointer_down(Tool::Select, original.max, BLACK, 2.0);
        sketch.pointer_up(Point::new(original.max.x + 40.0, original.max.y + 20.0));
        let resized = sketch.objects()[0].bounds();

        assert!(resized.width() > original.width());
        assert!(resized.height() > original.height());
        assert!(sketch.undo());
        assert_eq!(sketch.objects()[0].bounds(), original);
    }

    #[test]
    fn held_pen_stroke_snaps_to_a_straight_line() {
        let mut sketch = SketchDocument::default();
        sketch.pointer_down(Tool::Pen, Point::new(10.0, 20.0), BLACK, 3.0);
        for point in [
            Point::new(30.0, 31.0),
            Point::new(50.0, 39.0),
            Point::new(70.0, 50.0),
            Point::new(90.0, 60.0),
        ] {
            sketch.pointer_move(point);
        }

        assert!(sketch.snap_active_pen_to_shape());
        sketch.pointer_up_current();

        match &sketch.objects()[0] {
            CanvasObject::Pen { points, .. } => {
                assert_eq!(points, &[Point::new(10.0, 20.0), Point::new(90.0, 60.0)]);
            }
            _ => panic!("expected a straight pen line"),
        }
    }

    #[test]
    fn held_closed_pen_stroke_snaps_to_a_rectangle() {
        let mut sketch = SketchDocument::default();
        let points = [
            Point::new(20.0, 20.0),
            Point::new(60.0, 21.0),
            Point::new(100.0, 20.0),
            Point::new(101.0, 50.0),
            Point::new(100.0, 80.0),
            Point::new(60.0, 79.0),
            Point::new(20.0, 80.0),
            Point::new(19.0, 50.0),
            Point::new(20.0, 20.0),
        ];
        sketch.pointer_down(Tool::Pen, points[0], BLACK, 3.0);
        for point in &points[1..] {
            sketch.pointer_move(*point);
        }

        assert!(sketch.snap_active_pen_to_shape());
        sketch.pointer_up_current();
        assert!(matches!(
            sketch.objects()[0],
            CanvasObject::Rectangle { .. }
        ));
    }

    #[test]
    fn held_round_pen_stroke_snaps_to_an_ellipse() {
        let mut sketch = SketchDocument::default();
        let center = Point::new(120.0, 90.0);
        let radius = Point::new(60.0, 35.0);
        let points: Vec<Point> = (0..=48)
            .map(|step| {
                let angle = std::f32::consts::TAU * step as f32 / 48.0;
                Point::new(
                    center.x + radius.x * angle.cos(),
                    center.y + radius.y * angle.sin(),
                )
            })
            .collect();
        sketch.pointer_down(Tool::Pen, points[0], BLACK, 3.0);
        for point in &points[1..] {
            sketch.pointer_move(*point);
        }

        assert!(sketch.snap_active_pen_to_shape());
        sketch.pointer_up_current();
        assert!(matches!(sketch.objects()[0], CanvasObject::Ellipse { .. }));
    }

    #[test]
    fn an_open_curve_is_not_mistaken_for_a_shape() {
        let mut sketch = SketchDocument::default();
        sketch.pointer_down(Tool::Pen, Point::new(10.0, 10.0), BLACK, 3.0);
        for point in [
            Point::new(24.0, 36.0),
            Point::new(43.0, 52.0),
            Point::new(68.0, 47.0),
            Point::new(92.0, 24.0),
        ] {
            sketch.pointer_move(point);
        }

        assert!(!sketch.snap_active_pen_to_shape());
    }
}
