use iced::{Column, Container, Element, Length, Text};

use crate::ui::{ds::image::revaut_colored_logo, message::Message};

pub fn installing_view() -> Element<'static, Message> {
    let svg = revaut_colored_logo()
        .width(Length::Units(300))
        .height(Length::Fill);

    let text = Text::new("Installing");

    let content = Column::new().push(svg).push(text);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
}
