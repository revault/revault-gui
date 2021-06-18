use std::error::Error;
use std::path::PathBuf;

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

mod app;
mod conversion;
mod installer;
mod revault;
mod revaultd;
mod ui;

use app::{
    config::{ConfigError, DEFAULT_FILE_NAME},
    App,
};
use installer::Installer;
use revaultd::config::default_datadir;

enum Args {
    ConfigPath(PathBuf),
    DatadirPath(PathBuf),
    None,
}

fn parse_args(args: Vec<String>) -> Result<Args, Box<dyn Error>> {
    if args.len() == 1 {
        return Ok(Args::None);
    }

    if args.len() == 3 {
        if args[1] == "--conf" {
            return Ok(Args::ConfigPath(PathBuf::from(args[2].to_owned())));
        } else if args[1] == "--datadir" {
            return Ok(Args::DatadirPath(PathBuf::from(args[2].to_owned())));
        }
    }

    println!("Usage:\n'--conf <configuration file path>'\n'--datadir <datadir path>'");
    Err(format!("Unknown arguments '{:?}'.", args).into())
}

fn log_level_from_config(config: &app::Config) -> Result<EnvFilter, Box<dyn Error>> {
    if let Some(level) = &config.log_level {
        match level.as_ref() {
            "info" => EnvFilter::try_new("revault_gui=info").map_err(|e| e.into()),
            "debug" => EnvFilter::try_new("revault_gui=debug").map_err(|e| e.into()),
            "trace" => EnvFilter::try_new("revault_gui=trace").map_err(|e| e.into()),
            _ => Err(format!("Unknown loglevel '{:?}'.", level).into()),
        }
    } else if let Some(true) = config.debug {
        EnvFilter::try_new("revault_gui=debug").map_err(|e| e.into())
    } else {
        EnvFilter::try_new("revault_gui=info").map_err(|e| e.into())
    }
}

pub enum GUI {
    Installer(Installer),
    App(App),
}

#[derive(Debug, Clone)]
pub enum Message {
    Install(installer::Message),
    Run(app::Message),
}

pub enum Config {
    Run(app::Config),
    Install(PathBuf),
}

impl Application for GUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn title(&self) -> String {
        match self {
            Self::Installer(_) => String::from("Revault Installer"),
            Self::App(_) => String::from("Revault GUI"),
        }
    }

    fn new(config: Config) -> (GUI, Command<Self::Message>) {
        match config {
            Config::Install(path) => {
                let (install, command) = Installer::new(path);
                (GUI::Installer(install), command.map(Message::Install))
            }
            Config::Run(cfg) => {
                let (application, command) = App::new(cfg);
                (GUI::App(application), command.map(Message::Run))
            }
        }
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        if let Message::Install(installer::Message::Exit(path)) = message {
            let cfg = app::Config::from_file(&path).unwrap();
            let (application, command) = App::new(cfg);
            *self = GUI::App(application);
            return command.map(Message::Run);
        }
        match (self, message) {
            (Self::Installer(i), Message::Install(msg)) => {
                i.update(msg, clipboard).map(Message::Install)
            }
            (Self::App(i), Message::Run(msg)) => i.update(msg, clipboard).map(Message::Run),
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Self::Installer(v) => v.subscription().map(Message::Install),
            Self::App(v) => v.subscription().map(Message::Run),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self {
            Self::Installer(v) => v.view().map(Message::Install),
            Self::App(v) => v.view().map(Message::Run),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect();

    let config = match parse_args(args)? {
        Args::ConfigPath(path) => Config::Run(app::Config::from_file(&path)?),
        Args::None => {
            let path = app::Config::default_path()
                .map_err(|e| format!("Failed to find revault GUI config: {}", e))?;

            match app::Config::from_file(&path) {
                Ok(cfg) => Config::Run(cfg),
                Err(ConfigError::NotFound) => {
                    let default_datadir_path =
                        default_datadir().expect("Unexpected filesystem error");
                    Config::Install(default_datadir_path)
                }
                Err(e) => {
                    return Err(format!("Failed to read configuration file: {}", e).into());
                }
            }
        }
        Args::DatadirPath(datadir_path) => {
            let mut path = datadir_path.clone();
            path.push(DEFAULT_FILE_NAME);
            match app::Config::from_file(&path) {
                Ok(cfg) => Config::Run(cfg),
                Err(ConfigError::NotFound) => Config::Install(datadir_path),
                Err(e) => {
                    return Err(format!("Failed to read configuration file: {}", e).into());
                }
            }
        }
    };

    let level = if let Config::Run(cfg) = &config {
        log_level_from_config(&cfg)?
    } else {
        EnvFilter::try_new("revault_gui=info").unwrap()
    };

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(level)
            .finish(),
    )?;

    if let Err(e) = GUI::run(Settings::with_flags(config)) {
        return Err(format!("Failed to launch UI: {}", e).into());
    };
    Ok(())
}
