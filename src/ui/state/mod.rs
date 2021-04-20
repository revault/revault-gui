pub mod charging;
mod cmd;
mod deposit;
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
pub use manager::{ManagerHomeState, ManagerNetworkState, ManagerSendState};
pub use settings::SettingsState;
pub use spend_transaction::{SpendTransactionListItem, SpendTransactionState};
pub use stakeholder::{
    StakeholderACKFundsState, StakeholderDelegateFundsState, StakeholderHomeState,
    StakeholderNetworkState,
};
pub use vaults::VaultsState;

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
