use crate::{
    delivery::{self, DeliveryOutcome},
    invocation::InvocationControl,
    model::{Point, PointerOutcome, SketchDocument, Tool},
    platform::{self, DeliveryTarget},
    render::{CanvasRenderOptions, SceneRenderer},
    settings::{
        parse_hex_color, BackgroundPreference, InvocationGesture, SendBackgroundPolicy, Settings,
    },
    trackpad::{HotspotAction, HotspotTrigger, TrackpadHotspot},
    tray, MainWindow,
};
use parking_lot::Mutex;
use slint::{CloseRequestResponse, Color, ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};
use std::{
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

const INITIAL_CANVAS_SIZE: (f32, f32) = (820.0, 520.0);
const DISPLAY_PIXEL_SCALE: f32 = 1.0;
const HOTSPOT_LONG_PRESS_DELAY: Duration = Duration::from_millis(550);
const HOTSPOT_CLICK_GRACE: Duration = Duration::from_millis(450);
const SHAPE_HOLD_DELAY: Duration = Duration::from_millis(650);
const SHAPE_HOLD_POLL_INTERVAL: Duration = Duration::from_millis(40);
const SHAPE_HOLD_MOVEMENT_TOLERANCE: f32 = 2.0;
const WINDOW_FRAME_POLL_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug)]
struct PenHoldState {
    anchor: Point,
    still_since: Instant,
    resolved: bool,
    snapped: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum TrackpadInteraction {
    #[default]
    None,
    Drawing,
    Armed {
        hotspot: TrackpadHotspot,
        trigger: HotspotTrigger,
        generation: u64,
    },
    Consumed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvocationAccessState {
    Ready,
    PermissionRequired,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InvocationToggleAction {
    StartNew,
    ShowExisting,
    HideExisting,
}

impl InvocationAccessState {
    const fn id(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::PermissionRequired => "permission-required",
            Self::Unavailable => "unavailable",
        }
    }
}

pub struct AppController {
    _state: Arc<Mutex<AppRuntime>>,
    _tray: Option<tray::AppTray>,
    _shape_hold_timer: slint::Timer,
    _window_frame_timer: slint::Timer,
    _invocation_access_timer: slint::Timer,
}

struct AppRuntime {
    document: SketchDocument,
    renderer: SceneRenderer,
    settings: Settings,
    invocation: InvocationControl,
    invocation_access: InvocationAccessState,
    tool: Tool,
    canvas_size: (f32, f32),
    delivery_target: DeliveryTarget,
    system_dark: bool,
    session_active: bool,
    session_visible: bool,
    pending_text_origin: Option<Point>,
    trackpad_available: bool,
    trackpad_mode: bool,
    trackpad_installed: bool,
    suggestion_shown: bool,
    selected_hotspot: usize,
    trackpad_interaction: TrackpadInteraction,
    trackpad_generation: u64,
    pen_hold: Option<PenHoldState>,
}

struct TrackpadContext {
    state: Arc<Mutex<AppRuntime>>,
    window: slint::Weak<MainWindow>,
}

static TRACKPAD_CONTEXT: OnceLock<TrackpadContext> = OnceLock::new();

impl AppController {
    pub fn attach(window: &MainWindow) -> Self {
        platform::initialize_application();
        let settings = Settings::load();
        let invocation = InvocationControl::new(settings.invocation_gesture);
        let invocation_access = if platform::invocation_access_granted() {
            InvocationAccessState::Ready
        } else {
            InvocationAccessState::PermissionRequired
        };
        let trackpad_available = cfg!(target_os = "macos");
        let runtime = AppRuntime {
            document: SketchDocument::default(),
            renderer: SceneRenderer::new(),
            tool: settings.last_tool,
            canvas_size: INITIAL_CANVAS_SIZE,
            delivery_target: DeliveryTarget::default(),
            system_dark: platform::system_dark_mode(),
            session_active: false,
            session_visible: false,
            pending_text_origin: None,
            trackpad_available,
            trackpad_mode: settings.trackpad_mode_default,
            trackpad_installed: false,
            suggestion_shown: false,
            selected_hotspot: 0,
            trackpad_interaction: TrackpadInteraction::None,
            trackpad_generation: 0,
            pen_hold: None,
            settings,
            invocation: invocation.clone(),
            invocation_access,
        };
        let state = Arc::new(Mutex::new(runtime));
        let weak = window.as_weak();
        let _ = TRACKPAD_CONTEXT.set(TrackpadContext {
            state: state.clone(),
            window: weak.clone(),
        });

        bind_callbacks(window, state.clone());
        let shape_hold_timer = slint::Timer::default();
        {
            let state = state.clone();
            let weak = weak.clone();
            shape_hold_timer.start(
                slint::TimerMode::Repeated,
                SHAPE_HOLD_POLL_INTERVAL,
                move || poll_shape_hold(&state, &weak),
            );
        }
        let window_frame_timer = slint::Timer::default();
        {
            let state = state.clone();
            let weak = weak.clone();
            window_frame_timer.start(
                slint::TimerMode::Repeated,
                WINDOW_FRAME_POLL_INTERVAL,
                move || {
                    with_window(&weak, |window| {
                        let mut runtime = state.lock();
                        if runtime.session_active && runtime.session_visible {
                            remember_window_frame(&mut runtime, window);
                        }
                    });
                },
            );
        }
        {
            let runtime = state.lock();
            sync_ui(&runtime, window);
        }
        if invocation_access == InvocationAccessState::Ready {
            invocation.start(weak.clone());
        } else {
            let state_for_permission = state.clone();
            let weak_for_permission = weak.clone();
            slint::Timer::single_shot(Duration::from_millis(120), move || {
                with_window(&weak_for_permission, |window| {
                    attempt_invocation_access(&state_for_permission, window, true)
                });
            });
        }
        let invocation_access_timer = slint::Timer::default();
        {
            let state = state.clone();
            let weak = weak.clone();
            invocation_access_timer.start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    let waiting =
                        state.lock().invocation_access == InvocationAccessState::PermissionRequired;
                    if waiting && platform::invocation_access_granted() {
                        with_window(&weak, |window| {
                            attempt_invocation_access(&state, window, false)
                        });
                    }
                },
            );
        }
        let app_tray = tray::AppTray::new(weak).ok();

        Self {
            _state: state,
            _tray: app_tray,
            _shape_hold_timer: shape_hold_timer,
            _window_frame_timer: window_frame_timer,
            _invocation_access_timer: invocation_access_timer,
        }
    }
}

