pub mod cmd;
mod deposit;
mod emergency;
pub mod history;
pub mod manager;
mod revault;
mod settings;
mod sign;
mod spend_transaction;
pub mod stakeholder;
mod vault;
mod vaults;

use iced::{Command, Element, Subscription};

pub use deposit::DepositState;
pub use emergency::EmergencyState;
pub use history::HistoryState;
pub use manager::{
    ManagerCreateSendTransactionState, ManagerHomeState, ManagerImportSendTransactionState,
    ManagerSendState,
};
pub use revault::RevaultVaultsState;
pub use settings::SettingsState;
pub use spend_transaction::{SpendTransactionListItem, SpendTransactionState};
pub use stakeholder::{
    StakeholderCreateVaultsState, StakeholderDelegateVaultsState, StakeholderHomeState,
};
pub use vaults::VaultsState;

use super::{context::Context, message::Message};

pub trait State {
    fn view(&mut self, ctx: &Context) -> Element<Message>;
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message>;
    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
    fn load(&self, _ctx: &Context) -> Command<Message> {
        Command::none()
    }
}
