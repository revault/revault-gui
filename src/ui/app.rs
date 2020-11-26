use std::fmt::Debug;
use std::path::PathBuf;

use iced::{executor, Application, Command, Element, Settings, Text};

use super::state::{charging::StateCharging, Message, State};

pub struct App {
    config: Config,
    state: Box<dyn State>,
}

pub fn run(config: Config) -> Result<(), iced::Error> {
    App::run(Settings::with_flags(config))
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn new(config: Config) -> (App, Command<Self::Message>) {
        (
            App {
                config: config.clone(),
                state: std::boxed::Box::new(StateCharging::new(
                    config.revaultd_config_path,
                    config.debug,
                )),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Revault GUI")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.state.view()
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub revaultd_config_path: Option<PathBuf>,
    pub debug: bool,
}
