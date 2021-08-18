use std::convert::From;

use iced::{Command, Element};

use super::{cmd::get_deposit_address, State};

use crate::app::{context::Context, error::Error, message::Message, view::DepositView};

/// DepositState handles the deposit process.
/// It gets a deposit address from the revault daemon and
/// give it to its view in order to be rendered.
#[derive(Debug)]
pub struct DepositState {
    address: Option<bitcoin::Address>,
    warning: Option<Error>,

    /// The deposit view is rendering the address.
    view: DepositView,
}

impl DepositState {
    pub fn new() -> Self {
        DepositState {
            view: DepositView::new(),
            warning: None,
            address: None,
        }
    }
}

impl State for DepositState {
    fn update(&mut self, _ctx: &Context, message: Message) -> Command<Message> {
        if let Message::DepositAddress(res) = message {
            match res {
                Ok(address) => {
                    // Address is loaded directly in the view in order to cache the created qrcode.
                    self.view.load(&address);
                    self.address = Some(address);
                }
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            }
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view
            .view(ctx, self.warning.as_ref(), self.address.as_ref())
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::perform(
            get_deposit_address(ctx.revaultd.clone()),
            Message::DepositAddress,
        )
    }
}

impl From<DepositState> for Box<dyn State> {
    fn from(s: DepositState) -> Box<dyn State> {
        Box::new(s)
    }
}
