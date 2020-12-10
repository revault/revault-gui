use iced::{button, Column, Container, Element, Length, Text};

use crate::ui::{ds::image::revaut_colored_logo, message::Message};

pub fn charging_connect_view() -> Element<'static, Message> {
    let svg = revaut_colored_logo()
        .width(Length::Units(300))
        .height(Length::Fill);

    let text = Text::new("Connecting to daemon...");

    let content = Column::new().push(svg).push(text);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
}

pub fn charging_starting_daemon_view() -> Element<'static, Message> {
    let svg = revaut_colored_logo()
        .width(Length::Units(300))
        .height(Length::Fill);

    let text = Text::new("Starting daemon...");

    let content = Column::new().push(svg).push(text);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
}

pub fn charging_syncing_view(progress: &f64) -> Element<'static, Message> {
    let svg = revaut_colored_logo()
        .width(Length::Units(300))
        .height(Length::Fill);

    let text = Text::new(format!("Syncing... {}%", progress));

    let content = Column::new().push(svg).push(text);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
}

pub fn charging_error_view(error: &str) -> Element<'static, Message> {
    let svg = revaut_colored_logo()
        .width(Length::Units(300))
        .height(Length::Fill);

    let text = Text::new(format!("Error: {}", error));

    let content = Column::new().push(svg).push(text);

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .center_x()
        .center_y()
        .into()
}

#[derive(Debug, Clone)]
pub struct ChargingAskInstallView {
    validate_button: button::State,
}

impl ChargingAskInstallView {
    pub fn new() -> ChargingAskInstallView {
        Self {
            validate_button: button::State::new(),
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let svg = revaut_colored_logo()
            .width(Length::Units(300))
            .height(Length::Fill);

        let text = Text::new("No config do you want to install ?");
        let button = button::Button::new(&mut self.validate_button, Text::new("Install"))
            .on_press(Message::Install);

        let content = Column::new().push(svg).push(text).push(button);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
