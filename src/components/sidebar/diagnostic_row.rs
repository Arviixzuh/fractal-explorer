use crate::components::sidebar::icon;
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, Symbol};

pub(super) fn build(
    symbol: Symbol,
    label: &str,
    value: impl Into<String>,
    icon_color: Color,
    text_color: Color,
) -> BoxedWidget {
    let value = value.into();
    let text = if value.is_empty() {
        label.to_owned()
    } else {
        format!("{label:<12} {value}")
    };
    Box::new(jsx! {
        <Flex direction={FlexDirection::Row} gap={7.0}>
            {icon(symbol, icon_color, 13.0)}
            <Text font_size={10.0} color={text_color} font_family={crate::MONOSPACE_FONT}>
                {text}
            </Text>
        </Flex>
    })
}
