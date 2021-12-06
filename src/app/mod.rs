pub mod config;
pub mod context;
pub mod menu;
pub mod message;
pub mod state;

mod error;
mod view;

use iced::{Clipboard, Command, Element, Subscription};
use iced_native::{window, Event};

pub use config::Config;
pub use message::Message;

use menu::Menu;
use state::{
    DepositState, EmergencyState, ManagerHomeState, ManagerSendState, SettingsState,
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
    match context.role {
        Role::Manager => match context.menu {
            Menu::Deposit => DepositState::new().into(),
            Menu::Home => ManagerHomeState::new().into(),
            Menu::Vaults => VaultsState::new().into(),
            Menu::Send => ManagerSendState::new().into(),
            // Manager cannot delegate funds, the user is redirected to the home.
            Menu::DelegateFunds => ManagerHomeState::new().into(),
            Menu::Settings => SettingsState::new(config.clone()).into(),
            _ => unreachable!(),
        },
        Role::Stakeholder => match context.menu {
            Menu::Deposit => DepositState::new().into(),
            Menu::Home => StakeholderHomeState::new().into(),
            Menu::Vaults => VaultsState::new().into(),
            Menu::CreateVaults => StakeholderCreateVaultsState::new().into(),
            Menu::DelegateFunds => StakeholderDelegateVaultsState::new().into(),
            Menu::Settings => SettingsState::new(config.clone()).into(),
            Menu::Emergency => EmergencyState::new().into(),
            _ => unreachable!(),
        },
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
