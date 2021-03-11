use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{cmd::get_deposit_address, State};

use crate::revaultd::RevaultD;

use crate::ui::{
    error::Error,
    message::Message,
    view::{Context, DepositView},
};

/// DepositState handles the deposit process.
/// It gets a deposit address from the revault daemon and
/// give it to its view in order to be rendered.
#[derive(Debug)]
pub struct DepositState {
    revaultd: Arc<RevaultD>,
    address: Option<bitcoin::Address>,
    warning: Option<Error>,

    /// The deposit view is rendering the address.
    view: DepositView,
}

impl DepositState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        DepositState {
            revaultd,
            view: DepositView::new(),
            warning: None,
            address: None,
        }
    }
}

impl State for DepositState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DepositAddress(res) => match res {
                Ok(address) => {
                    // Address is loaded directly in the view in order to cache the created qrcode.
                    self.view.load(&address);
                    self.address = Some(address);
                }
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view
            .view(ctx, self.warning.as_ref().into(), self.address.as_ref())
    }

    fn load(&self) -> Command<Message> {
        Command::perform(
            get_deposit_address(self.revaultd.clone()),
            Message::DepositAddress,
        )
    }
}

impl From<DepositState> for Box<dyn State> {
    fn from(s: DepositState) -> Box<dyn State> {
        Box::new(s)
    }
}
