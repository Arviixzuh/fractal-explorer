use super::terminal_button;
use crate::fractal::FractalMode;
use crate::{schedule_render, AppState};
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, WindowHandle};
use std::cell::RefCell;
use std::rc::Rc;

pub(super) fn build(
    selected: FractalMode,
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
) -> BoxedWidget {
    let mandelbrot_state = state.clone();
    let mandelbrot_window = window_slot.clone();
    let julia_state = state.clone();
    let julia_window = window_slot.clone();

    Box::new(jsx! {
        <Flex direction={FlexDirection::Column} gap={4.0}>
            {terminal_button::build(
                FractalMode::Mandelbrot.label(),
                selected == FractalMode::Mandelbrot,
                move || {
                    if mandelbrot_state.borrow().mode != FractalMode::Mandelbrot {
                        mandelbrot_state.borrow_mut().mode = FractalMode::Mandelbrot;
                        schedule_render(&mandelbrot_state, &mandelbrot_window);
                    }
                },
            )}
            {terminal_button::build(
                FractalMode::Julia.label(),
                selected == FractalMode::Julia,
                move || {
                    if julia_state.borrow().mode != FractalMode::Julia {
                        julia_state.borrow_mut().mode = FractalMode::Julia;
                        schedule_render(&julia_state, &julia_window);
                    }
                },
            )}
        </Flex>
    })
}
