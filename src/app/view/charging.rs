use iced::Element;

use crate::app::{component, message::Message, view::layout};

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
