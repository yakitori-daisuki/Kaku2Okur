use i_slint_core::{
    input::{InternalKeyEvent, KeyEventType},
    window::WindowInner,
};
use kaku2okur::MainWindow;
use slint::{
    platform::{
        software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
        Platform, WindowAdapter,
    },
    ComponentHandle,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct TestPlatform(Rc<MinimalSoftwareWindow>);
impl Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn composition(window: &MainWindow, text: &str, commit: bool) {
    let mut event = InternalKeyEvent::default();
    if commit {
        event.event_type = KeyEventType::CommitComposition;
        event.key_event.text = text.into();
    } else {
        event.event_type = KeyEventType::UpdateComposition;
        event.preedit_text = text.into();
        event.preedit_selection = Some(0..text.len() as i32);
    }
    WindowInner::from_pub(window.window()).process_key_input(event);
    slint::platform::update_timers_and_animations();
}

fn enter(window: &MainWindow, repeat: bool) {
    let mut event = InternalKeyEvent::default();
    event.event_type = KeyEventType::KeyPressed;
    event.key_event.text = slint::platform::Key::Return.into();
    event.key_event.repeat = repeat;
    WindowInner::from_pub(window.window()).process_key_input(event);
    slint::platform::update_timers_and_animations();
}

#[test]
fn japanese_composition_and_editor_enter_never_send() {
    let software_window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(software_window.clone()))).unwrap();
    let window = MainWindow::new().unwrap();
    let sends = Rc::new(Cell::new(0));
    let entry_count = Rc::new(Cell::new(0));
    window.on_send_requested({
        let sends = sends.clone();
        move || sends.set(sends.get() + 1)
    });
    window.on_text_entry_requested({
        let weak = window.as_weak();
        let entry_count = entry_count.clone();
        move |x, y, _, _, text| {
            entry_count.set(entry_count.get() + 1);
            let window = weak.unwrap();
            window.set_text_editor_x(x);
            window.set_text_editor_y(y);
            window.set_text_editor_value(text);
            window.set_text_editor_visible(true);
        }
    });
    window.show().unwrap();
    window
        .window()
        .set_size(slint::LogicalSize::new(820.0, 520.0));
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::WindowActiveChanged(true));
    window.invoke_focus_canvas_input();
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(100.0, 100.0),
        });
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(300.0, 300.0),
        });
    // Closing the permission notice restores typing without a canvas click.
    window.set_invocation_permission_visible(true);
    slint::platform::update_timers_and_animations();
    window.set_invocation_permission_visible(false);
    slint::platform::update_timers_and_animations();

    // Composition can begin in the neutral canvas input before an editor exists.
    composition(&window, "n", false);
    assert!(
        window.get_text_editor_visible(),
        "show the popup at the first romaji preedit, before confirmation"
    );
    assert_eq!(window.get_text_editor_preedit(), "n");
    assert_eq!(window.get_text_editor_value(), "");
    composition(&window, "に", false);
    assert_eq!(window.get_text_editor_preedit(), "に");
    composition(&window, "にほんご", false);
    assert_eq!(window.get_text_editor_preedit(), "にほんご");
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(700.0, 400.0),
        });
    // Optional visual proof of the real popup rendering unconfirmed Japanese.
    if let Ok(path) = std::env::var("K2O_IME_SCREENSHOT") {
        let mut pixels = vec![slint::Rgb8Pixel::default(); 820 * 520];
        software_window.request_redraw();
        assert!(software_window.draw_if_needed(|renderer| {
            renderer.render(&mut pixels, 820);
        }));
        let mut image = tiny_skia::Pixmap::new(820, 520).unwrap();
        for (pixel, rgba) in pixels.iter().zip(image.data_mut().chunks_exact_mut(4)) {
            rgba.copy_from_slice(&[pixel.r, pixel.g, pixel.b, 255]);
        }
        image.save_png(path).unwrap();
    }
    enter(&window, false);
    assert_eq!(sends.get(), 0);
    assert_eq!(window.get_text_editor_value(), "");
    assert_eq!(window.get_text_editor_preedit(), "にほんご");
    composition(&window, "日本語", false);
    assert_eq!(window.get_text_editor_preedit(), "日本語");
    composition(&window, "日本語", true);
    assert!(window.get_text_editor_visible());
    assert_eq!(window.get_text_editor_preedit(), "");
    assert_eq!(window.get_text_editor_value(), "日本語");
    assert_eq!(
        entry_count.get(),
        1,
        "candidate changes must not restart text entry"
    );
    assert_eq!(window.get_text_editor_x(), 300.0);
    assert_eq!(window.get_text_editor_y(), 300.0);
    // Some IMEs deliver Return after committing and clearing their preedit.
    enter(&window, false);
    enter(&window, true);
    assert_eq!(sends.get(), 0);
    assert!(window.get_text_editor_visible());

    composition(&window, "へんかん", false);
    composition(&window, "変換", true);
    enter(&window, false);
    assert_eq!(sends.get(), 0);
    assert!(window.get_text_editor_value().contains("変換"));

    let accepted = Rc::new(RefCell::new(Vec::new()));
    window.on_text_accepted({
        let weak = window.as_weak();
        let accepted = accepted.clone();
        move |text| {
            accepted.borrow_mut().push(text);
            weak.unwrap().set_text_editor_visible(false);
        }
    });
    for (is_macos, modifier) in [
        // Slint's winit backend normalizes macOS Command to Key::Control.
        (true, slint::platform::Key::Control),
        (false, slint::platform::Key::Control),
    ] {
        window.set_is_macos(is_macos);
        window.set_text_editor_value("日本語の配置".into());
        window.set_text_editor_visible(true);
        slint::platform::update_timers_and_animations();
        window
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                text: modifier.into(),
            });
        enter(&window, false);
        assert!(!window.get_text_editor_visible());
        assert_eq!(accepted.borrow().last().unwrap(), "日本語の配置");
        assert_eq!(sends.get(), 0);
        // Holding or pressing the same shortcut again on the canvas never sends.
        enter(&window, true);
        enter(&window, false);
        assert_eq!(sends.get(), 0);
        window
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyReleased {
                text: modifier.into(),
            });
    }
    assert_eq!(accepted.borrow().len(), 2);

    // A new composition after placing text opens immediately, with no old text.
    composition(&window, "k", false);
    assert!(window.get_text_editor_visible());
    assert_eq!(window.get_text_editor_value(), "");
    assert_eq!(window.get_text_editor_preedit(), "k");
    composition(&window, "か", false);
    composition(&window, "", false); // IME cancellation leaves an editable empty popup.
    assert_eq!(window.get_text_editor_preedit(), "");
    assert_eq!(window.get_text_editor_value(), "");
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: "a".into() });
    assert_eq!(window.get_text_editor_value(), "a");
    assert_eq!(entry_count.get(), 2);

    // An explicit return to the canvas restores Enter-to-Send, except repeats.
    window.set_text_editor_visible(false);
    slint::platform::update_timers_and_animations();
    // The hidden persistent input must never intercept drawing at the pointer.
    let canvas_presses = Rc::new(Cell::new(0));
    window.on_pointer_pressed({
        let canvas_presses = canvas_presses.clone();
        move |_, _, _, _| canvas_presses.set(canvas_presses.get() + 1)
    });
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerMoved {
            position: slint::LogicalPosition::new(650.0, 400.0),
        });
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerPressed {
            position: slint::LogicalPosition::new(650.0, 400.0),
            button: slint::platform::PointerEventButton::Left,
        });
    window
        .window()
        .dispatch_event(slint::platform::WindowEvent::PointerReleased {
            position: slint::LogicalPosition::new(650.0, 400.0),
            button: slint::platform::PointerEventButton::Left,
        });
    assert_eq!(canvas_presses.get(), 1);
    enter(&window, true);
    assert_eq!(sends.get(), 0);
    enter(&window, false);
    assert_eq!(sends.get(), 1);
}
