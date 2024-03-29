pub mod config;
pub mod context;
pub mod menu;
pub mod message;
pub mod state;

mod error;
mod view;

use std::sync::Arc;
use std::time::Duration;

use iced::{clipboard, time, Command, Element, Subscription};
use iced_native::{window, Event};

pub use config::Config;
pub use message::{Message, SettingsMessage};

use menu::Menu;
use state::{
    DepositState, EmergencyState, HistoryState, ManagerCreateSendTransactionState,
    ManagerHomeState, ManagerImportSendTransactionState, ManagerSendState, RevaultVaultsState,
    SettingsState, StakeholderCreateVaultsState, StakeholderDelegateVaultsState,
    StakeholderHomeState, State, VaultsState,
};

use crate::{app::context::Context, revault::Role};

pub struct App {
    should_exit: bool,
    state: Box<dyn State>,
    context: Context,
}

pub fn new_state(context: &Context) -> Box<dyn State> {
    match (context.role, &context.menu) {
        (_, Menu::Deposit) => DepositState::new().into(),
        (_, Menu::History) => HistoryState::new().into(),
        (_, Menu::Vaults(menu)) => VaultsState::new(menu).into(),
        (_, Menu::RevaultVaults) => RevaultVaultsState::default().into(),
        (_, Menu::Settings) => SettingsState::new(context).into(),
        (Role::Stakeholder, Menu::Home) => StakeholderHomeState::new().into(),
        (Role::Stakeholder, Menu::CreateVaults) => StakeholderCreateVaultsState::new().into(),
        (Role::Stakeholder, Menu::DelegateFunds) => StakeholderDelegateVaultsState::new().into(),
        (Role::Stakeholder, Menu::Emergency) => EmergencyState::new().into(),
        (Role::Manager, Menu::Home) => ManagerHomeState::new().into(),
        (Role::Manager, Menu::Send) => ManagerSendState::new().into(),
        (Role::Manager, Menu::CreateSpend) => ManagerCreateSendTransactionState::new().into(),
        (Role::Manager, Menu::ImportSpend) => ManagerImportSendTransactionState::new().into(),

        // If menu is not available for the role, the user is redirected to Home.
        (Role::Stakeholder, _) => StakeholderHomeState::new().into(),
        (Role::Manager, _) => ManagerHomeState::new().into(),
    }
}

impl App {
    pub fn new(context: Context) -> (App, Command<Message>) {
        let state = new_state(&context);
        let cmd = state.load(&context);
        (
            Self {
                should_exit: false,
                state,
                context,
            },
            cmd,
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            iced_native::subscription::events().map(Message::Event),
            time::every(Duration::from_secs(30)).map(|_| Message::Tick),
            self.state.subscription(),
        ])
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn stop(&mut self) {
        log::info!("Close requested");
        if !self.context.revaultd.is_external() {
            log::info!("Stopping internal daemon...");
            if let Some(d) = Arc::get_mut(&mut self.context.revaultd) {
                d.stop().expect("Daemon is internal");
                log::info!("Internal daemon stopped");
                self.should_exit = true;
            }
        } else {
            self.should_exit = true;
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick => {
                let revaultd = self.context.revaultd.clone();
                Command::perform(
                    async move { revaultd.get_info().map(|res| res.blockheight) },
                    Message::BlockHeight,
                )
            }
            Message::BlockHeight(res) => {
                if let Ok(blockheight) = res {
                    self.context.blockheight = blockheight;
                }
                Command::none()
            }
            Message::LoadDaemonConfig(cfg) => {
                let res = self.context.load_daemon_config(cfg);
                self.update(Message::DaemonConfigLoaded(res))
            }
            Message::ChangeRole(role) => {
                self.context.role = role;
                self.state = new_state(&self.context);
                self.state.load(&self.context)
            }
            Message::Menu(menu) => {
                self.context.menu = menu;
                self.state = new_state(&self.context);
                self.state.load(&self.context)
            }
            Message::Clipboard(text) => clipboard::write(text),
            Message::Event(Event::Window(window::Event::CloseRequested)) => {
                self.stop();
                Command::none()
            }
            _ => self.state.update(&self.context, message),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        self.state.view(&self.context)
    }
}
