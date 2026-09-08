use crate::model::{Bounds, CanvasObject, Point, Rgba};
use crate::text::glyph_for;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, PixmapMut, Stroke, StrokeDash, Transform};
use ttf_parser::OutlineBuilder;

const SELECTION_COLOR: Rgba = [23, 105, 255, 255];

#[derive(Clone, Copy)]
struct ViewTransform {
    offset: Point,
    scale: f32,
}

pub struct CanvasRenderOptions<'a> {
    pub pixel_scale: f32,
    pub background: Rgba,
    pub objects: &'a [CanvasObject],
    pub selection: Option<Bounds>,
}

impl ViewTransform {
    fn map(self, point: Point) -> Point {
        Point::new(
            (point.x - self.offset.x) * self.scale,
            (point.y - self.offset.y) * self.scale,
        )
    }

    fn map_bounds(self, bounds: Bounds) -> Bounds {
        Bounds::from_points(self.map(bounds.min), self.map(bounds.max))
    }
}

pub struct SceneRenderer;

impl Default for SceneRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render_canvas(
        &self,
        width: u32,
        height: u32,
        pixel_scale: f32,
        background: Rgba,
        objects: &[CanvasObject],
        selection: Option<Bounds>,
    ) -> Pixmap {
        let pixel_scale = pixel_scale.clamp(1.0, 3.0);
        let (pixel_width, pixel_height) = Self::canvas_pixel_dimensions(width, height, pixel_scale);
        let mut pixmap = blank_pixmap(pixel_width, pixel_height, background);
        {
            let mut target = pixmap.as_mut();
            self.render_canvas_target(&mut target, pixel_scale, objects, selection);
        }
        pixmap
    }

    pub fn canvas_pixel_dimensions(width: u32, height: u32, pixel_scale: f32) -> (u32, u32) {
        let pixel_scale = pixel_scale.clamp(1.0, 3.0);
        (
            (width as f32 * pixel_scale).round().max(1.0) as u32,
            (height as f32 * pixel_scale).round().max(1.0) as u32,
        )
    }

    pub fn render_canvas_into(
        &self,
        pixels: &mut [u8],
        pixel_width: u32,
        pixel_height: u32,
        options: CanvasRenderOptions<'_>,
    ) {
        let mut target = PixmapMut::from_bytes(pixels, pixel_width, pixel_height)
            .expect("pixel buffer matches canvas dimensions");
        target.fill(Color::from_rgba8(
            options.background[0],
            options.background[1],
            options.background[2],
            options.background[3],
        ));
        self.render_canvas_target(
            &mut target,
            options.pixel_scale,
            options.objects,
            options.selection,
        );
    }

    pub fn render_send_result(
        &self,
        background: Rgba,
        objects: &[CanvasObject],
        padding: f32,
    ) -> Option<Pixmap> {
        let bounds = objects
            .iter()
            .map(CanvasObject::bounds)
            .reduce(Bounds::union)?
            .expanded(padding.max(0.0));

        let natural_width = bounds.width().ceil().max(1.0);
        let natural_height = bounds.height().ceil().max(1.0);
        let scale = 2.0_f32
            .min(4096.0 / natural_width)
            .min(4096.0 / natural_height)
            .max(1.0);
        let width = (natural_width * scale).ceil() as u32;
        let height = (natural_height * scale).ceil() as u32;
        let mut pixmap = blank_pixmap(width, height, background);
        let view = ViewTransform {
            offset: bounds.min,
            scale,
        };
        {
            let mut target = pixmap.as_mut();
            for object in objects {
                self.render_object(&mut target, object, view);
            }
        }
        Some(pixmap)
    }

    fn render_canvas_target(
        &self,
        target: &mut PixmapMut<'_>,
        pixel_scale: f32,
        objects: &[CanvasObject],
        selection: Option<Bounds>,
    ) {
        let view = ViewTransform {
            offset: Point::default(),
            scale: pixel_scale,
        };
        for object in objects {
            self.render_object(target, object, view);
        }
        if let Some(bounds) = selection {
            render_selection(target, bounds, view);
        }
    }

    fn render_object(
        &self,
        pixmap: &mut PixmapMut<'_>,
        object: &CanvasObject,
        view: ViewTransform,
    ) {
        match object {
            CanvasObject::Pen {
                color,
                width,
                points,
                ..
            } => {
                let Some(first) = points.first().copied() else {
                    return;
                };
                let mut builder = PathBuilder::new();
                let first = view.map(first);
                builder.move_to(first.x, first.y);
                for point in &points[1..] {
                    let point = view.map(*point);
                    builder.line_to(point.x, point.y);
                }
                if let Some(path) = builder.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint(*color),
                        &stroke(*width * view.scale),
                        Transform::identity(),
                        None,
                    );
                }
            }
            CanvasObject::Arrow {
                color,
                width,
                start,
                end,
                ..
            } => {
                let start = view.map(*start);
                let end = view.map(*end);
                let mut builder = PathBuilder::new();
                builder.move_to(start.x, start.y);
                builder.line_to(end.x, end.y);
                let angle = (end.y - start.y).atan2(end.x - start.x);
                let head = ((10.0 + width * 2.0).min(24.0)) * view.scale;
                for offset in [2.55_f32, -2.55_f32] {
                    builder.move_to(end.x, end.y);
                    builder.line_to(
                        end.x + (angle + offset).cos() * head,
                        end.y + (angle + offset).sin() * head,
                    );
                }
                if let Some(path) = builder.finish() {
                    pixmap.stroke_path(
                        &path,
                        &paint(*color),
                        &stroke(*width * view.scale),
                        Transform::identity(),
                        None,
                    );
                }
            }
            CanvasObject::Rectangle {
                color,
                width,
                origin,
                size,
                ..
            } => {
                let bounds = view.map_bounds(Bounds::from_points(*origin, *origin + *size));
                if let Some(rect) = tiny_skia::Rect::from_ltrb(
                    bounds.min.x,
                    bounds.min.y,
                    bounds.max.x,
                    bounds.max.y,
                ) {
                    pixmap.stroke_path(
                        &PathBuilder::from_rect(rect),
                        &paint(*color),
                        &stroke(*width * view.scale),
                        Transform::identity(),
                        None,
                    );
                }
            }
            CanvasObject::Ellipse {
                color,
                width,
                center,
                radius,
                ..
            } => {
                let bounds = view.map_bounds(Bounds::from_points(
                    Point::new(center.x - radius.x.abs(), center.y - radius.y.abs()),
                    Point::new(center.x + radius.x.abs(), center.y + radius.y.abs()),
                ));
                if let Some(rect) = tiny_skia::Rect::from_ltrb(
                    bounds.min.x,
                    bounds.min.y,
                    bounds.max.x,
                    bounds.max.y,
                ) {
                    if let Some(path) = PathBuilder::from_oval(rect) {
                        pixmap.stroke_path(
                            &path,
                            &paint(*color),
                            &stroke(*width * view.scale),
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            CanvasObject::Text {
                color,
                origin,
                text,
                font_size,
                ..
            } => self.render_text(
                pixmap,
                view.map(*origin),
                text,
                *font_size * view.scale,
                *color,
            ),
        }
    }

    fn render_text(
        &self,
        pixmap: &mut PixmapMut<'_>,
        origin: Point,
        text: &str,
        font_size: f32,
        color: Rgba,
    ) {
        let line_height = font_size * 1.25;
        let mut cursor_x = origin.x;
        let mut baseline = origin.y + font_size;

        for character in text.chars() {
            if character == '\n' {
                cursor_x = origin.x;
                baseline += line_height;
                continue;
            }
            let Some((face, glyph)) = glyph_for(character) else {
                cursor_x += font_size * 0.58;
                continue;
            };
            let scale = font_size / face.units_per_em() as f32;
            let mut builder = GlyphPathBuilder::new(cursor_x, baseline, scale);
            if face.outline_glyph(glyph, &mut builder).is_some() {
                if let Some(path) = builder.finish() {
                    pixmap.fill_path(
                        &path,
                        &paint(color),
                        tiny_skia::FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            cursor_x += face
                .glyph_hor_advance(glyph)
                .map(|advance| advance as f32 * scale)
                .unwrap_or(font_size * 0.58);
        }
    }
}

fn blank_pixmap(width: u32, height: u32, background: Rgba) -> Pixmap {
    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("valid canvas dimensions");
    pixmap.fill(Color::from_rgba8(
        background[0],
        background[1],
        background[2],
        background[3],
    ));
    pixmap
}

fn paint(color: Rgba) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color[0], color[1], color[2], color[3]);
    paint.anti_alias = true;
    paint
}

fn stroke(width: f32) -> Stroke {
    Stroke {
        width: width.max(0.5),
        line_cap: tiny_skia::LineCap::Round,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    }
}

fn render_selection(pixmap: &mut PixmapMut<'_>, bounds: Bounds, view: ViewTransform) {
    let bounds = view.map_bounds(bounds.expanded(4.0));
    let Some(rect) =
        tiny_skia::Rect::from_ltrb(bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y)
    else {
        return;
    };
    let mut outline = stroke(1.5 * view.scale);
    outline.dash = StrokeDash::new(vec![5.0 * view.scale, 4.0 * view.scale], 0.0);
    pixmap.stroke_path(
        &PathBuilder::from_rect(rect),
        &paint(SELECTION_COLOR),
        &outline,
        Transform::identity(),
        None,
    );

    let handle_size = 8.0 * view.scale;
    if let Some(handle) = tiny_skia::Rect::from_xywh(
        bounds.max.x - handle_size * 0.5,
        bounds.max.y - handle_size * 0.5,
        handle_size,
        handle_size,
    ) {
        pixmap.fill_rect(handle, &paint(SELECTION_COLOR), Transform::identity(), None);
    }
}

struct GlyphPathBuilder {
    builder: PathBuilder,
    origin_x: f32,
    baseline: f32,
    scale: f32,
}

impl GlyphPathBuilder {
    fn new(origin_x: f32, baseline: f32, scale: f32) -> Self {
        Self {
            builder: PathBuilder::new(),
            origin_x,
            baseline,
            scale,
        }
    }

    fn point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.origin_x + x * self.scale,
            self.baseline - y * self.scale,
        )
    }

    fn finish(self) -> Option<tiny_skia::Path> {
        self.builder.finish()
    }
}

