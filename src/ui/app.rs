use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use copypasta::{ClipboardContext, ClipboardProvider};
use iced::{executor, Application, Color, Command, Element, Settings, Subscription};
use tracing::error;

use super::menu::Menu;
use super::message::{Message, SignMessage, SpendTxMessage, VaultMessage};
use super::state::{
    ChargingState, DepositState, HistoryState, InstallingState, ManagerHomeState,
    ManagerNetworkState, ManagerSendState, SettingsState, StakeholderACKFundsState,
    StakeholderDelegateFundsState, StakeholderHomeState, StakeholderNetworkState, State,
};

use crate::{conversion::Converter, revault::Role, revaultd::RevaultD, ui::view::Context};

pub struct App {
    config: Config,
    revaultd: Option<Arc<RevaultD>>,
    state: Box<dyn State>,
    clipboard: ClipboardContext,
    context: Context,
}

pub fn run(config: Config) -> Result<(), iced::Error> {
    App::run(Settings::with_flags(config))
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
                Menu::History => HistoryState::new(revaultd).into(),
                Menu::Network => ManagerNetworkState::new(revaultd).into(),
                Menu::Send => ManagerSendState::new(revaultd).into(),
                // Manager cannot delegate funds, the user is redirected to the home.
                Menu::DelegateFunds => ManagerHomeState::new(revaultd).into(),
                Menu::Settings => SettingsState::new(revaultd.config.clone()).into(),
                _ => unreachable!(),
            },
            Role::Stakeholder => match self.context.menu {
                Menu::Deposit => DepositState::new(revaultd).into(),
                Menu::Home => StakeholderHomeState::new(revaultd).into(),
                Menu::History => HistoryState::new(revaultd).into(),
                Menu::Network => StakeholderNetworkState::new(revaultd).into(),
                Menu::ACKFunds => StakeholderACKFundsState::new(revaultd).into(),
                Menu::DelegateFunds => StakeholderDelegateFundsState::new(revaultd).into(),
                Menu::Settings => SettingsState::new(revaultd.config.clone()).into(),
                _ => unreachable!(),
            },
        };
        self.state.load()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn new(config: Config) -> (App, Command<Self::Message>) {
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
                clipboard: ClipboardContext::new().expect("Failed to get clipboard provider"),
                context: Context::default(),
            },
            cmd,
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        self.state.subscription()
    }

    fn title(&self) -> String {
        String::from("Revault GUI")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Install => {
                self.state = InstallingState::new().into();
                self.state.load()
            }
            Message::Synced(revaultd) => {
                self.context = Context::new(
                    Converter::new(revaultd.network()),
                    revaultd.network(),
                    true,
                    Role::Manager,
                    Menu::Home,
                );
                self.context.network_up = true;
                self.revaultd = Some(revaultd);
                self.load_state(Role::Manager, Menu::Home)
            }
            Message::ChangeRole(role) => self.load_state(role, self.context.menu.to_owned()),
            Message::Menu(menu) => self.load_state(self.context.role, menu),
            Message::Clipboard(text)
            | Message::SpendTx(SpendTxMessage::Sign(SignMessage::Clipboard(text)))
            | Message::Vault(VaultMessage::Sign(SignMessage::Clipboard(text))) => {
                if let Err(e) = self.clipboard.set_contents(text) {
                    error!("Failed to set contents to clipboard: {}", e);
                };
                Command::none()
            }
            _ => self.state.update(message),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = self.state.view(&self.context);
        if self.config.debug {
            return content.explain(Color::BLACK);
        }

        content
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub revaultd_config_path: Option<PathBuf>,
    pub revaultd_path: Option<PathBuf>,
    pub debug: bool,
}