fn bind_callbacks(window: &MainWindow, state: Arc<Mutex<AppRuntime>>) {
    let weak = window.as_weak();
    window.on_open_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| open_session(&state, window))
    });
    window.on_invocation_triggered({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| toggle_session_visibility(&state, window))
    });
    window.on_about_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| open_about(&state, window))
    });
    window.on_about_closed({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                window.set_about_visible(false);
                let should_hide = {
                    let runtime = state.lock();
                    !runtime.session_active || !runtime.session_visible
                };
                if should_hide {
                    platform::hide_window(window.window());
                }
            })
        }
    });

    window.on_tool_selected({
        let state = state.clone();
        let weak = weak.clone();
        move |id| {
            let Some(tool) = Tool::from_id(id.as_str()) else {
                return;
            };
            with_window(&weak, |window| {
                if window.get_text_editor_visible() {
                    commit_pending_text(&state, window);
                }
                let mut runtime = state.lock();
                runtime.pen_hold = None;
                runtime.tool = tool;
                runtime.settings.last_tool = tool;
                let _ = runtime.settings.save();
                window.set_active_tool(tool.id().into());
            });
        }
    });

    window.on_pointer_hovered({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y| {
            with_window(&weak, |window| {
                let runtime = state.lock();
                window.set_text_draggable(
                    runtime
                        .document
                        .text_draggable_at(runtime.tool, Point::new(x, y)),
                );
            })
        }
    });

    window.on_pointer_pressed({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y, width, height| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                update_canvas_size(&mut runtime, width, height);
                let tool = runtime
                    .document
                    .pointer_tool(runtime.tool, Point::new(x, y));
                let color = effective_stroke_color(&runtime);
                let stroke_width = runtime.settings.stroke_width;
                let point = Point::new(x, y);
                let outcome = runtime
                    .document
                    .pointer_down(tool, point, color, stroke_width);
                begin_pen_hold(&mut runtime, tool, point);
                if let PointerOutcome::BeginText(origin) = outcome {
                    runtime.pen_hold = None;
                    runtime.pending_text_origin = Some(origin);
                    window.set_text_editor_x(origin.x);
                    window.set_text_editor_y(origin.y);
                    window.set_text_editor_value("".into());
                    window.set_text_editor_cursor_offset(0);
                    window.set_text_editor_visible(true);
                }
                sync_history_ui(&runtime, window);
                redraw(&runtime, window);
            });
        }
    });

    window.on_pointer_moved({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y, width, height| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                update_canvas_size(&mut runtime, width, height);
                move_active_pointer(&mut runtime, Point::new(x, y));
                redraw(&runtime, window);
            });
        }
    });

    window.on_pointer_released({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y, width, height| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                update_canvas_size(&mut runtime, width, height);
                finish_active_pointer(&mut runtime, Point::new(x, y));
                sync_history_ui(&runtime, window);
                redraw(&runtime, window);
            });
        }
    });

    window.on_text_entry_requested({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y, width, height, text| {
            with_window(&weak, |window| {
                begin_text_entry(
                    &state,
                    window,
                    Point::new(x, y),
                    width,
                    height,
                    text.as_str(),
                );
            });
        }
    });

    window.on_undo_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || mutate_document(&state, &weak, SketchDocument::undo)
    });
    window.on_redo_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || mutate_document(&state, &weak, SketchDocument::redo)
    });
    window.on_delete_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || mutate_document(&state, &weak, SketchDocument::delete_selection)
    });

    window.on_send_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| send_session(&state, window))
    });
    window.on_dismiss_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| hide_session(&state, window, true))
    });
    window.on_escape_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || with_window(&weak, |window| escape_session(&state, window))
    });
    window.on_window_drag_requested({
        let weak = weak.clone();
        move |x, y| {
            with_window(&weak, |window| {
                platform::request_window_drag(window.window(), x, y)
            })
        }
    });
    window.on_window_drag_moved({
        let weak = weak.clone();
        move |x, y| {
            with_window(&weak, |window| {
                platform::update_window_drag(window.window(), x, y)
            })
        }
    });
    window.on_window_drag_ended({
        let state = state.clone();
        let weak = weak.clone();
        move |x, y| {
            with_window(&weak, |window| {
                platform::end_window_drag(window.window(), x, y);
                let mut runtime = state.lock();
                remember_window_frame(&mut runtime, window);
            })
        }
    });

    window.on_text_accepted({
        let state = state.clone();
        let weak = weak.clone();
        move |text| {
            with_window(&weak, |window| {
                commit_text_value(&state, window, text.as_str());
            });
        }
    });
    window.on_text_cancelled({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                state.lock().pending_text_origin = None;
                window.set_text_editor_visible(false);
                window.set_text_editor_value("".into());
            });
        }
    });

    bind_setting_callbacks(window, state.clone(), weak.clone());

    window.window().on_close_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| hide_session(&state, window, true));
            CloseRequestResponse::HideWindow
        }
    });
}

