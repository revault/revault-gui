use std::sync::Arc;

use iced::{Command, Element};

use super::State;
use crate::revaultd::RevaultD;
use crate::ui::message::{Context, Message};

#[derive(Debug)]
pub struct StakeholderHomeState {
    revaultd: Arc<RevaultD>,
}

impl StakeholderHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderHomeState { revaultd }
    }
}

impl State for StakeholderHomeState {
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

impl From<StakeholderHomeState> for Box<dyn State> {
    fn from(s: StakeholderHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}
