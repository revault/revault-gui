use std::sync::Arc;

use iced::{Command, Element};

use super::State;
use crate::revaultd::RevaultD;
use crate::ui::{message::Message, view::Context};

#[derive(Debug)]
pub struct StakeholderState {
    revaultd: Arc<RevaultD>,
}

impl StakeholderState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderState { revaultd }
    }
}

impl State for StakeholderState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self, _ctx: &Context) -> Element<Message> {
        iced::Container::new(iced::Text::new("hello")).into()
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![])
    }
}

impl From<StakeholderState> for Box<dyn State> {
    fn from(s: StakeholderState) -> Box<dyn State> {
        Box::new(s)
    }
}