fn bind_setting_callbacks(
    window: &MainWindow,
    state: Arc<Mutex<AppRuntime>>,
    weak: slint::Weak<MainWindow>,
) {
    window.on_stroke_color_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            if parse_hex_color(value.as_str()).is_none() {
                return;
            }
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.stroke_color = value.to_string();
                runtime.settings.stroke_color_automatic = false;
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_stroke_color_auto_selected({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.stroke_color_automatic = true;
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_stroke_width_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.stroke_width = value.clamp(1.0, 14.0);
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_auto_shape_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |enabled| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.auto_shape_enabled = enabled;
                if !enabled {
                    runtime.pen_hold = None;
                }
                let _ = runtime.settings.save();
                window.set_auto_shape_enabled(enabled);
            });
        }
    });
    window.on_background_mode_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            let Some(mode) = BackgroundPreference::from_id(value.as_str()) else {
                return;
            };
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.canvas_background = mode;
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_custom_canvas_color_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            if parse_hex_color(value.as_str()).is_none() {
                return;
            }
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.custom_canvas_color = value.to_string();
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_send_background_mode_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            let Some(mode) = SendBackgroundPolicy::from_id(value.as_str()) else {
                return;
            };
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.send_background = mode;
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_custom_send_color_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            if parse_hex_color(value.as_str()).is_none() {
                return;
            }
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.custom_send_color = value.to_string();
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_invocation_mode_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |value| {
            let Some(mode) = InvocationGesture::from_id(value.as_str()) else {
                return;
            };
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.settings.invocation_gesture = mode;
                runtime.settings.invocation_gesture_configured = true;
                runtime.invocation.set_gesture(mode);
                save_sync_redraw(&mut runtime, window);
            });
        }
    });
    window.on_invocation_permission_requested({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                if window.get_invocation_permission_checking() {
                    return;
                }
                window.set_invocation_permission_checking(true);
                window.set_invocation_permission_result("".into());
                let state = state.clone();
                let weak = window.as_weak();
                // Paint the busy state before calling the OS permission check.
                slint::Timer::single_shot(Duration::from_millis(150), move || {
                    with_window(&weak, |window| {
                        if window.get_invocation_permission_visible() {
                            attempt_invocation_access(&state, window, false);
                            if state.lock().invocation_access == InvocationAccessState::PermissionRequired {
                                window.set_invocation_permission_result(
                                    if cfg!(target_os = "macos") {
                                        "Not granted to this app. Enable Kaku2Okur in Input Monitoring. If already enabled, quit and reopen this copy of the app."
                                    } else {
                                        "Keyboard monitoring is still unavailable. Check your system permissions and try again."
                                    }.into(),
                                );
                            }
                        }
                        window.set_invocation_permission_checking(false);
                    });
                });
            })
        }
    });
    window.on_invocation_settings_requested({
        let weak = weak.clone();
        move || {
            if !platform::open_invocation_access_settings() {
                with_window(&weak, |window| {
                    window.set_invocation_permission_result(
                        "Could not open system settings. Open Privacy & Security > Input Monitoring manually.".into(),
                    );
                });
            }
        }
    });
    window.on_invocation_permission_dismissed({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                window.set_invocation_permission_visible(false);
                let should_hide = {
                    let runtime = state.lock();
                    !runtime.session_active || !runtime.session_visible
                };
                if should_hide {
                    platform::hide_window(window.window());
                }
            })
        }
    });
    window.on_invocation_listener_failed({
        let state = state.clone();
        let weak = weak.clone();
        move || {
            with_window(&weak, |window| {
                let access = if platform::invocation_access_granted() {
                    InvocationAccessState::Unavailable
                } else {
                    InvocationAccessState::PermissionRequired
                };
                state.lock().invocation_access = access;
                show_invocation_permission(&state, window);
                window.set_invocation_permission_result(
                    "Keyboard listener could not start. Check permission for this copy of Kaku2Okur, then retry or quit and reopen the app.".into(),
                );
            })
        }
    });
    window.on_trackpad_mode_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |active| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.trackpad_mode = active;
                runtime.settings.trackpad_mode_default = active;
                if !active {
                    runtime.trackpad_interaction = TrackpadInteraction::None;
                    runtime.pen_hold = None;
                    runtime.document.cancel_active();
                }
                let _ = runtime.settings.save();
                window.set_trackpad_mode(active);
                redraw(&runtime, window);
            });
        }
    });
    window.on_hotspot_selected({
        let state = state.clone();
        let weak = weak.clone();
        move |index| {
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                runtime.selected_hotspot = (index as usize).min(7);
                sync_hotspot_ui(&runtime, window);
            });
        }
    });
    window.on_hotspot_binding_changed({
        let state = state.clone();
        let weak = weak.clone();
        move |index, trigger, action| {
            let Some(trigger) = HotspotTrigger::from_id(trigger.as_str()) else {
                return;
            };
            let Some(action) = HotspotAction::from_id(action.as_str()) else {
                return;
            };
            with_window(&weak, |window| {
                let mut runtime = state.lock();
                let index = (index as usize).min(7);
                runtime.selected_hotspot = index;
                runtime.settings.trackpad_hotspots[index] =
                    crate::trackpad::HotspotBinding { trigger, action };
                let _ = runtime.settings.save();
                sync_hotspot_ui(&runtime, window);
            });
        }
    });
}

fn attempt_invocation_access(
    state: &Arc<Mutex<AppRuntime>>,
    window: &MainWindow,
    request_if_needed: bool,
) {
    let granted = platform::invocation_access_granted()
        || (request_if_needed && platform::request_invocation_access());
    if !granted {
        state.lock().invocation_access = InvocationAccessState::PermissionRequired;
        show_invocation_permission(state, window);
        return;
    }

    let (invocation, hide_after_start) = {
        let mut runtime = state.lock();
        runtime.invocation_access = InvocationAccessState::Ready;
        (
            runtime.invocation.clone(),
            window.get_invocation_permission_visible() && !runtime.session_visible,
        )
    };
    window.set_invocation_access_state(InvocationAccessState::Ready.id().into());
    window.set_invocation_permission_result("".into());
    window.set_invocation_permission_visible(false);
    invocation.start(window.as_weak());
    if hide_after_start {
        platform::hide_window(window.window());
    }
}

