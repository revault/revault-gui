pub mod config;
mod context;
mod error;
mod menu;
mod message;
mod state;
mod view;

use iced::{Clipboard, Command, Element, Subscription};

pub use config::Config;
pub use message::Message;

use menu::Menu;
use state::{
    ChargingState, DepositState, EmergencyState, ManagerHomeState, ManagerSendState, SettingsState,
    StakeholderCreateVaultsState, StakeholderDelegateFundsState, StakeholderHomeState, State,
    VaultsState,
};

use crate::{app::context::Context, conversion::Converter, revault::Role};

pub enum App {
    Charging(ChargingState),
    Running {
        config: Config,
        state: Box<dyn State>,
        context: Context,
    },
}

#[allow(unreachable_patterns)]
pub fn new_state(context: &Context, config: &Config) -> Box<dyn State> {
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
            Menu::Deposit => StakeholderHomeState::new().into(),
            Menu::Home => StakeholderHomeState::new().into(),
            Menu::Vaults => VaultsState::new().into(),
            Menu::CreateVaults => StakeholderCreateVaultsState::new().into(),
            Menu::DelegateFunds => StakeholderDelegateFundsState::new().into(),
            Menu::Settings => SettingsState::new(config.clone()).into(),
            Menu::Emergency => EmergencyState::new().into(),
            _ => unreachable!(),
        },
    }
}

impl App {
    pub fn new(config: Config) -> (App, Command<Message>) {
        let state = ChargingState::new(config);
        let cmd = state.load();
        (Self::Charging(state), cmd)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match self {
            Self::Running { state, .. } => state.subscription(),
            Self::Charging(_) => Subscription::none(),
        }
    }

    pub fn update(&mut self, message: Message, clipboard: &mut Clipboard) -> Command<Message> {
        match self {
            Self::Running {
                state,
                context,
                config,
            } => match message {
                Message::ChangeRole(role) => {
                    context.role = role;
                    *state = new_state(context, config);
                    state.load(context)
                }
                Message::Menu(menu) => {
                    context.menu = menu;
                    *state = new_state(context, config);
                    state.load(context)
                }
                Message::Clipboard(text) => {
                    clipboard.write(text);
                    Command::none()
                }
                _ => state.update(context, message),
            },
            Self::Charging(state) => match message {
                // After the synchronisation process, the UI displays the home panel to the user
                // according to the role specified in the revaultd configuration.
                Message::Synced(info, revaultd) => {
                    let config = state.gui_config.clone();
                    let role = if revaultd.config.stakeholder_config.is_some() {
                        Role::Stakeholder
                    } else {
                        Role::Manager
                    };

                    // The user is both a manager and a stakholder, then role can be modified.
                    let edit_role = revaultd.config.stakeholder_config.is_some()
                        && revaultd.config.manager_config.is_some();

                    let context = Context::new(
                        revaultd.clone(),
                        Converter::new(revaultd.network()),
                        revaultd.network(),
                        edit_role,
                        role,
                        Menu::Home,
                        info.managers_threshold,
                    );

                    let state = new_state(&context, &config);
                    let cmd = state.load(&context);
                    *self = Self::Running {
                        context,
                        state,
                        config,
                    };
                    cmd
                }
                _ => state.update(message),
            },
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self {
            Self::Charging(state) => state.view(),
            Self::Running { state, context, .. } => state.view(&context),
        }
    }
}
