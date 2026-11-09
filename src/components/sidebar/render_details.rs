use super::terminal_button;
use crate::{schedule_render, AppState, Camera, MAX_ITERATIONS, MIN_ITERATIONS};
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, WindowHandle};
use std::cell::RefCell;
use std::rc::Rc;

pub(super) fn build(
    iterations: u32,
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
) -> BoxedWidget {
    let decrement_state = state.clone();
    let decrement_window = window_slot.clone();
    let increment_state = state.clone();
    let increment_window = window_slot.clone();
    let slider_state = state.clone();
    let slider_window = window_slot.clone();
    let reset_state = state.clone();
    let reset_window = window_slot.clone();

    Box::new(jsx! {
        <Flex direction={FlexDirection::Column} gap={6.0}>
            <Flex direction={FlexDirection::Row} gap={6.0}>
                {terminal_button::build("-", false, move || {
                    decrement_state.borrow_mut().iterations =
                        iterations.saturating_sub(64).max(MIN_ITERATIONS);
                    schedule_render(&decrement_state, &decrement_window);
                })}
                {terminal_button::build("+", false, move || {
                    increment_state.borrow_mut().iterations = (iterations + 64).min(MAX_ITERATIONS);
                    schedule_render(&increment_state, &increment_window);
                })}
            </Flex>
            <Text font_size={10.0} color={Color::rgb(142, 147, 142)} font_family={crate::MONOSPACE_FONT}>
                {format!("DEPTH  {iterations}")}
            </Text>
            <Slider
                value={(iterations.saturating_sub(MIN_ITERATIONS) as f32)
                    / (MAX_ITERATIONS - MIN_ITERATIONS) as f32}
                on_change={move |value| {
                    let iterations = MIN_ITERATIONS
                        + ((MAX_ITERATIONS - MIN_ITERATIONS) as f32 * value.clamp(0.0, 1.0))
                            .round() as u32;
                    if slider_state.borrow().iterations != iterations {
                        slider_state.borrow_mut().iterations = iterations;
                        schedule_render(&slider_state, &slider_window);
                    }
                }}
            />
            {terminal_button::build("Reset View", false, move || {
                let mut current = reset_state.borrow_mut();
                current.camera = Camera::default();
                current.visual_camera.set(current.camera);
                drop(current);
                schedule_render(&reset_state, &reset_window);
            })}
        </Flex>
    })
}