impl OutlineBuilder for GlyphPathBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.builder.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let (x, y) = self.point(x, y);
        self.builder.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x, y) = self.point(x, y);
        self.builder.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (x1, y1) = self.point(x1, y1);
        let (x2, y2) = self.point(x2, y2);
        let (x, y) = self.point(x, y);
        self.builder.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drawing_changes_pixels_without_changing_the_background_corners() {
        let renderer = SceneRenderer::new();
        let objects = vec![CanvasObject::Rectangle {
            id: 1,
            color: [0, 0, 0, 255],
            width: 2.0,
            origin: Point::new(4.0, 4.0),
            size: Point::new(10.0, 10.0),
        }];
        let pixmap = renderer.render_canvas(20, 20, 1.0, [255, 255, 255, 255], &objects, None);

        assert_eq!(&pixmap.data()[0..4], &[255, 255, 255, 255]);
        assert!(pixmap.data().chunks_exact(4).any(|pixel| pixel[0] < 255));
    }

    #[test]
    fn selection_keeps_the_object_opaque_and_adds_a_visible_handle() {
        let renderer = SceneRenderer::new();
        let object = CanvasObject::Rectangle {
            id: 1,
            color: [0, 0, 0, 255],
            width: 2.0,
            origin: Point::new(20.0, 20.0),
            size: Point::new(40.0, 40.0),
        };
        let selection = object.bounds();
        let pixmap = renderer.render_canvas(
            80,
            80,
            1.0,
            [255, 255, 255, 255],
            &[object],
            Some(selection),
        );

        assert!(pixmap
            .data()
            .chunks_exact(4)
            .any(|pixel| pixel == [0, 0, 0, 255]));
        assert!(pixmap
            .data()
            .chunks_exact(4)
            .any(|pixel| pixel == SELECTION_COLOR));
    }

    #[test]
    fn send_result_is_cropped_and_scaled() {
        let renderer = SceneRenderer::new();
        let objects = vec![CanvasObject::Rectangle {
            id: 1,
            color: [0, 0, 0, 255],
            width: 2.0,
            origin: Point::new(300.0, 200.0),
            size: Point::new(100.0, 50.0),
        }];
        let pixmap = renderer
            .render_send_result([255, 255, 255, 255], &objects, 16.0)
            .unwrap();

        assert!(pixmap.width() < 300);
        assert!(pixmap.height() < 200);
        assert!(pixmap.width() > 200);
    }

    #[test]
    fn text_is_rendered_with_a_mapped_system_font() {
        let renderer = SceneRenderer::new();
        let objects = vec![CanvasObject::Text {
            id: 1,
            color: [0, 0, 0, 255],
            origin: Point::new(8.0, 8.0),
            text: "Kaku2Okur".into(),
            font_size: 24.0,
        }];
        let pixmap = renderer.render_canvas(180, 60, 1.0, [255, 255, 255, 255], &objects, None);

        assert!(pixmap
            .data()
            .chunks_exact(4)
            .any(|pixel| pixel[0] < 220 && pixel[3] == 255));
    }

    #[test]
    fn japanese_text_hit_area_covers_the_rendered_right_edge() {
        let renderer = SceneRenderer::new();
        let object = CanvasObject::Text {
            id: 1,
            color: [0, 0, 0, 255],
            origin: Point::new(10.0, 10.0),
            text: "日本語の文字をつかんで移動".into(),
            font_size: 24.0,
        };
        let canvas =
            renderer.render_canvas(600, 80, 1.0, [255, 255, 255, 255], &[object.clone()], None);
        let rightmost = canvas
            .data()
            .chunks_exact(4)
            .enumerate()
            .filter(|(_, pixel)| pixel[0] < 128)
            .map(|(index, _)| index % 600)
            .max()
            .expect("system fonts must render Japanese text");
        assert!(object.hit_test(Point::new(rightmost as f32, 22.0), 0.0));
    }
}
