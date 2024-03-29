use std::convert::From;

use iced::{Command, Element};

use super::{cmd::get_deposit_address, State};

use crate::app::{
    context::Context, error::Error, message::Message, view::DepositView, view::LoadingDashboard,
};

/// DepositState handles the deposit process.
/// It gets a deposit address from the revault daemon and
/// give it to its view in order to be rendered.
#[derive(Debug)]
pub enum DepositState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    Loaded {
        address: bitcoin::Address,
        // Error in case of reload failure.
        warning: Option<Error>,

        /// The deposit view is rendering the address.
        view: DepositView,
    },
}

impl DepositState {
    pub fn new() -> Self {
        DepositState::Loading {
            view: LoadingDashboard::default(),
            fail: None,
        }
    }
}

impl State for DepositState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::DepositAddress(res) = message {
                    match res {
                        Ok(address) => {
                            let mut view = DepositView::new();
                            view.load(&address);
                            *self = Self::Loaded {
                                address,
                                warning: None,
                                view,
                            };
                        }
                        Err(e) => *fail = Some(e.into()),
                    };
                }
            }
            Self::Loaded {
                address,
                warning,
                view,
            } => {
                match message {
                    Message::Reload => return self.load(ctx),
                    Message::DepositAddress(res) => {
                        match res {
                            Ok(addr) => {
                                // Address is loaded directly in the view in order to cache the created qrcode.
                                view.load(&address);
                                *address = addr;
                            }
                            Err(e) => *warning = Some(e.into()),
                        }
                    }
                    _ => {}
                }
            }
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref()),
            Self::Loaded {
                warning,
                address,
                view,
            } => view.view(ctx, warning.as_ref(), address),
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        let revaultd = ctx.revaultd.clone();
        Command::perform(get_deposit_address(revaultd), Message::DepositAddress)
    }
}

impl From<DepositState> for Box<dyn State> {
    fn from(s: DepositState) -> Box<dyn State> {
        Box::new(s)
    }
}
