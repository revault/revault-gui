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

use iced::{scrollable, Column, Container, Element, Length, Row};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message},
    daemon::client::Client,
    ui::component::{button, card, navbar, scroll, text::Text, ContainerBackgroundStyle},
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

#[derive(Debug)]
pub struct LoadingModal {
    scroll: scrollable::State,
    close_button: iced::button::State,
}

impl LoadingModal {
    pub fn new() -> Self {
        LoadingModal {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        _ctx: &Context<C>,
        warning: Option<&Error>,
        close_redirect: Menu,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .push(
                Row::new().push(Column::new().width(Length::Fill)).push(
                    Container::new(
                        button::close_button(&mut self.close_button)
                            .on_press(Message::Menu(close_redirect)),
                    )
                    .width(Length::Shrink),
                ),
            )
            .spacing(50);

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(Text::new(&format!(
                "{}",
                error
            )))))
        }

        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}
