use iced::{button, Column, Container, Element};

use crate::ui::{component, message::Message, view::layout};

pub fn charging_connect_view() -> Element<'static, Message> {
    layout::cover(component::text::paragraph("Connecting to daemon..."))
}

pub fn charging_starting_daemon_view() -> Element<'static, Message> {
    layout::cover(component::text::paragraph("Starting daemon..."))
}

pub fn charging_syncing_view(progress: &f64) -> Element<'static, Message> {
    layout::cover(component::text::paragraph(&format!(
        "Syncing... {}%",
        progress
    )))
}

pub fn charging_error_view(error: &str) -> Element<'static, Message> {
    layout::cover(component::text::paragraph(&format!("Error: {}", error)))
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
        let text = component::text::paragraph("No config do you want to install ?");
        let button = button::Button::new(
            &mut self.validate_button,
            component::text::simple("Install"),
        )
        .on_press(Message::Install);

        layout::cover(Container::new(Column::new().push(text).push(button)))
    }
}
