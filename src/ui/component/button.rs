use crate::ui::{color, component::text, icon::clipboard_icon};
use iced::{button, Color, Container, Row, Vector};

macro_rules! button {
    ($name:ident, $style_name:ident, $bg_color:expr, $text_color:expr) => {
        pub fn $name<'a, T: 'a + Clone>(
            state: &'a mut button::State,
            content: Container<'a, T>,
        ) -> button::Button<'a, T> {
            button::Button::new(state, content).style($style_name {})
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
    primary_disable,
    PrimaryDisableStyle,
    color::PRIMARY_LIGHT,
    color::FOREGROUND
);

button!(cancel, CancelStyle, color::CANCEL, color::FOREGROUND);

button!(important, ImportantStyle, color::CANCEL, color::FOREGROUND);

button!(success, SuccessStyle, color::SUCCESS, color::FOREGROUND);

button!(
    transparent,
    TransparentStyle,
    Color::TRANSPARENT,
    Color::BLACK
);

pub fn button_content<'a, T: 'a>(icon: Option<iced::Text>, text: &str) -> Container<'a, T> {
    match icon {
        None => Container::new(text::simple(text)).padding(5),
        Some(i) => Container::new(
            Row::new()
                .push(i)
                .push(text::simple(text))
                .spacing(10)
                .align_items(iced::Align::Center),
        )
        .padding(5),
    }
}

pub fn clipboard<'a, T: 'a + Clone>(
    state: &'a mut button::State,
    message: T,
) -> button::Button<'a, T> {
    button::Button::new(state, clipboard_icon().size(15))
        .on_press(message)
        .style(ClipboardButtonStyle {})
}

struct ClipboardButtonStyle {}
impl button::StyleSheet for ClipboardButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::default(),
            background: Color::TRANSPARENT.into(),
            border_radius: 10.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT.into(),
            text_color: Color::BLACK.into(),
        }
    }
}

pub fn white_card_button<'a, T: 'a + Clone>(
    state: &'a mut button::State,
    content: Container<'a, T>,
) -> button::Button<'a, T> {
    button::Button::new(state, content.padding(10)).style(WhiteCardButtonStyle {})
}

struct WhiteCardButtonStyle {}
impl button::StyleSheet for WhiteCardButtonStyle {
    fn active(&self) -> button::Style {
        button::Style {
            border_radius: 10.0,
            background: color::FOREGROUND.into(),
            ..button::Style::default()
        }
    }
    fn hovered(&self) -> button::Style {
        button::Style {
            border_radius: 10.0,
            background: color::FOREGROUND.into(),
            border_color: color::SECONDARY,
            border_width: 1.0,
            ..button::Style::default()
        }
    }
}
