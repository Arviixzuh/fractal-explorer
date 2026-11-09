mod diagnostic_row;
mod fractal_mode;
mod navigation;
mod palette;
mod render_details;
mod section_card;
mod status;
mod terminal_button;

use crate::{AppState, SIDEBAR_LEFT, SIDEBAR_TOP, SIDEBAR_WIDTH};
use creamui::core::layout::{FlexDirection, Position};
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, WindowHandle};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) fn build(
    state: &Rc<RefCell<AppState>>,
    window_slot: &Rc<RefCell<Option<WindowHandle>>>,
    fps: f32,
    height: f32,
) -> BoxedWidget {
    let (iterations, palette, mode, render_ms, center, zoom) = {
        let state = state.borrow();
        (
            state.iterations,
            state.palette,
            state.mode,
            state.render_ms,
            state.camera.center,
            state.camera.zoom,
        )
    };
    let accent = Color::rgb(174, 195, 184);

    Box::new(jsx! {
        <Flex
            direction={FlexDirection::Column}
            size={(SIDEBAR_WIDTH, (height - 28.0).max(1.0))}
            gap={9.0}
            padding={14.0}
            background={Color::rgba(8, 9, 9, 222)}
            border={(Color::rgba(174, 195, 184, 110), 1.0)}
            corner_radius={3.0}
            position={Position::Absolute}
            left={SIDEBAR_LEFT}
            top={SIDEBAR_TOP}
            box_shadow={creamui::core::BoxShadow::new(
                0.0, 0.0, 22.0, 0.0, Color::rgba(174, 195, 184, 18)
            )}
        >
            <Flex direction={FlexDirection::Row} gap={9.0}>
                {icon(creamui::Symbol::Display, accent, 22.0)}
                <Text font_size={19.0} color={Color::rgb(232, 230, 224)} font_family={crate::DISPLAY_FONT}>
                    "Fractal Explorer"
                </Text>
            </Flex>
            <Text font_size={10.0} color={Color::rgb(142, 147, 142)}>
                "COMPLEX PLANE  //  LIVE VISUALIZATION"
            </Text>
            {section_card::build(
                creamui::Symbol::Grid,
                "System Status",
                accent,
                status::build(fps, iterations, render_ms, center, zoom),
            )}
            {section_card::build(
                creamui::Symbol::Controls,
                "Fractal Type",
                accent,
                fractal_mode::build(mode, state, window_slot),
            )}
            {section_card::build(
                creamui::Symbol::Appearance,
                "Visual Theme",
                accent,
                palette::build(palette, state, window_slot),
            )}
            {section_card::build(
                creamui::Symbol::Sliders,
                "Render Detail",
                accent,
                render_details::build(iterations, state, window_slot),
            )}
            {section_card::build(
                creamui::Symbol::Search,
                "Navigation",
                accent,
                navigation::build(),
            )}
        </Flex>
    })
}

pub(super) fn icon(symbol: creamui::Symbol, color: Color, size: f32) -> BoxedWidget {
    Box::new(creamui::Icon::new(symbol, color).size(size))
}
