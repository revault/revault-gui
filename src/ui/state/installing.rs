use super::State;
use crate::ui::message::Message;
use crate::ui::view::installing::installing_view;
use iced::{Command, Element};

#[derive(Debug, Clone)]
pub struct InstallingState {}

impl InstallingState {
    pub fn new() -> Self {
        InstallingState {}
    }
}

impl State for InstallingState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        installing_view()
    }
}

impl From<InstallingState> for Box<dyn State> {
    fn from(s: InstallingState) -> Box<dyn State> {
        Box::new(s)
    }
}
