use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{executor, Application, Color, Command, Element, Settings, Subscription};

use super::message::{Menu, Message, Role};
use super::state::{
    ChargingState, InstallingState, ManagerHistoryState, ManagerHomeState, StakeholderState, State,
};

use crate::revaultd::RevaultD;

pub struct App {
    config: Config,
    revaultd: Option<Arc<RevaultD>>,
    state: Box<dyn State>,

    role: Role,
    menu: Menu,
}

pub fn run(config: Config) -> Result<(), iced::Error> {
    App::run(Settings::with_flags(config))
}

impl App {
    pub fn load_state(&mut self, role: Role, menu: Menu) -> Command<Message> {
        self.role = role;
        self.menu = menu;
        self.state = match self.role {
            Role::Manager => match self.menu {
                Menu::Home => ManagerHomeState::new(self.revaultd.clone().unwrap()).into(),
                Menu::History => ManagerHistoryState::new(self.revaultd.clone().unwrap()).into(),
            },
            Role::Stakeholder => StakeholderState::new(self.revaultd.clone().unwrap()).into(),
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
                role: Role::Manager,
                menu: Menu::Home,
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
                self.revaultd = Some(revaultd);
                self.load_state(Role::Manager, Menu::Home)
            }
            Message::ChangeRole(role) => self.load_state(role, self.menu.to_owned()),
            Message::Menu(menu) => self.load_state(self.role, menu),
            _ => self.state.update(message),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = self.state.view();
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
