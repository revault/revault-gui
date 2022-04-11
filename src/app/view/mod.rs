mod deposit;
mod emergency;
mod history;
mod home;
mod layout;
pub mod manager;
mod revault;
pub mod settings;
mod sidebar;
pub mod sign;
pub mod spend_transaction;
pub mod stakeholder;
pub mod vault;
mod vaults;
mod warning;

pub use deposit::DepositView;
pub use emergency::{EmergencyTriggeredView, EmergencyView};
pub use history::{HistoryEventListItemView, HistoryEventView, HistoryView};
pub use home::{ManagerHomeView, StakeholderHomeView};
pub use revault::{RevaultSelectVaultsView, RevaultSuccessView, RevaultVaultListItemView};
pub use settings::SettingsView;
pub use spend_transaction::{SpendTransactionListItemView, SpendTransactionView};
pub use stakeholder::{
    StakeholderCreateVaultsView, StakeholderDelegateVaultsView,
    StakeholderSelecteVaultsToDelegateView,
};
pub use vault::VaultView;
pub use vaults::VaultsView;

use iced::{Column, Element};

use crate::app::{context::Context, error::Error, message::Message};

#[derive(Debug, Default)]
pub struct LoadingDashboard {
    dashboard: layout::Dashboard,
}

impl LoadingDashboard {
    pub fn view<'a>(&'a mut self, ctx: &Context, warning: Option<&Error>) -> Element<'a, Message> {
        self.dashboard.view(ctx, warning, Column::new())
    }
}

#[derive(Debug, Default)]
pub struct LoadingModal {
    modal: layout::Modal,
}

impl LoadingModal {
    pub fn view<'a, T: Into<Message>>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        close_redirect: T,
    ) -> Element<'a, Message> {
        self.modal
            .view(ctx, warning, Column::new(), None, close_redirect.into())
    }
}
