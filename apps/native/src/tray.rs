use crate::MainWindow;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub struct AppTray {
    _icon: tray_icon::TrayIcon,
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub struct AppTray;

impl AppTray {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn new(window: slint::Weak<MainWindow>) -> Result<Self, String> {
        use tray_icon::{
            menu::{Menu, MenuEvent, MenuItem},
            MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
        };

        let menu = Menu::new();
        let new_sketch = MenuItem::with_id("new-sketch", "New Sketch", true, None);
        let about = MenuItem::with_id("about", "About Kaku2Okur", true, None);
        let quit = MenuItem::with_id("quit", "Quit Kaku2Okur", true, None);
        let _ = menu.append_items(&[&new_sketch, &about, &quit]);

        let tray_window = window.clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let weak = tray_window.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(window) = weak.upgrade() {
                        window.invoke_open_requested();
                    }
                });
            }
        }));

        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let id = event.id().0.as_str();
            match id {
                "new-sketch" => {
                    let weak = window.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = weak.upgrade() {
                            window.invoke_open_requested();
                        }
                    });
                }
                "about" => {
                    let weak = window.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = weak.upgrade() {
                            window.invoke_about_requested();
                        }
                    });
                }
                "quit" => {
                    let _ = slint::invoke_from_event_loop(|| {
                        let _ = slint::quit_event_loop();
                    });
                }
                _ => {}
            }
        }));

        let icon =
            tray_icon::Icon::from_rgba(tray_pixels(), 32, 32).map_err(|error| error.to_string())?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Kaku2Okur")
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_icon(icon)
            .with_icon_as_template(cfg!(target_os = "macos"))
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self { _icon: tray })
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub fn new(_window: slint::Weak<MainWindow>) -> Result<Self, std::convert::Infallible> {
        Ok(Self)
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn tray_pixels() -> Vec<u8> {
    use tiny_skia::{
        BlendMode, FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform,
    };

    let mut pixmap = Pixmap::new(32, 32).expect("valid tray dimensions");
    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);
    let stroke = Stroke {
        width: 1.6,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Stroke::default()
    };
    let mut outline = PathBuilder::new();
    outline.move_to(7.0, 11.0);
    outline.line_to(3.0, 14.0);
    outline.line_to(3.0, 26.0);
    outline.quad_to(3.0, 28.0, 5.0, 28.0);
    outline.line_to(26.0, 28.0);
    outline.quad_to(28.0, 28.0, 28.0, 26.0);
    outline.line_to(28.0, 14.0);
    outline.line_to(24.0, 11.0);
    outline.move_to(12.0, 6.0);
    outline.line_to(15.5, 3.0);
    outline.line_to(19.0, 6.0);
    outline.move_to(7.0, 16.5);
    outline.line_to(7.0, 6.0);
    outline.line_to(24.0, 6.0);
    outline.line_to(24.0, 16.5);
    outline.move_to(3.0, 14.0);
    outline.line_to(12.0, 21.0);
    outline.move_to(28.0, 14.0);
    outline.line_to(20.0, 20.0);
    outline.move_to(4.0, 27.0);
    outline.line_to(12.0, 20.0);
    outline.quad_to(13.0, 19.0, 14.0, 19.0);
    outline.line_to(17.0, 19.0);
    outline.line_to(26.0, 27.0);
    outline.move_to(10.0, 14.5);
    outline.quad_to(13.0, 10.0, 15.0, 13.0);
    outline.quad_to(17.0, 15.0, 19.0, 15.0);
    pixmap.stroke_path(
        &outline.finish().unwrap(),
        &paint,
        &stroke,
        Transform::identity(),
        None,
    );

    // Clear the envelope behind the pencil to keep the small template legible.
    let mut pencil = PathBuilder::new();
    pencil.move_to(19.5, 16.5);
    pencil.line_to(23.0, 18.0);
    pencil.line_to(30.0, 25.0);
    pencil.quad_to(30.5, 25.5, 30.0, 26.0);
    pencil.line_to(28.5, 27.5);
    pencil.quad_to(28.0, 28.0, 27.5, 27.5);
    pencil.line_to(20.5, 20.5);
    pencil.close();
    let pencil = pencil.finish().unwrap();
    let mut clear = paint.clone();
    clear.blend_mode = BlendMode::Clear;
    pixmap.stroke_path(
        &pencil,
        &clear,
        &Stroke {
            width: 2.8,
            ..stroke
        },
        Transform::identity(),
        None,
    );
    pixmap.fill_path(
        &pencil,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
    pixmap.data().to_vec()
}

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests {
    #[test]
    fn tray_template_is_transparent_monochrome_and_contains_envelope_and_pencil() {
        let pixels = super::tray_pixels();
        assert_eq!(pixels.len(), 32 * 32 * 4);
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[..3] == [0, 0, 0]));
        let alpha = |x: usize, y: usize| pixels[(y * 32 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0);
        assert_eq!(alpha(31, 31), 0);
        assert!(alpha(3, 23) > 0);
        assert!(alpha(16, 28) > 0);
        assert!(alpha(28, 25) > 0);
    }
}
