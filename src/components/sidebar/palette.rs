use super::terminal_button;
use crate::palette::Palette;
use crate::{schedule_render, AppState};
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, WindowHandle};
use std::cell::RefCell;
use std::rc::Rc;

pub(super) fn build(
    selected: Palette,
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
) -> BoxedWidget {
    let mono_state = state.clone();
    let mono_window = window_slot.clone();
    let neon_state = state.clone();
    let neon_window = window_slot.clone();
    let blue_state = state.clone();
    let blue_window = window_slot.clone();

    Box::new(jsx! {
        <Flex direction={FlexDirection::Column} gap={4.0}>
            {terminal_button::build(
                Palette::MonochromeTerminal.label(),
                selected == Palette::MonochromeTerminal,
                move || {
                    if mono_state.borrow().palette != Palette::MonochromeTerminal {
                        mono_state.borrow_mut().palette = Palette::MonochromeTerminal;
                        schedule_render(&mono_state, &mono_window);
                    }
                },
            )}
            {terminal_button::build(
                Palette::CyberpunkNeon.label(),
                selected == Palette::CyberpunkNeon,
                move || {
                    if neon_state.borrow().palette != Palette::CyberpunkNeon {
                        neon_state.borrow_mut().palette = Palette::CyberpunkNeon;
                        schedule_render(&neon_state, &neon_window);
                    }
                },
            )}
            {terminal_button::build(
                Palette::DeepVoidBlue.label(),
                selected == Palette::DeepVoidBlue,
                move || {
                    if blue_state.borrow().palette != Palette::DeepVoidBlue {
                        blue_state.borrow_mut().palette = Palette::DeepVoidBlue;
                        schedule_render(&blue_state, &blue_window);
                    }
                },
            )}
        </Flex>
    })
}
