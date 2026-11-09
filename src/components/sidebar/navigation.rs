use super::diagnostic_row;
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, Symbol};

pub(super) fn build() -> BoxedWidget {
    let accent = Color::rgb(174, 195, 184);
    let muted = Color::rgb(142, 147, 142);
    Box::new(jsx! {
        <Flex direction={FlexDirection::Column} gap={4.0}>
            {diagnostic_row::build(Symbol::Search, "HOLD : ZOOM TO CURSOR", "", accent, muted)}
            {diagnostic_row::build(Symbol::Display, "DRAG : PAN PLANE", "", accent, muted)}
            {diagnostic_row::build(Symbol::Controls, "WHEEL : ZOOM IN / OUT", "", accent, muted)}
        </Flex>
    })
}
