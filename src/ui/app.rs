use std::fmt::Debug;
use std::path::PathBuf;

use iced::{executor, Application, Color, Command, Element, Settings, Subscription};

use super::message::Message;
use super::state::{
    charging::connect, charging::ChargingState, installing::InstallingState, manager::ManagerState,
    State,
};

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
                state: std::boxed::Box::new(ChargingState::new(
                    config.revaultd_config_path.to_owned(),
                )),
            },
            Command::perform(connect(config.revaultd_config_path), Message::Connected),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        self.state.subscription()
    }

    fn title(&self) -> String {
        String::from("Revault GUI")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Install => {
                self.state = Box::new(InstallingState {});
            }
            Message::Synced(revaultd) => {
                self.state = Box::new(ManagerState::new(revaultd));
            }
            _ => return self.state.update(message),
        }
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = self.state.view();
        if self.config.debug {
            return content.explain(Color::BLACK);
        }

        content
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub revaultd_config_path: Option<PathBuf>,
    pub debug: bool,
}
