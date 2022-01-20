pub mod config;
pub mod context;
pub mod menu;
pub mod message;
pub mod state;

mod error;
mod view;

use std::time::Duration;

use iced::{time, Clipboard, Command, Element, Subscription};
use iced_native::{window, Event};

pub use config::Config;
pub use message::Message;

use menu::Menu;
use state::{
    DepositState, EmergencyState, HistoryState, ManagerHomeState, ManagerSendState, SettingsState,
    StakeholderCreateVaultsState, StakeholderDelegateVaultsState, StakeholderHomeState, State,
    VaultsState,
};

use crate::{app::context::Context, daemon::client::Client, revault::Role};

pub struct App<C: Client + Send + Sync + 'static> {
    should_exit: bool,
    config: Config,
    state: Box<dyn State<C>>,
    context: Context<C>,
}

#[allow(unreachable_patterns)]
pub fn new_state<C: Client + Send + Sync + 'static>(
    context: &Context<C>,
    config: &Config,
) -> Box<dyn State<C>> {
    match (context.role, &context.menu) {
        (_, Menu::Deposit) => DepositState::new().into(),
        (_, Menu::History) => HistoryState::new().into(),
        (_, Menu::Vaults) => VaultsState::new().into(),
        (_, Menu::Settings) => SettingsState::new(config.clone()).into(),
        (Role::Stakeholder, Menu::Home) => StakeholderHomeState::new().into(),
        (Role::Stakeholder, Menu::CreateVaults) => StakeholderCreateVaultsState::new().into(),
        (Role::Stakeholder, Menu::DelegateFunds) => StakeholderDelegateVaultsState::new().into(),
        (Role::Stakeholder, Menu::Emergency) => EmergencyState::new().into(),
        (Role::Manager, Menu::Home) => ManagerHomeState::new().into(),
        (Role::Manager, Menu::Send) => ManagerSendState::new().into(),

        // If menu is not available for the role, the user is redirected to Home.
        (Role::Stakeholder, _) => StakeholderHomeState::new().into(),
        (Role::Manager, _) => ManagerHomeState::new().into(),
    }
}

impl<C: Client + Send + Sync + 'static> App<C> {
    pub fn new(context: Context<C>, config: Config) -> (App<C>, Command<Message>) {
        let state = new_state(&context, &config);
        let cmd = state.load(&context);
        (
            Self {
                should_exit: false,
                config,
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

    pub fn stop(&mut self) -> Command<Message> {
        self.should_exit = true;
        if self.context.internal_daemon {
            return Command::perform(
                state::cmd::stop(self.context.revaultd.clone()),
                Message::StoppingDaemon,
            );
        }
        Command::none()
    }

    pub fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
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
            Message::ChangeRole(role) => {
                self.context.role = role;
                self.state = new_state(&self.context, &self.config);
                self.state.load(&self.context)
            }
            Message::Menu(menu) => {
                self.context.menu = menu;
                self.state = new_state(&self.context, &self.config);
                self.state.load(&self.context)
            }
            Message::Clipboard(text) => {
                clipboard.write(text);
                Command::none()
            }
            Message::Event(Event::Window(window::Event::CloseRequested)) => self.stop(),
            _ => self.state.update(&self.context, message),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        self.state.view(&self.context)
    }
}
