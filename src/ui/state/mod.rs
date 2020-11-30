pub mod charging;

use iced::Element;

pub trait State {
    fn view(&mut self) -> Element<Message>;
}

#[derive(Debug, Clone)]
pub enum Message {}
