use crate::{
    platform::{self, ModifierKeyState},
    settings::InvocationGesture,
    MainWindow,
};
use rdev::{Event, EventType, Key};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

const DOUBLE_TAP_WINDOW: Duration = Duration::from_millis(360);

#[derive(Clone)]
pub struct InvocationControl {
    gesture: Arc<AtomicU8>,
    listener_active: Arc<AtomicBool>,
}

impl InvocationControl {
    pub fn new(gesture: InvocationGesture) -> Self {
        Self {
            gesture: Arc::new(AtomicU8::new(gesture_code(gesture))),
            listener_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn set_gesture(&self, gesture: InvocationGesture) {
        self.gesture.store(gesture_code(gesture), Ordering::Relaxed);
    }

    pub fn start(&self, window: slint::Weak<MainWindow>) -> bool {
        if self.listener_active.swap(true, Ordering::AcqRel) {
            return false;
        }
        let gesture = self.gesture.clone();
        let listener_active = self.listener_active.clone();
        let failure_window = window.clone();
        thread::Builder::new()
            .name("k2o-invocation-listener".into())
            .spawn(move || {
                let initial = platform::modifier_key_state().unwrap_or_default();
                let mut primary_left = initial.primary_left;
                let mut primary_right = initial.primary_right;
                let mut alternate_left = initial.alternate_left;
                let mut alternate_right = initial.alternate_right;
                let mut left_shift = initial.shift_left;
                let mut right_shift = initial.shift_right;
                let mut dual_primary_armed = true;
                let mut dual_alternate_armed = true;
                let mut chord_armed = true;
                let mut last_primary_tap = None;
                let mut last_alternate_tap = None;
                let mut last_published = initial;
                publish_modifier_state(&window, initial);

                let callback = move |event: Event, event_state: Option<ModifierKeyState>| {
                    let (primary_left_key, primary_right_key) = primary_modifier_keys();
                    let (alternate_left_key, alternate_right_key) = alternate_modifier_keys();
                    let mut primary_double_tapped = false;
                    let mut alternate_double_tapped = false;
                    let is_key_event = matches!(
                        event.event_type,
                        EventType::KeyPress(_) | EventType::KeyRelease(_)
                    );
                    match event.event_type {
                        EventType::KeyPress(key) if key == primary_left_key => {
                            if !primary_left {
                                primary_double_tapped =
                                    register_tap(&mut last_primary_tap, key, Instant::now());
                            }
                            primary_left = true;
                        }
                        EventType::KeyPress(key) if key == primary_right_key => {
                            if !primary_right {
                                primary_double_tapped =
                                    register_tap(&mut last_primary_tap, key, Instant::now());
                            }
                            primary_right = true;
                        }
                        EventType::KeyRelease(key) if key == primary_left_key => {
                            primary_left = false
                        }
                        EventType::KeyRelease(key) if key == primary_right_key => {
                            primary_right = false
                        }
                        EventType::KeyPress(key) if key == alternate_left_key => {
                            if !alternate_left {
                                alternate_double_tapped =
                                    register_tap(&mut last_alternate_tap, key, Instant::now());
                            }
                            alternate_left = true;
                        }
                        EventType::KeyPress(key) if key == alternate_right_key => {
                            if !alternate_right {
                                alternate_double_tapped =
                                    register_tap(&mut last_alternate_tap, key, Instant::now());
                            }
                            alternate_right = true;
                        }
                        EventType::KeyRelease(key) if key == alternate_left_key => {
                            alternate_left = false
                        }
                        EventType::KeyRelease(key) if key == alternate_right_key => {
                            alternate_right = false
                        }
                        EventType::KeyPress(Key::ShiftLeft) => left_shift = true,
                        EventType::KeyPress(Key::ShiftRight) => right_shift = true,
                        EventType::KeyRelease(Key::ShiftLeft) => left_shift = false,
                        EventType::KeyRelease(Key::ShiftRight) => right_shift = false,
                        EventType::KeyRelease(Key::Space) => chord_armed = true,
                        _ => {}
                    }

                    if is_key_event {
                        if let Some(current) = event_state {
                            primary_left = current.primary_left;
                            primary_right = current.primary_right;
                            alternate_left = current.alternate_left;
                            alternate_right = current.alternate_right;
                            left_shift = current.shift_left;
                            right_shift = current.shift_right;
                        }
                        let current = ModifierKeyState {
                            primary_left,
                            primary_right,
                            alternate_left,
                            alternate_right,
                            shift_left: left_shift,
                            shift_right: right_shift,
                        };
                        if current != last_published {
                            last_published = current;
                            publish_modifier_state(&window, current);
                        }
                    }

                    let dual_primary_triggered = is_key_event
                        && update_dual_modifier_latch(
                            primary_left,
                            primary_right,
                            &mut dual_primary_armed,
                        );
                    let dual_alternate_triggered = is_key_event
                        && update_dual_modifier_latch(
                            alternate_left,
                            alternate_right,
                            &mut dual_alternate_armed,
                        );
                    let active = gesture.load(Ordering::Relaxed);
                    let triggered = match active {
                        value if value == gesture_code(InvocationGesture::DualModifier) => {
                            dual_primary_triggered
                        }
                        value if value == gesture_code(InvocationGesture::DoubleModifier) => {
                            primary_double_tapped
                        }
                        value if value == gesture_code(InvocationGesture::DualAlternate) => {
                            dual_alternate_triggered
                        }
                        value if value == gesture_code(InvocationGesture::DoubleAlternate) => {
                            alternate_double_tapped
                        }
                        value if value == gesture_code(InvocationGesture::Conventional) => {
                            (primary_left || primary_right)
                                && (left_shift || right_shift)
                                && matches!(event.event_type, EventType::KeyPress(Key::Space))
                                && chord_armed
                        }
                        _ => false,
                    };

                    if triggered {
                        chord_armed = false;
                        let _ = window.upgrade_in_event_loop(|window| {
                            window.invoke_invocation_triggered();
                        });
                    }
                };

                if let Err(error) = listen_invocation_events(callback) {
                    listener_active.store(false, Ordering::Release);
                    eprintln!("global invocation listener unavailable: {error:?}");
                    let _ = failure_window.upgrade_in_event_loop(|window| {
                        window.invoke_invocation_listener_failed();
                    });
                }
            })
            .expect("failed to start invocation listener");
        true
    }
}

#[cfg(not(target_os = "macos"))]
fn listen_invocation_events(
    mut callback: impl FnMut(Event, Option<ModifierKeyState>) + 'static,
) -> Result<(), String> {
    rdev::listen(move |event| callback(event, platform::modifier_key_state()))
        .map_err(|error| format!("{error:?}"))
}

#[cfg(target_os = "macos")]
fn listen_invocation_events(
    mut callback: impl FnMut(Event, Option<ModifierKeyState>),
) -> Result<(), String> {
    platform::listen_invocation(move |mask, code, down| {
        let key = match code {
            55 => Key::MetaLeft,
            54 => Key::MetaRight,
            58 => Key::Alt,
            61 => Key::AltGr,
            56 => Key::ShiftLeft,
            60 => Key::ShiftRight,
            49 => Key::Space,
            _ => return,
        };
        callback(
            Event {
                event_type: if down {
                    EventType::KeyPress(key)
                } else {
                    EventType::KeyRelease(key)
                },
                time: std::time::SystemTime::now(),
                name: None,
            },
            Some(ModifierKeyState::from_mask(mask)),
        );
    })
}

fn publish_modifier_state(window: &slint::Weak<MainWindow>, state: ModifierKeyState) {
    let _ = window.upgrade_in_event_loop(move |window| {
        window.set_modifier_primary_left_down(state.primary_left);
        window.set_modifier_primary_right_down(state.primary_right);
        window.set_modifier_alternate_left_down(state.alternate_left);
        window.set_modifier_alternate_right_down(state.alternate_right);
    });
}

fn gesture_code(gesture: InvocationGesture) -> u8 {
    match gesture {
        InvocationGesture::DualModifier => 0,
        InvocationGesture::DoubleModifier => 1,
        InvocationGesture::DualAlternate => 2,
        InvocationGesture::DoubleAlternate => 3,
        InvocationGesture::Conventional => 4,
    }
}

#[cfg(target_os = "macos")]
fn primary_modifier_keys() -> (Key, Key) {
    (Key::MetaLeft, Key::MetaRight)
}

#[cfg(not(target_os = "macos"))]
fn primary_modifier_keys() -> (Key, Key) {
    (Key::ControlLeft, Key::ControlRight)
}

fn alternate_modifier_keys() -> (Key, Key) {
    (Key::Alt, Key::AltGr)
}

fn register_tap(last_tap: &mut Option<(Key, Instant)>, key: Key, now: Instant) -> bool {
    let double_tapped = last_tap.take().is_some_and(|(previous_key, previous)| {
        previous_key == key && now.duration_since(previous) <= DOUBLE_TAP_WINDOW
    });
    if !double_tapped {
        *last_tap = Some((key, now));
    }
    double_tapped
}

fn update_dual_modifier_latch(left: bool, right: bool, armed: &mut bool) -> bool {
    if !left && !right {
        *armed = true;
        return false;
    }
    if left && right && *armed {
        *armed = false;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_quick_taps_trigger_once() {
        let first = Instant::now();
        let mut previous = None;
        assert!(!register_tap(&mut previous, Key::MetaLeft, first));
        assert!(register_tap(
            &mut previous,
            Key::MetaLeft,
            first + Duration::from_millis(200)
        ));
        assert!(!register_tap(
            &mut previous,
            Key::MetaLeft,
            first + Duration::from_millis(300)
        ));
    }

    #[test]
    fn a_slow_second_tap_starts_a_new_pair() {
        let first = Instant::now();
        let mut previous = None;
        assert!(!register_tap(&mut previous, Key::MetaLeft, first));
        assert!(!register_tap(
            &mut previous,
            Key::MetaLeft,
            first + DOUBLE_TAP_WINDOW + Duration::from_millis(1)
        ));
    }

    #[test]
    fn opposite_sides_do_not_count_as_a_double_tap() {
        let first = Instant::now();
        let mut previous = None;
        assert!(!register_tap(&mut previous, Key::MetaLeft, first));
        assert!(!register_tap(
            &mut previous,
            Key::MetaRight,
            first + Duration::from_millis(100)
        ));
    }

    #[test]
    fn simultaneous_modifier_pair_triggers_once_until_both_are_released() {
        let mut armed = true;

        assert!(!update_dual_modifier_latch(true, false, &mut armed));
        assert!(update_dual_modifier_latch(true, true, &mut armed));
        assert!(!update_dual_modifier_latch(true, true, &mut armed));
        assert!(!update_dual_modifier_latch(false, true, &mut armed));
        assert!(!update_dual_modifier_latch(false, false, &mut armed));
        assert!(update_dual_modifier_latch(true, true, &mut armed));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_option_event_flags_support_both_press_and_release_orders() {
        extern "C" {
            fn k2o_modifier_mask_from_flags(flags: u64) -> u32;
        }
        // macOS device-specific left/right Option bits, plus aggregate Option.
        for sequence in [
            [0x80020, 0x80060, 0x80040, 0, 0x80040, 0x80060, 0x80020, 0],
            [0x80040, 0x80060, 0x80020, 0, 0x80020, 0x80060, 0x80040, 0],
        ] {
            let mut armed = true;
            let triggered: Vec<_> = sequence
                .into_iter()
                .map(|flags| {
                    let state =
                        ModifierKeyState::from_mask(unsafe { k2o_modifier_mask_from_flags(flags) });
                    assert!(!state.primary_left && !state.primary_right);
                    update_dual_modifier_latch(
                        state.alternate_left,
                        state.alternate_right,
                        &mut armed,
                    )
                })
                .collect();
            assert_eq!(
                triggered,
                [false, true, false, false, false, true, false, false]
            );
        }
    }
}
