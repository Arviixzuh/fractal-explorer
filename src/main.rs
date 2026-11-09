mod camera;
mod components;
mod fractal;
mod palette;

use camera::Camera;
use creamui::core::layout::FlexDirection;
use creamui::core::{BoxedWidget, Point, Size};
use creamui::{jsx, run, AppHandle, ImageData, Theme, WindowHandle, WindowOptions};
use fractal::{adaptive_iterations, FractalMode, RenderParams, RenderedFrame};
use palette::Palette;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const SIDEBAR_WIDTH: f32 = 320.0;
const SIDEBAR_LEFT: f32 = 14.0;
const SIDEBAR_TOP: f32 = 14.0;
const MIN_ITERATIONS: u32 = 64;
const MAX_ITERATIONS: u32 = 4096;
const FINAL_RENDER_DELAY: Duration = Duration::from_millis(180);
const PREVIEW_RENDER_DELAY: Duration = Duration::from_millis(55);
const PREVIEW_DIVISOR: u32 = 2;
const HOLD_ZOOM_INTERVAL: Duration = Duration::from_millis(16);
const HOLD_ZOOM_OCTAVES_PER_SECOND: f64 = 1.6;
const PAN_START_DISTANCE: f32 = 4.0;
const MONOSPACE_FONT: &str = "Consolas, 'Courier New', monospace";
const DISPLAY_FONT: &str = "Georgia, 'Times New Roman', serif";

#[derive(Clone)]
struct DisplayFrame {
    image: ImageData,
    camera: Camera,
}

#[derive(Clone)]
struct AppState {
    camera: Camera,
    iterations: u32,
    palette: Palette,
    mode: FractalMode,
    render_width: u32,
    render_height: u32,
    generation: u64,
    render_running: bool,
    debounce_running: bool,
    interactive_pending: bool,
    debounce_generation: Arc<AtomicU64>,
    visual_camera: Rc<Cell<Camera>>,
    image: Option<DisplayFrame>,
    render_ms: f32,
    revision: creamui::Signal<u64>,
    last_pointer: Point,
    drag_pointer: Option<Point>,
    drag_origin: Option<Point>,
    hold_zoom_pointer: Option<Point>,
    hold_zoom_last_tick: Option<Instant>,
    drag_pan_active: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            iterations: 320,
            palette: Palette::CyberpunkNeon,
            mode: FractalMode::Mandelbrot,
            render_width: 800,
            render_height: 600,
            generation: 0,
            render_running: false,
            debounce_running: false,
            interactive_pending: false,
            debounce_generation: Arc::new(AtomicU64::new(0)),
            visual_camera: Rc::new(Cell::new(Camera::default())),
            image: None,
            render_ms: 0.0,
            revision: creamui::Signal::new(0),
            last_pointer: Point { x: 400.0, y: 300.0 },
            drag_pointer: None,
            drag_origin: None,
            hold_zoom_pointer: None,
            hold_zoom_last_tick: None,
            drag_pan_active: false,
        }
    }
}

