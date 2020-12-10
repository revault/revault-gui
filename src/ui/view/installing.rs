use iced::Element;

use crate::ui::{component, message::Message, view::layout};

pub fn installing_view() -> Element<'static, Message> {
    layout::cover(component::text::paragraph("Installing"))
}
