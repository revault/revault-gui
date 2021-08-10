pub mod config;
mod error;
mod menu;
mod message;
mod state;
mod view;

use std::sync::Arc;

use iced::{Clipboard, Color, Command, Element, Subscription};

pub use config::Config;
pub use message::Message;

use menu::Menu;
use state::{
    ChargingState, DepositState, EmergencyState, ManagerHomeState, ManagerSendState, SettingsState,
    StakeholderCreateVaultsState, StakeholderDelegateFundsState, StakeholderHomeState, State,
    VaultsState,
};

use crate::{
    app::view::Context,
    conversion::Converter,
    revault::Role,
    revaultd::{GetInfoResponse, RevaultD},
};

pub struct App {
    config: Config,
    revaultd: Option<Arc<RevaultD>>,
    state: Box<dyn State>,
    context: Context,
}

impl App {
    #[allow(unreachable_patterns)]
    pub fn load_state(&mut self, role: Role, menu: Menu) -> Command<Message> {
        self.context.role = role;
        self.context.menu = menu;
        let revaultd = self.revaultd.clone().unwrap();
        self.state = match self.context.role {
            Role::Manager => match self.context.menu {
                Menu::Deposit => DepositState::new(revaultd).into(),
                Menu::Home => ManagerHomeState::new(revaultd).into(),
                Menu::Vaults => VaultsState::new(revaultd).into(),
                Menu::Send => ManagerSendState::new(revaultd).into(),
                // Manager cannot delegate funds, the user is redirected to the home.
                Menu::DelegateFunds => ManagerHomeState::new(revaultd).into(),
                Menu::Settings => SettingsState::new(revaultd, self.config.clone()).into(),
                _ => unreachable!(),
            },
            Role::Stakeholder => match self.context.menu {
                Menu::Deposit => StakeholderHomeState::new(revaultd).into(),
                Menu::Home => StakeholderHomeState::new(revaultd).into(),
                Menu::Vaults => VaultsState::new(revaultd).into(),
                Menu::CreateVaults => StakeholderCreateVaultsState::new(revaultd).into(),
                Menu::DelegateFunds => StakeholderDelegateFundsState::new(revaultd).into(),
                Menu::Settings => SettingsState::new(revaultd, self.config.clone()).into(),
                Menu::Emergency => EmergencyState::new(revaultd).into(),
                _ => unreachable!(),
            },
        };
        self.state.load()
    }

    /// After the synchronisation process, the UI displays the home panel to the user
    /// according to the role specified in the revaultd configuration.
    fn on_synced(
        &mut self,
        information: GetInfoResponse,
        revaultd: Arc<RevaultD>,
    ) -> Command<Message> {
        let role = if revaultd.config.stakeholder_config.is_some() {
            Role::Stakeholder
        } else {
            Role::Manager
        };

        // The user is both a manager and a stakholder, then role can be modified.
        let edit_role = revaultd.config.stakeholder_config.is_some()
            && revaultd.config.manager_config.is_some();

        self.context = Context::new(
            Converter::new(revaultd.network()),
            revaultd.network(),
            edit_role,
            role,
            Menu::Home,
            information.managers_threshold,
        );
        self.context.network_up = true;
        self.revaultd = Some(revaultd);
        self.load_state(role, Menu::Home)
    }

    pub fn new(config: Config) -> (App, Command<Message>) {
        let state = ChargingState::new(
            config.revaultd_config_path.to_owned(),
            config.revaultd_path.to_owned(),
        );
        let cmd = state.load();
        (
            App {
                config,
                state: std::boxed::Box::new(state),
                revaultd: None,
                context: Context::default(),
            },
            cmd,
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.state.subscription()
    }

    pub fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Synced(info, revaultd) => self.on_synced(info, revaultd),
            Message::ChangeRole(role) => self.load_state(role, self.context.menu.to_owned()),
            Message::Menu(menu) => self.load_state(self.context.role, menu),
            Message::Clipboard(text) => {
                clipboard.write(text);
                Command::none()
            }
            _ => self.state.update(message),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let content = self.state.view(&self.context);
        if let Some(true) = self.config.debug {
            return content.explain(Color::BLACK);
        }

        content
    }
}