fn show_invocation_permission(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    let access = state.lock().invocation_access;
    window.set_invocation_access_state(access.id().into());
    if window.get_invocation_permission_visible() {
        return;
    }
    window.set_invocation_permission_result("".into());
    window.set_settings_visible(false);
    window.set_about_visible(false);
    window.set_invocation_permission_visible(true);
    platform::show_window(window.window());

    let state_for_placement = state.clone();
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_millis(30), move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        if !state_for_placement.lock().session_active {
            platform::configure_and_place_window(window.window(), None);
        }
        platform::show_window(window.window());
    });
}

fn open_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    {
        let mut runtime = state.lock();
        if !runtime.session_active {
            runtime.delivery_target = platform::capture_delivery_target();
            runtime.system_dark = platform::system_dark_mode();
            runtime.session_active = true;
        }
        runtime.session_visible = true;
        window.set_session_active(true);
        window.set_about_visible(false);
        window.set_invocation_permission_visible(false);
        sync_ui(&runtime, window);
    }

    platform::show_window(window.window());
    let state_for_open = state.clone();
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_millis(30), move || {
        if let Some(window) = weak.upgrade() {
            finish_open_session(&state_for_open, &window);
        }
    });
}

fn open_about(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    window.set_about_visible(true);
    window.set_settings_visible(false);
    window.set_invocation_permission_visible(false);
    platform::show_window(window.window());

    let weak = window.as_weak();
    let state_for_about = state.clone();
    slint::Timer::single_shot(Duration::from_millis(30), move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        if !state_for_about.lock().session_active {
            platform::configure_and_place_window(window.window(), None);
        }
        platform::show_window(window.window());
    });
}

fn finish_open_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    let runtime = state.lock();
    if !runtime.session_active || !runtime.session_visible {
        return;
    }
    drop(runtime);
    let pointer_display = platform::configure_and_place_window(window.window(), None);
    let saved = pointer_display.as_ref().and_then(|display_id| {
        state
            .lock()
            .settings
            .display_frames
            .get(display_id)
            .copied()
    });
    if let Some((width, height)) = platform::preferred_window_size(window.window(), saved) {
        window
            .window()
            .set_size(slint::LogicalSize::new(width, height));
        let mut runtime = state.lock();
        runtime.canvas_size = (width, height);
        redraw(&runtime, window);
    }

    let state_for_placement = state.clone();
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_millis(30), move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let runtime = state_for_placement.lock();
        if !runtime.session_active || !runtime.session_visible {
            return;
        }
        drop(runtime);
        platform::configure_and_place_window(window.window(), saved);
        platform::show_window(window.window());
        window.invoke_focus_canvas_input();
    });

    let mut runtime = state.lock();
    if runtime.trackpad_available && !runtime.trackpad_installed {
        runtime.trackpad_installed = platform::install_trackpad_capture(
            window.window(),
            handle_trackpad_touch,
            handle_native_dismiss,
            handle_native_finish_text,
        );
    }
    if runtime.trackpad_available && !runtime.trackpad_mode && !runtime.suggestion_shown {
        runtime.suggestion_shown = true;
        window.set_trackpad_suggestion_visible(true);
    }
}

fn toggle_session_visibility(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    let action = {
        let runtime = state.lock();
        invocation_toggle_action(runtime.session_active, runtime.session_visible)
    };
    match action {
        InvocationToggleAction::HideExisting => suspend_session(state, window),
        InvocationToggleAction::ShowExisting => {
            state.lock().delivery_target = platform::capture_delivery_target();
            open_session(state, window);
        }
        InvocationToggleAction::StartNew => open_session(state, window),
    }
}

fn invocation_toggle_action(active: bool, visible: bool) -> InvocationToggleAction {
    match (active, visible) {
        (true, true) => InvocationToggleAction::HideExisting,
        (true, false) => InvocationToggleAction::ShowExisting,
        (false, _) => InvocationToggleAction::StartNew,
    }
}

fn suspend_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    let target = {
        let mut runtime = state.lock();
        if !runtime.session_active || !runtime.session_visible {
            return;
        }
        remember_window_frame(&mut runtime, window);
        runtime.document.cancel_active();
        runtime.trackpad_interaction = TrackpadInteraction::None;
        runtime.pen_hold = None;
        runtime.session_visible = false;
        runtime.delivery_target
    };
    platform::hide_window(window.window());
    let _ = platform::restore_delivery_target(target);
}

fn hide_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow, discard: bool) {
    let mut runtime = state.lock();
    remember_window_frame(&mut runtime, window);
    if discard {
        runtime.document.clear();
        runtime.pending_text_origin = None;
    }
    runtime.session_active = false;
    runtime.session_visible = false;
    runtime.trackpad_interaction = TrackpadInteraction::None;
    runtime.pen_hold = None;
    window.set_session_active(false);
    window.set_settings_visible(false);
    window.set_about_visible(false);
    window.set_invocation_permission_visible(false);
    window.set_text_editor_visible(false);
    platform::hide_window(window.window());
    window.set_canvas_image(Image::default());
    sync_history_ui(&runtime, window);
}

