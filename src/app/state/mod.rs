pub mod charging;
mod cmd;
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

pub use charging::ChargingState;
pub use deposit::DepositState;
pub use emergency::EmergencyState;
pub use manager::{ManagerHomeState, ManagerSendState};
pub use settings::SettingsState;
pub use spend_transaction::{SpendTransactionListItem, SpendTransactionState};
pub use stakeholder::{
    StakeholderCreateVaultsState, StakeholderDelegateFundsState, StakeholderHomeState,
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