fn main() {
    let mut theme = Theme::dark();
    theme.font_family = MONOSPACE_FONT;
    theme.button_radius = 2.0;
    theme.card_radius = 3.0;
    theme.colors.surface = creamui::Color::rgba(8, 9, 9, 220);
    theme.colors.surface_elevated = creamui::Color::rgba(16, 18, 17, 232);
    theme.colors.surface_hover = creamui::Color::rgba(174, 195, 184, 28);
    theme.colors.accent = creamui::Color::rgb(174, 195, 184);
    theme.colors.accent_hover = creamui::Color::rgb(215, 225, 219);
    theme.colors.accent_pressed = creamui::Color::rgb(139, 165, 151);
    theme.colors.text_primary = creamui::Color::rgb(232, 230, 224);
    theme.colors.text_secondary = creamui::Color::rgb(142, 147, 142);
    theme.colors.border = creamui::Color::rgba(174, 195, 184, 76);
    theme.colors.border_strong = creamui::Color::rgba(198, 215, 206, 145);
    let state = Rc::new(RefCell::new(AppState::default()));
    let window_slot = Rc::new(RefCell::new(None::<WindowHandle>));
    let frame_counter = Rc::new(RefCell::new((Instant::now(), 0_u32, 0.0_f32)));
    let ready_window = window_slot.clone();
    let ready_state = state.clone();
    run(
        WindowOptions {
            title: "Mandel-Engine // Fractal Explorer".into(),
            width: 1180,
            height: 760,
            resizable: true,
            theme,
            ..Default::default()
        },
        creamui::Color::rgb(4, 5, 5),
        move |window| {
            let stop_telemetry = Arc::new(AtomicBool::new(false));
            window.on_close_requested({
                let stop_telemetry = stop_telemetry.clone();
                move || {
                    stop_telemetry.store(true, Ordering::Relaxed);
                    true
                }
            });
            window.on_pointer_moved({
                let state = ready_state.clone();
                move |point| {
                    let mut state = state.borrow_mut();
                    state.last_pointer = Point {
                        x: point.x.clamp(0.0, state.render_width as f32),
                        y: point.y.clamp(0.0, state.render_height as f32),
                    };
                }
            });
            let app = window.app();
            *ready_window.borrow_mut() = Some(window.clone());
            schedule_render(&ready_state, &ready_window);
            schedule_telemetry_tick(app, ready_state.clone(), window, stop_telemetry);
        },
        {
            let state = state.clone();
            let window_slot = window_slot.clone();
            let frame_counter = frame_counter.clone();
            move |viewport: Size| -> BoxedWidget {
                let _revision = state.borrow().revision.get();
                let current_fps = frame_counter.borrow().2;

                let canvas_width = viewport.width.max(1.0);
                let canvas_height = viewport.height.max(1.0);
                let (width, height) = (canvas_width.round() as u32, canvas_height.round() as u32);
                let dimensions_changed = {
                    let mut state = state.borrow_mut();
                    if state.render_width == width && state.render_height == height {
                        false
                    } else {
                        state.render_width = width;
                        state.render_height = height;
                        true
                    }
                };
                if dimensions_changed {
                    schedule_render(&state, &window_slot);
                }

                let canvas_state = state.clone();
                let drag_start_state = canvas_state.clone();
                let drag_start_window = window_slot.clone();
                let drag_state = canvas_state.clone();
                let drag_window = window_slot.clone();
                let scroll_state = canvas_state.clone();
                let scroll_window = window_slot.clone();
                let canvas = components::fractal_canvas::build(components::fractal_canvas::Props {
                    style: creamui::core::layout::Style {
                        flex_grow: 1.0,
                        ..Default::default()
                    }
                    .into(),
                    image: state.borrow().image.clone(),
                    visual_camera: state.borrow().visual_camera.clone(),
                    frame_counter: frame_counter.clone(),
                    on_drag_start: Rc::new(move |point, _| {
                        let start_zoom = {
                            let mut state = drag_start_state.borrow_mut();
                            state.drag_origin = Some(point);
                            state.drag_pan_active = false;
                            if point_is_over_menu(point, state.render_height as f32) {
                                state.drag_pointer = None;
                                state.hold_zoom_pointer = None;
                                state.hold_zoom_last_tick = None;
                                false
                            } else {
                                state.drag_pointer = Some(point);
                                state.hold_zoom_pointer = Some(point);
                                state.hold_zoom_last_tick = Some(Instant::now());
                                true
                            }
                        };
                        if start_zoom {
                            schedule_hold_zoom_tick(&drag_start_state, &drag_start_window);
                        }
                    }),
                    on_drag: Rc::new(move |point, _| {
                        {
                            let mut state = drag_state.borrow_mut();
                            let height = state.render_height as f32;
                            if state
                                .drag_origin
                                .is_some_and(|origin| point_is_over_menu(origin, height))
                            {
                                return;
                            }
                            if let Some(previous) = state.drag_pointer.replace(point) {
                                let origin = state.drag_origin.unwrap_or(previous);
                                let start_distance = (point.x - origin.x).hypot(point.y - origin.y);
                                let began_pan =
                                    !state.drag_pan_active && start_distance >= PAN_START_DISTANCE;
                                if began_pan {
                                    state.drag_pan_active = true;
                                    state.hold_zoom_pointer = None;
                                    state.hold_zoom_last_tick = None;
                                }
                                if state.drag_pan_active {
                                    let render_height = state.render_height as f64;
                                    state.camera.pan_pixels(
                                        (point.x - if began_pan { origin.x } else { previous.x })
                                            as f64,
                                        (point.y - if began_pan { origin.y } else { previous.y })
                                            as f64,
                                        render_height,
                                    );
                                    state.visual_camera.set(state.camera);
                                } else {
                                    state.hold_zoom_pointer = Some(point);
                                }
                            }
                        }
                        schedule_interactive_render(&drag_state, &drag_window);
                    }),
                    on_drag_end: Rc::new({
                        let state = canvas_state.clone();
                        let window_slot = window_slot.clone();
                        move || {
                            let had_canvas_gesture = {
                                let mut state = state.borrow_mut();
                                let active = state.drag_pointer.is_some()
                                    || state.hold_zoom_pointer.is_some()
                                    || state.drag_pan_active;
                                state.drag_pointer = None;
                                state.drag_origin = None;
                                state.hold_zoom_pointer = None;
                                state.hold_zoom_last_tick = None;
                                state.drag_pan_active = false;
                                active
                            };
                            if had_canvas_gesture {
                                schedule_interactive_render(&state, &window_slot);
                            }
                        }
                    }),
                    on_scroll: Rc::new(move |delta| {
                        {
                            let mut state = scroll_state.borrow_mut();
                            let pointer = state.last_pointer;
                            if point_is_over_menu(pointer, state.render_height as f32) {
                                return;
                            }
                            let render_width = state.render_width as f64;
                            let render_height = state.render_height as f64;
                            state.camera.zoom_at(
                                pointer.x as f64,
                                pointer.y as f64,
                                render_width,
                                render_height,
                                1.15_f64.powf((-(delta as f64) / 40.0).clamp(-8.0, 8.0)),
                            );
                            state.visual_camera.set(state.camera);
                        }
                        schedule_interactive_render(&scroll_state, &scroll_window);
                    }),
                });

                Box::new(jsx! {
                    <Flex
                        direction={FlexDirection::Column}
                        size={(viewport.width, viewport.height)}
                        background={creamui::Color::rgb(4, 5, 5)}
                    >
                        {canvas}
                        {components::sidebar::build(&state, &window_slot, current_fps, viewport.height)}
                    </Flex>
                })
            }
        },
    );
}