fn send_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    if window.get_text_editor_visible() {
        commit_pending_text(state, window);
    }

    let (image, target) = {
        let mut runtime = state.lock();
        if runtime.document.is_empty() {
            show_toast(window, "Draw something first");
            return;
        }
        remember_window_frame(&mut runtime, window);
        let visible_background = runtime.settings.canvas_color(runtime.system_dark);
        let send_background = runtime.settings.send_color(visible_background);
        let Some(image) =
            runtime
                .renderer
                .render_send_result(send_background, runtime.document.objects(), 18.0)
        else {
            return;
        };
        runtime.session_active = false;
        runtime.session_visible = false;
        runtime.trackpad_interaction = TrackpadInteraction::None;
        runtime.pen_hold = None;
        (image, runtime.delivery_target)
    };

    window.set_session_active(false);
    window.set_settings_visible(false);
    window.set_about_visible(false);
    window.set_text_editor_visible(false);
    platform::hide_window(window.window());
    window.set_canvas_image(Image::default());

    let state_for_delivery = state.clone();
    let weak = window.as_weak();
    thread::Builder::new()
        .name("k2o-paste-dispatch".into())
        .spawn(move || {
            let outcome = delivery::deliver(image, target);
            if outcome == DeliveryOutcome::Dispatched {
                state_for_delivery.lock().document.clear();
            }
            let _ = weak.upgrade_in_event_loop(move |window| {
                let runtime = state_for_delivery.lock();
                sync_history_ui(&runtime, &window);
                if let DeliveryOutcome::ClipboardFallback(_) = outcome {
                    window.set_toast("Copied to clipboard".into());
                }
            });
        })
        .expect("failed to start paste dispatch");
}

fn escape_session(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    if window.get_text_editor_visible() {
        state.lock().pending_text_origin = None;
        window.set_text_editor_visible(false);
        window.set_text_editor_value("".into());
        return;
    }

    let mut runtime = state.lock();
    runtime.pen_hold = None;
    if runtime.document.cancel_active() || runtime.document.clear_selection() {
        sync_history_ui(&runtime, window);
        redraw(&runtime, window);
        return;
    }
    drop(runtime);
    hide_session(state, window, true);
}

fn commit_pending_text(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow) {
    let value = window.get_text_editor_value();
    commit_text_value(state, window, value.as_str());
}

fn commit_text_value(state: &Arc<Mutex<AppRuntime>>, window: &MainWindow, value: &str) {
    let mut runtime = state.lock();
    if let Some(origin) = runtime.pending_text_origin.take() {
        let color = effective_stroke_color(&runtime);
        let font_size = runtime.settings.font_size;
        runtime
            .document
            .commit_text(origin, value.to_owned(), color, font_size);
    }
    window.set_text_editor_visible(false);
    window.set_text_editor_value("".into());
    sync_history_ui(&runtime, window);
    redraw(&runtime, window);
}

fn begin_text_entry(
    state: &Arc<Mutex<AppRuntime>>,
    window: &MainWindow,
    point: Point,
    width: f32,
    height: f32,
    initial_text: &str,
) {
    // Composition starts before any committed text exists. The persistent
    // TextInput keeps the preedit; this callback only establishes its origin.
    if !initial_text.is_empty() && !is_printable_text_input(initial_text) {
        return;
    }

    let mut runtime = state.lock();
    update_canvas_size(&mut runtime, width, height);
    runtime.pen_hold = None;
    let origin = Point::new(
        point.x.clamp(0.0, runtime.canvas_size.0.max(0.0)),
        point.y.clamp(0.0, runtime.canvas_size.1.max(0.0)),
    );
    let color = effective_stroke_color(&runtime);
    let stroke_width = runtime.settings.stroke_width;
    let outcome = runtime
        .document
        .pointer_down(Tool::Text, origin, color, stroke_width);
    let PointerOutcome::BeginText(origin) = outcome else {
        return;
    };
    runtime.pending_text_origin = Some(origin);
    window.set_text_editor_x(origin.x);
    window.set_text_editor_y(origin.y);
    window.set_text_editor_value(initial_text.to_owned().into());
    window.set_text_editor_cursor_offset(initial_text.len() as i32);
    window.set_text_editor_visible(true);
    sync_history_ui(&runtime, window);
    redraw(&runtime, window);
}

fn is_printable_text_input(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|character| {
            !character.is_control()
                && !matches!(
                    character as u32,
                    0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD
                )
        })
}

fn mutate_document(
    state: &Arc<Mutex<AppRuntime>>,
    weak: &slint::Weak<MainWindow>,
    mutation: impl FnOnce(&mut SketchDocument) -> bool,
) {
    with_window(weak, |window| {
        let mut runtime = state.lock();
        runtime.pen_hold = None;
        if mutation(&mut runtime.document) {
            sync_history_ui(&runtime, window);
            redraw(&runtime, window);
        }
    });
}

fn save_sync_redraw(runtime: &mut AppRuntime, window: &MainWindow) {
    let _ = runtime.settings.save();
    sync_ui(runtime, window);
    redraw(runtime, window);
}

fn sync_ui(runtime: &AppRuntime, window: &MainWindow) {
    window.set_active_tool(runtime.tool.id().into());
    window.set_system_dark(runtime.system_dark);
    window.set_is_macos(cfg!(target_os = "macos"));
    let stroke_color = effective_stroke_color(runtime);
    window.set_stroke_color(color(stroke_color));
    window.set_stroke_color_hex(hex_color(stroke_color).into());
    window.set_stroke_color_automatic(runtime.settings.stroke_color_automatic);
    window.set_stroke_width(runtime.settings.stroke_width);
    window.set_auto_shape_enabled(runtime.settings.auto_shape_enabled);
    window.set_canvas_background_mode(runtime.settings.canvas_background.id().into());
    window.set_custom_canvas_color(runtime.settings.custom_canvas_color.clone().into());
    window.set_send_background_mode(runtime.settings.send_background.id().into());
    window.set_custom_send_color(runtime.settings.custom_send_color.clone().into());
    window.set_invocation_mode(runtime.settings.invocation_gesture.id().into());
    window.set_invocation_access_state(runtime.invocation_access.id().into());
    let modifier_state = platform::modifier_key_state().unwrap_or_default();
    window.set_modifier_primary_left_down(modifier_state.primary_left);
    window.set_modifier_primary_right_down(modifier_state.primary_right);
    window.set_modifier_alternate_left_down(modifier_state.alternate_left);
    window.set_modifier_alternate_right_down(modifier_state.alternate_right);
    window.set_trackpad_available(runtime.trackpad_available);
    window.set_trackpad_mode(runtime.trackpad_mode);
    window.set_session_active(runtime.session_active);
    sync_history_ui(runtime, window);
    sync_hotspot_ui(runtime, window);
}

