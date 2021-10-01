use std::convert::From;

use iced::{Command, Element};

use super::{cmd::list_vaults, State};

use crate::daemon::{client::Client, model::VaultStatus};

use crate::app::{
    context::Context,
    error::Error,
    menu::Menu,
    message::Message,
    state::cmd,
    view::{EmergencyView, LoadingModal},
};

#[derive(Debug)]
pub enum EmergencyState {
    Loading {
        fail: Option<Error>,
        view: LoadingModal,
    },
    Loaded {
        view: EmergencyView,

        vaults_number: usize,
        funds_amount: u64,

        warning: Option<Error>,

        processing: bool,
        success: bool,
    },
}

impl EmergencyState {
    pub fn new() -> Self {
        Self::Loading {
            view: LoadingModal::new(),
            fail: None,
        }
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for EmergencyState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match self {
                Self::Loading { fail, .. } => match res {
                    Ok(vaults) => {
                        *self = Self::Loaded {
                            view: EmergencyView::new(),
                            vaults_number: vaults.len(),
                            funds_amount: vaults
                                .into_iter()
                                .fold(0, |acc, vault| acc + vault.amount),
                            warning: None,
                            processing: false,
                            success: false,
                        };
                    }
                    Err(e) => *fail = Error::from(e).into(),
                },
                Self::Loaded {
                    vaults_number,
                    funds_amount,
                    warning,
                    ..
                } => match res {
                    Ok(vaults) => {
                        *vaults_number = vaults.len();
                        *funds_amount = vaults.into_iter().fold(0, |acc, vault| acc + vault.amount);
                        *warning = None;
                    }
                    Err(e) => *warning = Error::from(e).into(),
                },
            },
            Message::Emergency => {
                if let Self::Loaded {
                    processing,
                    warning,
                    ..
                } = self
                {
                    *processing = true;
                    *warning = None;
                    return Command::perform(
                        cmd::emergency(ctx.revaultd.clone()),
                        Message::EmergencyBroadcasted,
                    );
                }
            }
            Message::EmergencyBroadcasted(res) => {
                if let Self::Loaded {
                    processing,
                    warning,
                    success,
                    ..
                } = self
                {
                    *processing = false;
                    if let Err(e) = res {
                        *warning = Some(Error::RevaultDError(e));
                    } else {
                        *success = true;
                    }
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref(), Menu::Home),
            Self::Loaded {
                view,
                funds_amount,
                warning,
                processing,
                success,
                vaults_number,
            } => view.view(
                ctx,
                *vaults_number,
                *funds_amount,
                warning.as_ref(),
                *processing,
                *success,
            ),
        }
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(
                ctx.revaultd.clone(),
                Some(&[
                    VaultStatus::Secured,
                    VaultStatus::Active,
                    VaultStatus::Activating,
                    VaultStatus::Unvaulting,
                    VaultStatus::Unvaulted,
                ]),
                None,
            ),
            Message::Vaults,
        )])
    }
}

impl<C: Client + Send + Sync + 'static> From<EmergencyState> for Box<dyn State<C>> {
    fn from(s: EmergencyState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
