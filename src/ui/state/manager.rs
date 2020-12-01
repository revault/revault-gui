use super::State;
use crate::ui::message::Message;
use crate::ui::view::manager::manager_view;
use iced::{Command, Element};

use crate::revaultd::RevaultD;

#[derive(Debug, Clone)]
pub struct ManagerState {
    revaultd: RevaultD,
}

impl ManagerState {
    pub fn new(revaultd: RevaultD) -> Self {
        ManagerState { revaultd }
    }
}

impl State for ManagerState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        manager_view()
    }
}