fn point_is_over_menu(point: Point, canvas_height: f32) -> bool {
    point.x >= SIDEBAR_LEFT
        && point.x <= SIDEBAR_LEFT + SIDEBAR_WIDTH
        && point.y >= SIDEBAR_TOP
        && point.y <= SIDEBAR_TOP + (canvas_height - 2.0 * SIDEBAR_TOP).max(0.0)
}

fn schedule_render(state: &Rc<RefCell<AppState>>, window_slot: &Rc<RefCell<Option<WindowHandle>>>) {
    {
        let mut state = state.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        state.debounce_generation.fetch_add(1, Ordering::Relaxed);
        state.debounce_running = false;
        state.interactive_pending = false;
    }
    refresh_ui(state);
    launch_latest_render(state, window_slot, RenderQuality::Full);
}

fn schedule_interactive_render(
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
) {
    if let Some(window) = window_slot.borrow().as_ref() {
        window.request_repaint();
    }
    let (token, should_start) = {
        let mut state = state.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        let token = state.debounce_generation.fetch_add(1, Ordering::Relaxed) + 1;
        state.interactive_pending = true;
        let should_start = !state.debounce_running;
        state.debounce_running = true;
        (token, should_start)
    };
    if should_start {
        start_interaction_timer(
            state,
            window_slot,
            token,
            PREVIEW_RENDER_DELAY,
            RenderQuality::Preview,
        );
    }
}

