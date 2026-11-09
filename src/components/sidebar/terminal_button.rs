use creamui::core::layout::{AlignItems, Display, FlexDirection, JustifyContent};
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, StateStyle, Style};

pub(super) fn build(label: &str, selected: bool, on_click: impl Fn() + 'static) -> BoxedWidget {
    let text_color = if selected {
        Color::rgb(174, 195, 184)
    } else {
        Color::rgb(232, 230, 224)
    };
    let background = if selected {
        Color::rgba(174, 195, 184, 32)
    } else {
        Color::rgba(12, 14, 13, 210)
    };
    let border = Color::rgba(174, 195, 184, if selected { 220 } else { 90 });
    let shadow = creamui::BoxShadow::new(
        0.0,
        0.0,
        if selected { 10.0 } else { 0.0 },
        0.0,
        Color::rgba(174, 195, 184, if selected { 34 } else { 0 }),
    );
    let button_style = Style::new()
        .display(Display::Flex)
        .flex_direction(FlexDirection::Row)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .width("100%")
        .padding("4px")
        .background(background)
        .color(text_color)
        .font_size(11.0)
        .font_family(crate::MONOSPACE_FONT)
        .text_align(creamui::core::TextAlign::Center)
        .border(border, 1.0)
        .corner_radius(4.0)
        .box_shadow(shadow)
        .hover(
            StateStyle::new()
                .background(Color::rgba(174, 195, 184, 48))
                .border(Color::rgb(198, 215, 206), 1.0),
        )
        .pressed(
            StateStyle::new()
                .background(Color::rgba(174, 195, 184, 82))
                .border(Color::rgb(215, 225, 219), 1.0),
        );
    Box::new(jsx! {
        <Button style={button_style} on_click={on_click}>{label}</Button>
    })
}
