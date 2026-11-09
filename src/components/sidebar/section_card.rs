use super::icon;
use creamui::core::layout::FlexDirection;
use creamui::core::BoxedWidget;
use creamui::{jsx, Color, Symbol};

pub(super) fn build(symbol: Symbol, label: &str, accent: Color, body: BoxedWidget) -> BoxedWidget {
    Box::new(jsx! {
        <Flex
            direction={FlexDirection::Column}
            gap={6.0}
            padding={8.0}
            background={Color::rgba(0, 0, 0, 135)}
            border={(Color::rgba(174, 195, 184, 58), 1.0)}
            corner_radius={3.0}
        >
            <Flex direction={FlexDirection::Row} gap={6.0}>
                {icon(symbol, accent, 14.0)}
                <Text font_size={11.0} color={accent} font_family={crate::MONOSPACE_FONT}>
                    {label}
                </Text>
            </Flex>
            {body}
        </Flex>
    })
}