fn schedule_hold_zoom_tick(
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
) {
    let Some(window) = window_slot.borrow().clone() else {
        return;
    };
    let state_for_result = state.clone();
    let window_slot_for_result = window_slot.clone();
    window.app().spawn_background(
        move || {
            std::thread::sleep(HOLD_ZOOM_INTERVAL);
            Instant::now()
        },
        move |tick_time| {
            let is_zooming = {
                let mut state = state_for_result.borrow_mut();
                match (state.hold_zoom_pointer, state.hold_zoom_last_tick) {
                    (Some(pointer), Some(last_tick)) => {
                        let elapsed = tick_time
                            .saturating_duration_since(last_tick)
                            .as_secs_f64()
                            .clamp(0.0, 0.05);
                        state.hold_zoom_last_tick = Some(tick_time);
                        let factor = 2.0_f64.powf(elapsed * HOLD_ZOOM_OCTAVES_PER_SECOND);
                        let (width, height) =
                            (state.render_width as f64, state.render_height as f64);
                        state.camera.zoom_at(
                            pointer.x as f64,
                            pointer.y as f64,
                            width,
                            height,
                            factor,
                        );
                        state.visual_camera.set(state.camera);
                        true
                    }
                    _ => false,
                }
            };
            if is_zooming {
                schedule_interactive_render(&state_for_result, &window_slot_for_result);
                schedule_hold_zoom_tick(&state_for_result, &window_slot_for_result);
            }
        },
    );
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RenderQuality {
    Preview,
    Full,
}

fn start_interaction_timer(
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
    token: u64,
    delay: Duration,
    quality: RenderQuality,
) {
    let Some(window) = window_slot.borrow().clone() else {
        return;
    };
    let state_for_result = state.clone();
    let window_slot_for_result = window_slot.clone();
    window.app().spawn_background(
        move || {
            std::thread::sleep(delay);
            token
        },
        move |timer_token| {
            let (current_token, interactive_pending, gesture_active) = {
                let mut state = state_for_result.borrow_mut();
                state.debounce_running = false;
                (
                    state.debounce_generation.load(Ordering::Relaxed),
                    state.interactive_pending,
                    state.hold_zoom_pointer.is_some() || state.drag_pan_active,
                )
            };
            if !interactive_pending {
                return;
            }
            let quality = if gesture_active || timer_token != current_token {
                RenderQuality::Full
            } else {
                quality
            };
            if quality == RenderQuality::Full && !gesture_active {
                state_for_result.borrow_mut().interactive_pending = false;
            }
            launch_latest_render(&state_for_result, &window_slot_for_result, quality);
        },
    );
}

fn launch_latest_render(
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
    quality: RenderQuality,
) {
    let Some(window) = window_slot.borrow().clone() else {
        return;
    };
    let (params, generation) = {
        let mut state = state.borrow_mut();
        if state.render_running {
            return;
        }
        state.render_running = true;
        let (width, height, iterations) = match quality {
            RenderQuality::Preview => (
                state.render_width.div_ceil(PREVIEW_DIVISOR).max(1),
                state.render_height.div_ceil(PREVIEW_DIVISOR).max(1),
                adaptive_iterations(state.iterations, state.camera.zoom, MAX_ITERATIONS),
            ),
            RenderQuality::Full => (
                state.render_width,
                state.render_height,
                adaptive_iterations(state.iterations, state.camera.zoom, MAX_ITERATIONS),
            ),
        };
        (
            RenderParams {
                width,
                height,
                camera: state.camera,
                iterations,
                palette: state.palette,
                mode: state.mode,
            },
            state.generation,
        )
    };
    let state_for_result = state.clone();
    let window_for_result = window.clone();
    let window_slot_for_result = window_slot.clone();
    window.app().spawn_background(
        move || fractal::render(params),
        move |result: Result<RenderedFrame, String>| match result {
            Ok(frame) => {
                let (is_current, gesture_active) = {
                    let mut state = state_for_result.borrow_mut();
                    state.render_running = false;
                    (
                        state.generation == generation,
                        state.hold_zoom_pointer.is_some() || state.drag_pan_active,
                    )
                };
                if !is_current && !gesture_active {
                    if !state_for_result.borrow().debounce_running {
                        if state_for_result.borrow().interactive_pending {
                            schedule_interactive_render(&state_for_result, &window_slot_for_result);
                        } else {
                            launch_latest_render(
                                &state_for_result,
                                &window_slot_for_result,
                                RenderQuality::Full,
                            );
                        }
                    }
                    return;
                }
                let image =
                    ImageData::from_premultiplied_rgba(frame.width, frame.height, frame.pixels);
                match image {
                    Ok(image) => {
                        let mut state = state_for_result.borrow_mut();
                        if is_current || gesture_active {
                            state.image = Some(DisplayFrame {
                                image,
                                camera: frame.camera,
                            });
                            state.render_ms = frame.elapsed_ms;
                            drop(state);
                            refresh_ui(&state_for_result);
                            window_for_result.request_repaint();
                            let (token, should_start, delay) = {
                                let mut state = state_for_result.borrow_mut();
                                let should_start =
                                    state.interactive_pending && !state.debounce_running;
                                if should_start {
                                    state.debounce_running = true;
                                }
                                (
                                    state.debounce_generation.load(Ordering::Relaxed),
                                    should_start,
                                    if state.hold_zoom_pointer.is_some() || state.drag_pan_active {
                                        FINAL_RENDER_DELAY
                                    } else {
                                        PREVIEW_RENDER_DELAY
                                    },
                                )
                            };
                            if should_start {
                                start_interaction_timer(
                                    &state_for_result,
                                    &window_slot_for_result,
                                    token,
                                    delay,
                                    RenderQuality::Full,
                                );
                            }
                        }
                    }
                    Err(error) => {
                        eprintln!("FRACTAL EXPLORER: could not create image: {error}");
                    }
                }
            }
            Err(error) => {
                let should_restart = {
                    let mut state = state_for_result.borrow_mut();
                    state.render_running = false;
                    if state.generation == generation {
                        state.interactive_pending = false;
                    }
                    !state.debounce_running && state.generation != generation
                };
                eprintln!("FRACTAL EXPLORER: render failed: {error}");
                if should_restart {
                    launch_latest_render(
                        &state_for_result,
                        &window_slot_for_result,
                        RenderQuality::Full,
                    );
                }
            }
        },
    );
}

fn refresh_ui(state: &Rc<RefCell<AppState>>) {
    let revision = state.borrow().revision.clone();
    revision.update(|value| *value = value.wrapping_add(1));
}

fn schedule_telemetry_tick(
    app: AppHandle,
    state: Rc<RefCell<AppState>>,
    window: WindowHandle,
    stop: Arc<AtomicBool>,
) {
    let next_app = app.clone();
    app.spawn_background(
        || std::thread::sleep(Duration::from_millis(500)),
        move |_| {
            if stop.load(Ordering::Relaxed) || !window.is_open() {
                return;
            }
            refresh_ui(&state);
            schedule_telemetry_tick(next_app, state, window, stop);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use creamui::core::Rect;

    #[test]
    fn transformed_preview_keeps_the_zoom_anchor_in_place() {
        let frame_camera = Camera::default();
        let mut current_camera = frame_camera;
        let (x, y) = (137.0_f32, 211.0_f32);
        current_camera.zoom_at(x as f64, y as f64, 800.0, 600.0, 2.0);
        let destination = components::fractal_canvas::transformed_frame_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            frame_camera,
            current_camera,
        );
        let image_scale = destination.width / 800.0;
        let mapped_x = destination.x + x * image_scale;
        let mapped_y = destination.y + y * image_scale;
        assert!((mapped_x - x).abs() < 0.001);
        assert!((mapped_y - y).abs() < 0.001);
    }

    #[test]
    fn clicks_inside_sidebar_are_not_zoom_gestures() {
        assert!(point_is_over_menu(Point { x: 20.0, y: 20.0 }, 760.0));
        assert!(point_is_over_menu(Point { x: 300.0, y: 500.0 }, 760.0));
        assert!(!point_is_over_menu(Point { x: 350.0, y: 500.0 }, 760.0));
        assert!(!point_is_over_menu(Point { x: 10.0, y: 20.0 }, 760.0));
        assert!(!point_is_over_menu(Point { x: 300.0, y: 750.0 }, 760.0));
    }
}
