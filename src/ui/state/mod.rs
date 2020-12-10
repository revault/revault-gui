pub mod charging;
pub mod installing;
pub mod manager;

use iced::{Command, Element, Subscription};

use super::message::Message;

pub trait State {
    fn view(&mut self) -> Element<Message>;
    fn update(&mut self, message: Message) -> Command<Message>;
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}
