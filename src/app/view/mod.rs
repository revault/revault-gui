mod deposit;
mod emergency;
mod history;
mod home;
mod layout;
pub mod manager;
mod settings;
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
pub use settings::SettingsView;
pub use spend_transaction::{SpendTransactionListItemView, SpendTransactionView};
pub use stakeholder::{
    StakeholderCreateVaultsView, StakeholderDelegateVaultsView,
    StakeholderSelecteVaultsToDelegateView,
};
pub use vault::VaultView;
pub use vaults::VaultsView;

use iced::{Column, Element};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message},
    daemon::client::Client,
};

#[derive(Debug)]
pub struct LoadingDashboard {
    dashboard: layout::Dashboard,
}

impl LoadingDashboard {
    pub fn new() -> Self {
        LoadingDashboard {
            dashboard: layout::Dashboard::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        self.dashboard.view(ctx, warning, Column::new())
    }
}

#[derive(Debug)]
pub struct LoadingModal {
    modal: layout::Modal,
}

impl LoadingModal {
    pub fn new() -> Self {
        LoadingModal {
            modal: layout::Modal::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        close_redirect: Menu,
    ) -> Element<'a, Message> {
        self.modal.view(
            ctx,
            warning,
            Column::new(),
            None,
            Message::Menu(close_redirect),
        )
    }
}
