use std::convert::From;

use iced::{Command, Element};

use super::State;

use crate::app::config::Config;

use crate::app::{
    context::Context,
    error::Error,
    message::Message,
    state::cmd::{get_blockheight, get_server_status},
    view::SettingsView,
};

use crate::revaultd::ServerStatusResponse;

#[derive(Debug)]
pub struct SettingsState {
    blockheight: u64,
    config: Config,
    warning: Option<Error>,
    view: SettingsView,
    server_status: Option<ServerStatusResponse>,
}

impl SettingsState {
    pub fn new(config: Config) -> Self {
        SettingsState {
            blockheight: 0,
            config,
            view: SettingsView::new(),
            warning: None,
            server_status: None,
        }
    }
}

impl State for SettingsState {
    fn update(&mut self, _ctx: &Context, message: Message) -> Command<Message> {
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
            self.blockheight,
            &ctx.revaultd.config,
            self.server_status.clone(),
        )
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(ctx.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                get_server_status(ctx.revaultd.clone()),
                Message::ServerStatus,
            ),
        ])
    }
}

impl From<SettingsState> for Box<dyn State> {
    fn from(s: SettingsState) -> Box<dyn State> {
        Box::new(s)
    }
}