fn sync_history_ui(runtime: &AppRuntime, window: &MainWindow) {
    window.set_can_undo(runtime.document.can_undo());
    window.set_can_redo(runtime.document.can_redo());
    window.set_has_selection(runtime.document.selected().is_some());
}

fn sync_hotspot_ui(runtime: &AppRuntime, window: &MainWindow) {
    let index = runtime.selected_hotspot.min(7);
    let hotspot = TrackpadHotspot::from_index(index).expect("hotspot index is clamped");
    let binding = runtime.settings.trackpad_hotspots[index];
    window.set_hotspot_selected_index(index as i32);
    window.set_hotspot_label(hotspot.label().into());
    window.set_hotspot_trigger(binding.trigger.id().into());
    window.set_hotspot_action(binding.action.id().into());
}

fn redraw(runtime: &AppRuntime, window: &MainWindow) {
    let width = runtime.canvas_size.0.round().clamp(1.0, 4096.0) as u32;
    let height = runtime.canvas_size.1.round().clamp(1.0, 4096.0) as u32;
    let background = runtime.settings.canvas_color(runtime.system_dark);
    let objects = runtime.document.display_objects();
    let (pixel_width, pixel_height) =
        SceneRenderer::canvas_pixel_dimensions(width, height, DISPLAY_PIXEL_SCALE);
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(pixel_width, pixel_height);
    runtime.renderer.render_canvas_into(
        buffer.make_mut_bytes(),
        pixel_width,
        pixel_height,
        CanvasRenderOptions {
            pixel_scale: DISPLAY_PIXEL_SCALE,
            background,
            objects: &objects,
            selection: runtime.document.selection_bounds(),
        },
    );
    window.set_canvas_image(Image::from_rgba8_premultiplied(buffer));
}

fn update_canvas_size(runtime: &mut AppRuntime, width: f32, height: f32) {
    if width >= 1.0 && height >= 1.0 {
        runtime.canvas_size = (width, height);
    }
}

fn begin_pen_hold(runtime: &mut AppRuntime, tool: Tool, point: Point) {
    runtime.pen_hold =
        (runtime.settings.auto_shape_enabled && tool == Tool::Pen).then(|| PenHoldState {
            anchor: point,
            still_since: Instant::now(),
            resolved: false,
            snapped: false,
        });
}

fn move_active_pointer(runtime: &mut AppRuntime, point: Point) {
    if runtime.pen_hold.is_some_and(|hold| hold.snapped) {
        return;
    }
    runtime.document.pointer_move(point);
    if let Some(hold) = &mut runtime.pen_hold {
        if hold.anchor.distance_to(point) >= SHAPE_HOLD_MOVEMENT_TOLERANCE {
            hold.anchor = point;
            hold.still_since = Instant::now();
            hold.resolved = false;
        }
    }
}

fn finish_active_pointer(runtime: &mut AppRuntime, point: Point) {
    let snapped = runtime.pen_hold.is_some_and(|hold| hold.snapped);
    if snapped {
        runtime.document.pointer_up_current();
    } else {
        runtime.document.pointer_up(point);
    }
    runtime.pen_hold = None;
}

fn poll_shape_hold(state: &Arc<Mutex<AppRuntime>>, weak: &slint::Weak<MainWindow>) {
    with_window(weak, |window| {
        let mut runtime = state.lock();
        let should_resolve = runtime.session_active
            && runtime.session_visible
            && runtime.settings.auto_shape_enabled
            && runtime.pen_hold.is_some_and(|hold| {
                !hold.resolved && hold.still_since.elapsed() >= SHAPE_HOLD_DELAY
            });
        if !should_resolve {
            return;
        }

        let snapped = runtime.document.snap_active_pen_to_shape();
        if let Some(hold) = &mut runtime.pen_hold {
            hold.resolved = true;
            hold.snapped = snapped;
        }
        if snapped {
            redraw(&runtime, window);
        }
    });
}

fn remember_window_frame(runtime: &mut AppRuntime, window: &MainWindow) {
    if let Some((display_id, frame)) = platform::current_relative_frame(window.window()) {
        if runtime
            .settings
            .display_frames
            .get(&display_id)
            .is_some_and(|saved| relative_frames_nearly_equal(*saved, frame))
        {
            return;
        }
        runtime.settings.display_frames.insert(display_id, frame);
        let _ = runtime.settings.save();
    }
}

fn relative_frames_nearly_equal(
    left: crate::settings::RelativeWindowFrame,
    right: crate::settings::RelativeWindowFrame,
) -> bool {
    const EPSILON: f32 = 0.0005;
    (left.x - right.x).abs() <= EPSILON
        && (left.y - right.y).abs() <= EPSILON
        && (left.width - right.width).abs() <= EPSILON
        && (left.height - right.height).abs() <= EPSILON
}

fn color(value: [u8; 4]) -> Color {
    Color::from_argb_u8(value[3], value[0], value[1], value[2])
}

fn effective_stroke_color(runtime: &AppRuntime) -> [u8; 4] {
    let canvas = runtime.settings.canvas_color(runtime.system_dark);
    runtime.settings.effective_stroke_color(canvas)
}

fn hex_color(value: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", value[0], value[1], value[2])
}

fn with_window(weak: &slint::Weak<MainWindow>, callback: impl FnOnce(&MainWindow)) {
    if let Some(window) = weak.upgrade() {
        callback(&window);
    }
}

fn show_toast(window: &MainWindow, message: &str) {
    window.set_toast(message.into());
    let weak = window.as_weak();
    slint::Timer::single_shot(Duration::from_millis(1500), move || {
        if let Some(window) = weak.upgrade() {
            window.set_toast("".into());
        }
    });
}

