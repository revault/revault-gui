pub mod charging;
mod cmd;
mod deposit;
mod history;
pub mod installing;
pub mod manager;
mod sign;
pub mod stakeholder;
mod vault;

use iced::{Command, Element, Subscription};

pub use charging::ChargingState;
pub use deposit::DepositState;
pub use history::HistoryState;
pub use installing::InstallingState;
pub use manager::{ManagerHomeState, ManagerNetworkState, ManagerSendState};
pub use stakeholder::{StakeholderACKFundsState, StakeholderHomeState, StakeholderNetworkState};

use super::{message::Message, view::Context};

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
