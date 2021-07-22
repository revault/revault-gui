use super::{color, font};
use iced::{Container, Text};

pub fn simple(content: &str) -> Text {
    Text::new(content).font(font::REGULAR).size(20)
}

pub fn small(content: &str) -> Text {
    Text::new(content).font(font::REGULAR).size(15)
}

pub fn paragraph<'a, T: 'a>(s: &str) -> Container<'a, T> {
    Container::new(Text::new(s).font(font::REGULAR))
}

pub fn bold(t: Text) -> Text {
    t.font(font::BOLD)
}

pub fn success(t: Text) -> Text {
    t.color(color::SUCCESS)
}
