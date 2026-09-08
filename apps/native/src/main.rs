#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use kaku2okur::{controller::AppController, MainWindow};
use slint::ComponentHandle;
use std::time::Duration;

fn main() -> Result<(), slint::PlatformError> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let window = MainWindow::new()?;
    let _controller = AppController::attach(&window);
    if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--verify-toggle")
    {
        verify_toggle(&window, arguments.get(index + 1).cloned());
    } else if let Some(index) = arguments
        .iter()
        .position(|argument| argument == "--verify-permission-recheck")
    {
        verify_permission_recheck(&window, arguments.get(index + 1).cloned());
    } else if arguments
        .iter()
        .any(|argument| argument == "--measure-cycle")
    {
        window.invoke_open_requested();
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_secs(1), move || {
            if let Some(window) = weak.upgrade() {
                window.invoke_escape_requested();
            }
        });
    } else if arguments.iter().any(|argument| argument == "--show-about") {
        window.invoke_about_requested();
    } else if arguments
        .iter()
        .any(|argument| argument == "--show-settings")
    {
        window.invoke_open_requested();
        window.set_settings_visible(true);
    } else if arguments.iter().any(|argument| argument == "--show") {
        window.invoke_open_requested();
    }
    slint::run_event_loop_until_quit()
}

fn verify_permission_recheck(window: &MainWindow, result_path: Option<String>) {
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_millis(500), move || {
        let window = weak.upgrade().expect("verification window disappeared");
        window.invoke_open_requested();
        window.set_invocation_permission_visible(true);
        window.invoke_invocation_permission_requested();
        assert!(window.get_invocation_permission_checking());
        window.invoke_invocation_permission_requested();
        assert!(window.get_invocation_permission_checking());
        let weak = window.as_weak();
        slint::Timer::single_shot(Duration::from_millis(500), move || {
            let window = weak.upgrade().expect("verification window disappeared");
            assert!(!window.get_invocation_permission_checking());
            assert!(
                !window.get_invocation_permission_visible()
                    || !window.get_invocation_permission_result().is_empty(),
                "recheck must close the dialog or explain the failure"
            );
            window.set_invocation_permission_visible(true);
            window.invoke_invocation_permission_requested();
            window.invoke_invocation_permission_dismissed();
            let weak = window.as_weak();
            slint::Timer::single_shot(Duration::from_millis(300), move || {
                let window = weak.upgrade().expect("verification window disappeared");
                assert!(!window.get_invocation_permission_checking());
                assert!(!window.get_invocation_permission_visible());
                let result =
                    "PASS: permission recheck busy state, result, duplicate click, and dismissal\n";
                if let Some(path) = result_path {
                    std::fs::write(path, result).expect("could not write verification result");
                }
                println!("{result}");
                slint::quit_event_loop().expect("verification event loop did not quit");
            });
        });
    });
}

fn verify_toggle(window: &MainWindow, result_path: Option<String>) {
    // Exercise the real UI callbacks and delayed placement without synthesizing OS keys.
    for step in 0..5 {
        let weak = window.as_weak();
        let result_path = result_path.clone();
        slint::Timer::single_shot(Duration::from_millis(300 + step * 250), move || {
            let window = weak.upgrade().expect("verification window disappeared");
            match step {
                0 => {
                    window.invoke_invocation_permission_dismissed();
                    window.invoke_invocation_triggered();
                }
                1 => {
                    assert!(window.window().is_visible());
                    window.invoke_text_entry_requested(
                        100.0,
                        100.0,
                        820.0,
                        520.0,
                        "Toggle check".into(),
                    );
                    window.invoke_invocation_triggered();
                }
                2 => {
                    assert!(!window.window().is_visible());
                    assert!(window.get_text_editor_visible());
                    assert_eq!(window.get_text_editor_value(), "Toggle check");
                    window.invoke_invocation_triggered();
                }
                3 => {
                    assert!(window.window().is_visible());
                    assert!(window.get_text_editor_visible());
                    assert_eq!(window.get_text_editor_value(), "Toggle check");
                    window.invoke_invocation_triggered();
                }
                _ => {
                    assert!(!window.window().is_visible());
                    println!("PASS: show/hide/show/hide and pending text preserved");
                    if let Some(path) = result_path {
                        std::fs::write(
                            path,
                            "PASS: show/hide/show/hide and pending text preserved\n",
                        )
                        .expect("could not write verification result");
                    }
                    slint::quit_event_loop().expect("verification event loop did not quit");
                }
            }
        });
    }
}
