use crate::ui::color;
use iced::{button, Color, Container, Row, Text, Vector};

macro_rules! button {
    ($name:ident, $style_name:ident, $bg_color:expr, $text_color:expr) => {
        pub fn $name<'a, T: 'a + Clone>(
            state: &'a mut button::State,
            content: Container<'a, T>,
            message: T,
        ) -> button::Button<'a, T> {
            button::Button::new(state, content)
                .on_press(message)
                .style($style_name {})
        }

        struct $style_name {}
        impl button::StyleSheet for $style_name {
            fn active(&self) -> button::Style {
                button::Style {
                    shadow_offset: Vector::default(),
                    background: $bg_color.into(),
                    border_radius: 10.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                    text_color: $text_color,
                }
            }
        }
    };
}

button!(primary, PrimaryStyle, color::PRIMARY, color::FOREGROUND);

button!(
    transparent,
    TransparentStyle,
    Color::TRANSPARENT,
    Color::BLACK
);

pub fn button_content<'a, T: 'a>(icon: Option<iced::Svg>, text: &str) -> Container<'a, T> {
    match icon {
        None => Container::new(Text::new(text)).padding(5),
        Some(svg) => Container::new(
            Row::new()
                .push(Container::new(svg))
                .push(Text::new(text))
                .spacing(10)
                .align_items(iced::Align::Center),
        )
        .padding(5),
    }
}
