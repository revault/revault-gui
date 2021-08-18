use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::State;

use crate::app::config::Config;
use crate::revaultd::RevaultD;

use crate::app::{
    context::Context, error::Error, message::Message, state::cmd::get_blockheight,
    view::SettingsView,
};

#[derive(Debug)]
pub struct SettingsState {
    blockheight: u64,
    revaultd: Arc<RevaultD>,
    config: Config,
    warning: Option<Error>,
    view: SettingsView,
}

impl SettingsState {
    pub fn new(revaultd: Arc<RevaultD>, config: Config) -> Self {
        SettingsState {
            blockheight: 0,
            revaultd,
            config,
            view: SettingsView::new(),
            warning: None,
        }
    }
}

impl State for SettingsState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BlockHeight(b) => {
                match b {
                    Ok(height) => {
                        self.blockheight = height.into();
                    }
                    Err(e) => {
                        self.warning = Error::from(e).into();
                    }
                };
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            self.warning.as_ref(),
            self.blockheight,
            &self.revaultd.config,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight)
    }
}

impl From<SettingsState> for Box<dyn State> {
    fn from(s: SettingsState) -> Box<dyn State> {
        Box::new(s)
    }
}
