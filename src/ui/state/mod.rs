pub mod charging;
pub mod installing;
pub mod manager;
pub mod stakeholder;
mod util;

use iced::{Command, Element, Subscription};

pub use charging::ChargingState;
pub use installing::InstallingState;
pub use manager::{ManagerHistoryState, ManagerHomeState, ManagerNetworkState, ManagerSendState};
pub use stakeholder::StakeholderState;

use super::message::Message;
use super::view::Context;

pub trait State {
    fn view(&mut self, ctx: &Context) -> Element<Message>;
    fn update(&mut self, message: Message) -> Command<Message>;
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
    fn load(&self) -> Command<Message> {
        Command::none()
    }
}
