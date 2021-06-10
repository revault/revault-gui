use iced::{
    text_input::{self, State, TextInput},
    Column, Container, Length,
};

use crate::ui::{color, component::text};

#[derive(Debug)]
pub struct Value<T> {
    pub value: T,
    pub valid: bool,
}

impl std::default::Default for Value<String> {
    fn default() -> Self {
        Self {
            value: "".to_string(),
            valid: true,
        }
    }
}

pub struct Form<'a, Message> {
    input: TextInput<'a, Message>,
    warning: Option<&'a str>,
    valid: bool,
}

impl<'a, Message: 'a> Form<'a, Message>
where
    Message: Clone,
{
    /// Creates a new [`Form`].
    ///
    /// It expects:
    /// - some [`iced::text_input::State`]
    /// - a placeholder
    /// - the current value
    /// - a function that produces a message when the [`Form`] changes
    pub fn new<F>(
        state: &'a mut State,
        placeholder: &str,
        value: &Value<String>,
        on_change: F,
    ) -> Self
    where
        F: 'static + Fn(String) -> Message,
    {
        Self {
            input: TextInput::new(state, placeholder, &value.value, on_change),
            warning: None,
            valid: value.valid,
        }
    }

    /// Sets the [`Form`] with a warning message
    pub fn warning(mut self, warning: &'a str) -> Self {
        self.warning = Some(warning);
        self
    }

    /// Sets the padding of the [`Form`].
    pub fn padding(mut self, units: u16) -> Self {
        self.input = self.input.padding(units);
        self
    }

    pub fn render(self) -> Container<'a, Message> {
        if !self.valid {
            if let Some(message) = self.warning {
                return Container::new(
                    Column::with_children(vec![
                        self.input.style(InvalidFormStyle).into(),
                        text::small(&message).color(color::WARNING).into(),
                    ])
                    .width(Length::Fill)
                    .spacing(5),
                );
            }
        }

        Container::new(self.input)
    }
}

struct InvalidFormStyle;
impl text_input::StyleSheet for InvalidFormStyle {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: iced::Background::Color(color::FOREGROUND),
            border_radius: 5.0,
            border_width: 1.0,
            border_color: color::WARNING,
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_color: color::WARNING,
            ..self.active()
        }
    }

    fn placeholder_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.3, 0.3, 0.3)
    }

    fn selection_color(&self) -> iced::Color {
        iced::Color::from_rgb(0.8, 0.8, 1.0)
    }
}
