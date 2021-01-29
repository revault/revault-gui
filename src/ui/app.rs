use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{executor, Application, Color, Command, Element, Settings, Subscription};

use super::message::{Menu, Message, Role};
use super::state::{
    ChargingState, HistoryState, InstallingState, ManagerHomeState, ManagerNetworkState,
    ManagerSendState, StakeholderHomeState, StakeholderNetworkState, State,
};

use crate::revaultd::RevaultD;
use crate::ui::message::Context;

pub struct App {
    config: Config,
    revaultd: Option<Arc<RevaultD>>,
    state: Box<dyn State>,

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
        self.state = match self.context.role {
            Role::Manager => match self.context.menu {
                Menu::Home => ManagerHomeState::new(self.revaultd.clone().unwrap()).into(),
                Menu::History => HistoryState::new(self.revaultd.clone().unwrap()).into(),
                Menu::Network => ManagerNetworkState::new(self.revaultd.clone().unwrap()).into(),
                Menu::Send => ManagerSendState::new(self.revaultd.clone().unwrap()).into(),
                _ => unreachable!(),
            },
            Role::Stakeholder => match self.context.menu {
                Menu::Home => StakeholderHomeState::new(self.revaultd.clone().unwrap()).into(),
                Menu::History => HistoryState::new(self.revaultd.clone().unwrap()).into(),
                Menu::Network => {
                    StakeholderNetworkState::new(self.revaultd.clone().unwrap()).into()
                }
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
        let state = ChargingState::new(config.revaultd_config_path.to_owned());
        let cmd = state.load();
        (
            App {
                config,
                state: std::boxed::Box::new(state),
                revaultd: None,
                context: Context::new(true, Role::Manager, Menu::Home),
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
                self.context.network = revaultd.network();
                self.context.network_up = true;
                self.revaultd = Some(revaultd);
                self.load_state(Role::Manager, Menu::Home)
            }
            Message::ChangeRole(role) => self.load_state(role, self.context.menu.to_owned()),
            Message::Menu(menu) => self.load_state(self.context.role, menu),
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
    pub debug: bool,
}