extern "C" fn handle_trackpad_touch(x: f64, y: f64, phase: i32, touch_count: usize) -> bool {
    let Some(context) = TRACKPAD_CONTEXT.get() else {
        return false;
    };
    let x = x.clamp(0.0, 1.0);
    let y = y.clamp(0.0, 1.0);
    let consume_mouse_down = if phase == 4 {
        let runtime = context.state.lock();
        runtime.trackpad_mode
            && runtime.session_active
            && runtime.session_visible
            && TrackpadHotspot::at(x, y).is_some_and(|hotspot| {
                runtime.settings.trackpad_hotspots[hotspot.index()].action != HotspotAction::None
            })
    } else {
        false
    };

    if touch_count != 1 {
        cancel_trackpad_interaction(context);
        return consume_mouse_down;
    }
    let state = context.state.clone();
    let weak = context.window.clone();
    let _ = weak.upgrade_in_event_loop(move |window| {
        let mut immediate_action = None;
        let mut schedule_long_press = None;
        let mut schedule_click_expiry = None;
        let mut runtime = state.lock();
        if !runtime.trackpad_mode || !runtime.session_active || !runtime.session_visible {
            return;
        }
        let point = Point::new(
            x as f32 * runtime.canvas_size.0,
            y as f32 * runtime.canvas_size.1,
        );
        match phase {
            0 => {
                runtime.pen_hold = None;
                runtime.trackpad_generation = runtime.trackpad_generation.wrapping_add(1);
                let generation = runtime.trackpad_generation;
                let hotspot = TrackpadHotspot::at(x, y);
                let binding = hotspot
                    .map(|hotspot| (hotspot, runtime.settings.trackpad_hotspots[hotspot.index()]));
                match binding.filter(|(_, binding)| binding.action != HotspotAction::None) {
                    Some((_, binding)) if binding.trigger == HotspotTrigger::Touch => {
                        runtime.trackpad_interaction = TrackpadInteraction::Consumed;
                        immediate_action = Some(binding.action);
                    }
                    Some((hotspot, binding)) => {
                        runtime.trackpad_interaction = TrackpadInteraction::Armed {
                            hotspot,
                            trigger: binding.trigger,
                            generation,
                        };
                        if binding.trigger == HotspotTrigger::LongPress {
                            schedule_long_press = Some((hotspot, generation));
                        }
                    }
                    None => begin_trackpad_drawing(&mut runtime, &window, point),
                }
            }
            1 => match runtime.trackpad_interaction {
                TrackpadInteraction::Drawing => move_active_pointer(&mut runtime, point),
                TrackpadInteraction::Armed { hotspot, .. } if !hotspot.contains(x, y) => {
                    runtime.trackpad_interaction = TrackpadInteraction::None;
                }
                TrackpadInteraction::None
                | TrackpadInteraction::Armed { .. }
                | TrackpadInteraction::Consumed => {}
            },
            2 => match runtime.trackpad_interaction {
                TrackpadInteraction::Drawing => {
                    finish_active_pointer(&mut runtime, point);
                    runtime.trackpad_interaction = TrackpadInteraction::None;
                }
                TrackpadInteraction::Armed {
                    trigger: HotspotTrigger::Click,
                    generation,
                    ..
                } => schedule_click_expiry = Some(generation),
                TrackpadInteraction::None
                | TrackpadInteraction::Armed { .. }
                | TrackpadInteraction::Consumed => {
                    runtime.trackpad_interaction = TrackpadInteraction::None;
                }
            },
            3 => {
                if runtime.trackpad_interaction == TrackpadInteraction::Drawing {
                    runtime.document.cancel_active();
                }
                runtime.pen_hold = None;
                runtime.trackpad_interaction = TrackpadInteraction::None;
            }
            4 => {
                let action = TrackpadHotspot::at(x, y).and_then(|hotspot| {
                    let binding = runtime.settings.trackpad_hotspots[hotspot.index()];
                    (binding.trigger == HotspotTrigger::Click
                        && binding.action != HotspotAction::None)
                        .then_some(binding.action)
                });
                if let Some(action) = action {
                    runtime.trackpad_interaction = TrackpadInteraction::Consumed;
                    immediate_action = Some(action);
                }
            }
            _ => return,
        }
        sync_history_ui(&runtime, &window);
        redraw(&runtime, &window);
        drop(runtime);

        if let Some((hotspot, generation)) = schedule_long_press {
            schedule_hotspot_long_press(state.clone(), window.as_weak(), hotspot, generation);
        }
        if let Some(generation) = schedule_click_expiry {
            schedule_hotspot_click_expiry(state.clone(), generation);
        }
        if let Some(action) = immediate_action {
            perform_hotspot_action(&state, &window, action);
        }
    });
    consume_mouse_down
}

extern "C" fn handle_native_finish_text() {
    let Some(context) = TRACKPAD_CONTEXT.get() else {
        return;
    };
    let _ = context.window.upgrade_in_event_loop(|window| {
        if window.get_text_editor_visible()
            && !window.get_settings_visible()
            && !window.get_about_visible()
            && !window.get_invocation_permission_visible()
        {
            window.invoke_text_accepted(window.get_text_editor_value());
        }
    });
}

extern "C" fn handle_native_dismiss() {
    let Some(context) = TRACKPAD_CONTEXT.get() else {
        return;
    };
    let state = context.state.clone();
    let weak = context.window.clone();
    let _ = weak.upgrade_in_event_loop(move |window| {
        hide_session(&state, &window, true);
    });
}

