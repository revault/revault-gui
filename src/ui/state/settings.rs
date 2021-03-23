use std::convert::From;

use iced::{Command, Element};

use super::State;

use crate::revaultd::config::Config;

use crate::ui::{
    error::Error,
    message::Message,
    view::{Context, SettingsView},
};

#[derive(Debug)]
pub struct SettingsState {
    view: SettingsView,
    warning: Option<Error>,
    config: Config,
}

impl SettingsState {
    pub fn new(config: Config) -> Self {
        SettingsState {
            view: SettingsView::new(),
            config,
            warning: None,
        }
    }
}

impl State for SettingsState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view
            .view(ctx, self.warning.as_ref(), self.config.clone())
    }
}

impl From<SettingsState> for Box<dyn State> {
    fn from(s: SettingsState) -> Box<dyn State> {
        Box::new(s)
    }
}
