pub mod cmd;
mod deposit;
mod emergency;
pub mod manager;
mod settings;
mod sign;
mod spend_transaction;
pub mod stakeholder;
mod vault;
mod vaults;

use iced::{Command, Element, Subscription};

pub use deposit::DepositState;
pub use emergency::EmergencyState;
pub use manager::{ManagerHomeState, ManagerSendState};
pub use settings::SettingsState;
pub use spend_transaction::{SpendTransactionListItem, SpendTransactionState};
pub use stakeholder::{
    StakeholderCreateVaultsState, StakeholderDelegateVaultsState, StakeholderHomeState,
};
pub use vaults::VaultsState;

use super::{context::Context, message::Message};
use crate::daemon::client::Client;

pub trait State<C: Client> {
    fn view(&mut self, ctx: &Context<C>) -> Element<Message>;
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message>;
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
    fn load(&self, _ctx: &Context<C>) -> Command<Message> {
        Command::none()
    }
}
