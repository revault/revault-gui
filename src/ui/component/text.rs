use super::{color, font};
use iced::{Color, Element, HorizontalAlignment, Length};

pub struct Text(iced::Text);

impl Text {
    pub fn new(content: &str) -> Self {
        Self(iced::Text::new(content).font(font::REGULAR).size(20))
    }

    pub fn bold(mut self) -> Self {
        self.0 = self.0.font(font::BOLD);
        self
    }

    pub fn small(mut self) -> Self {
        self.0 = self.0.size(15);
        self
    }

    pub fn size(mut self, i: u16) -> Self {
        self.0 = self.0.size(i);
        self
    }

    pub fn success(mut self) -> Self {
        self.0 = self.0.color(color::SUCCESS);
        self
    }
    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.0 = self.0.horizontal_alignment(alignment);
        self
    }
    pub fn width(mut self, width: Length) -> Self {
        self.0 = self.0.width(width);
        self
    }

    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.0 = self.0.color(color);
        self
    }
}

impl<'a, Message> From<Text> for Element<'a, Message> {
    fn from(text: Text) -> Element<'a, Message> {
        text.0.into()
    }
}

impl From<iced::Text> for Text {
    fn from(text: iced::Text) -> Text {
        Text(text)
    }
}