fn begin_trackpad_drawing(runtime: &mut AppRuntime, window: &MainWindow, point: Point) {
    let color = effective_stroke_color(runtime);
    let stroke_width = runtime.settings.stroke_width;
    let tool = runtime.document.pointer_tool(runtime.tool, point);
    let outcome = runtime
        .document
        .pointer_down(tool, point, color, stroke_width);
    begin_pen_hold(runtime, tool, point);
    if let PointerOutcome::BeginText(origin) = outcome {
        runtime.pen_hold = None;
        runtime.trackpad_interaction = TrackpadInteraction::Consumed;
        runtime.pending_text_origin = Some(origin);
        window.set_text_editor_x(origin.x);
        window.set_text_editor_y(origin.y);
        window.set_text_editor_value("".into());
        window.set_text_editor_cursor_offset(0);
        window.set_text_editor_visible(true);
    } else {
        runtime.trackpad_interaction = TrackpadInteraction::Drawing;
    }
}

fn schedule_hotspot_long_press(
    state: Arc<Mutex<AppRuntime>>,
    weak: slint::Weak<MainWindow>,
    hotspot: TrackpadHotspot,
    generation: u64,
) {
    slint::Timer::single_shot(HOTSPOT_LONG_PRESS_DELAY, move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        let action = {
            let mut runtime = state.lock();
            let still_armed = matches!(
                runtime.trackpad_interaction,
                TrackpadInteraction::Armed {
                    hotspot: armed_hotspot,
                    trigger: HotspotTrigger::LongPress,
                    generation: armed_generation,
                } if armed_hotspot == hotspot && armed_generation == generation
            );
            if !runtime.trackpad_mode
                || !runtime.session_active
                || !runtime.session_visible
                || !still_armed
            {
                None
            } else {
                let binding = runtime.settings.trackpad_hotspots[hotspot.index()];
                if binding.trigger == HotspotTrigger::LongPress
                    && binding.action != HotspotAction::None
                {
                    runtime.trackpad_interaction = TrackpadInteraction::Consumed;
                    Some(binding.action)
                } else {
                    runtime.trackpad_interaction = TrackpadInteraction::None;
                    None
                }
            }
        };
        if let Some(action) = action {
            perform_hotspot_action(&state, &window, action);
        }
    });
}

fn schedule_hotspot_click_expiry(state: Arc<Mutex<AppRuntime>>, generation: u64) {
    slint::Timer::single_shot(HOTSPOT_CLICK_GRACE, move || {
        let mut runtime = state.lock();
        if matches!(
            runtime.trackpad_interaction,
            TrackpadInteraction::Armed {
                trigger: HotspotTrigger::Click,
                generation: armed_generation,
                ..
            } if armed_generation == generation
        ) {
            runtime.trackpad_interaction = TrackpadInteraction::None;
        }
    });
}

fn perform_hotspot_action(
    state: &Arc<Mutex<AppRuntime>>,
    window: &MainWindow,
    action: HotspotAction,
) {
    if let Some(tool) = action.tool() {
        if window.get_text_editor_visible() {
            commit_pending_text(state, window);
        }
        let mut runtime = state.lock();
        runtime.pen_hold = None;
        runtime.tool = tool;
        runtime.settings.last_tool = tool;
        let _ = runtime.settings.save();
        window.set_active_tool(tool.id().into());
        return;
    }

    match action {
        HotspotAction::Undo | HotspotAction::Redo | HotspotAction::Delete => {
            let mut runtime = state.lock();
            runtime.pen_hold = None;
            let changed = match action {
                HotspotAction::Undo => runtime.document.undo(),
                HotspotAction::Redo => runtime.document.redo(),
                HotspotAction::Delete => runtime.document.delete_selection(),
                _ => unreachable!(),
            };
            if changed {
                sync_history_ui(&runtime, window);
                redraw(&runtime, window);
            }
        }
        HotspotAction::Send => send_session(state, window),
        HotspotAction::None
        | HotspotAction::Select
        | HotspotAction::Pen
        | HotspotAction::Eraser
        | HotspotAction::Arrow
        | HotspotAction::Rectangle
        | HotspotAction::Ellipse
        | HotspotAction::Text => {}
    }
}

fn cancel_trackpad_interaction(context: &TrackpadContext) {
    let state = context.state.clone();
    let weak = context.window.clone();
    let _ = weak.upgrade_in_event_loop(move |window| {
        let mut runtime = state.lock();
        if runtime.trackpad_interaction == TrackpadInteraction::Drawing {
            runtime.document.cancel_active();
        }
        runtime.pen_hold = None;
        runtime.trackpad_interaction = TrackpadInteraction::None;
        sync_history_ui(&runtime, &window);
        redraw(&runtime, &window);
    });
}

#[cfg(test)]
mod tests {
    use super::{
        invocation_toggle_action, is_printable_text_input, relative_frames_nearly_equal,
        InvocationToggleAction,
    };
    use crate::settings::RelativeWindowFrame;

    #[test]
    fn printable_text_can_start_an_inline_editor() {
        assert!(is_printable_text_input("A"));
        assert!(is_printable_text_input("日本語"));
        assert!(is_printable_text_input(" "));
    }

    #[test]
    fn control_and_slint_key_codes_do_not_start_an_inline_editor() {
        assert!(!is_printable_text_input(""));
        assert!(!is_printable_text_input("\n"));
        assert!(!is_printable_text_input("\u{e000}"));
    }

    #[test]
    fn frame_polling_ignores_subpixel_noise_but_keeps_real_moves() {
        let frame = RelativeWindowFrame {
            x: 0.25,
            y: 0.25,
            width: 0.5,
            height: 0.5,
        };
        assert!(relative_frames_nearly_equal(
            frame,
            RelativeWindowFrame { x: 0.2501, ..frame }
        ));
        assert!(!relative_frames_nearly_equal(
            frame,
            RelativeWindowFrame { x: 0.3, ..frame }
        ));
    }

    #[test]
    fn invocation_gesture_toggles_an_existing_session() {
        assert_eq!(
            invocation_toggle_action(false, false),
            InvocationToggleAction::StartNew
        );
        assert_eq!(
            invocation_toggle_action(true, true),
            InvocationToggleAction::HideExisting
        );
        assert_eq!(
            invocation_toggle_action(true, false),
            InvocationToggleAction::ShowExisting
        );
    }
}
