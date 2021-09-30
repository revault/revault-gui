mod deposit;
mod emergency;
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

pub use deposit::DepositView;
pub use emergency::EmergencyView;
pub use home::{ManagerHomeView, StakeholderHomeView};
pub use settings::SettingsView;
pub use spend_transaction::{SpendTransactionListItemView, SpendTransactionView};
pub use stakeholder::{StakeholderCreateVaultsView, StakeholderDelegateFundsView};
pub use vault::VaultView;
pub use vaults::VaultsView;

use iced::{scrollable, Column, Container, Element};

use crate::{
    app::{context::Context, error::Error, message::Message},
    daemon::client::Client,
    ui::component::{navbar, scroll},
};

#[derive(Debug)]
pub struct LoadingDashboard {
    sidebar: sidebar::Sidebar,
    scroll: scrollable::State,
}

impl LoadingDashboard {
    pub fn new() -> Self {
        LoadingDashboard {
            sidebar: sidebar::Sidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(Column::new()),
            ))),
        )
        .into()
    }
}
