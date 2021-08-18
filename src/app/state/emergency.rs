use std::convert::From;

use iced::{Command, Element};

use super::{cmd::list_vaults, State};

use crate::revaultd::model::VaultStatus;

use crate::app::{
    context::Context, error::Error, message::Message, state::cmd, view::EmergencyView,
};

#[derive(Debug)]
pub struct EmergencyState {
    view: EmergencyView,

    vaults_number: usize,
    funds_amount: u64,

    warning: Option<Error>,

    /// loading is true until Message::Vaults is handled
    loading: bool,
    processing: bool,
    success: bool,
}

impl EmergencyState {
    pub fn new() -> Self {
        EmergencyState {
            view: EmergencyView::new(),
            vaults_number: 0,
            funds_amount: 0,
            warning: None,
            loading: true,
            processing: false,
            success: false,
        }
    }
}

impl State for EmergencyState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => {
                    self.loading = false;
                    self.vaults_number = vaults.len();
                    self.funds_amount = vaults.into_iter().fold(0, |acc, vault| acc + vault.amount);
                }
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Emergency => {
                self.processing = true;
                self.warning = None;
                return Command::perform(
                    cmd::emergency(ctx.revaultd.clone()),
                    Message::EmergencyBroadcasted,
                );
            }
            Message::EmergencyBroadcasted(res) => {
                self.processing = false;
                if let Err(e) = res {
                    self.warning = Some(Error::RevaultDError(e));
                } else {
                    self.success = true;
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            self.vaults_number,
            self.funds_amount,
            self.warning.as_ref(),
            self.loading,
            self.processing,
            self.success,
        )
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
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

impl From<EmergencyState> for Box<dyn State> {
    fn from(s: EmergencyState) -> Box<dyn State> {
        Box::new(s)
    }
}
