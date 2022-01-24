use std::convert::From;

use iced::{Command, Element};

use super::State;

use crate::app::config::Config;

use crate::app::{
    context::Context, error::Error, message::Message, state::cmd::get_server_status,
    view::SettingsView,
};

use crate::daemon::{client::Client, model::ServersStatuses};

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

impl<C: Client + Send + Sync + 'static> State<C> for SettingsState {
    fn update(&mut self, _ctx: &Context<C>, message: Message) -> Command<Message> {
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

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        self.view.view(
            ctx,
            self.warning.as_ref(),
            &ctx.revaultd.config,
            self.server_status.clone(),
        )
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_server_status(ctx.revaultd.clone()),
            Message::ServerStatus,
        )])
    }
}

impl<C: Client + Send + Sync + 'static> From<SettingsState> for Box<dyn State<C>> {
    fn from(s: SettingsState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
