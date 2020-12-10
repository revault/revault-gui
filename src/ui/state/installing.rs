use super::State;
use crate::ui::message::Message;
use crate::ui::view::installing::installing_view;
use iced::{Command, Element};

#[derive(Debug, Clone)]
pub struct InstallingState {}

impl State for InstallingState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        installing_view()
    }
}
