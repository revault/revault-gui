use std::convert::From;

use iced::{Command, Element};

use super::State;

use crate::app::config::Config;

use crate::app::{
    context::Context, error::Error, message::Message, state::cmd::get_server_status,
    view::SettingsView,
};

use crate::daemon::model::ServersStatuses;

#[derive(Debug)]
pub struct SettingsState {
    _config: Config,
    warning: Option<Error>,
    view: SettingsView,
    server_status: Option<ServersStatuses>,
}

impl SettingsState {
    pub fn new(config: Config) -> Self {
        SettingsState {
            _config: config,
            view: SettingsView::new(),
            warning: None,
            server_status: None,
        }
    }
}

impl State for SettingsState {
    fn update(&mut self, _ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::ServerStatus(s) => {
                match s {
                    Ok(server_status) => self.server_status = Some(server_status),
                    Err(e) => self.warning = Error::from(e).into(),
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
            &ctx.config.daemon,
            self.server_status.clone(),
        )
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_server_status(ctx.revaultd.clone()),
            Message::ServerStatus,
        )])
    }
}

impl From<SettingsState> for Box<dyn State> {
    fn from(s: SettingsState) -> Box<dyn State> {
        Box::new(s)
    }
}
