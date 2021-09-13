use std::error::Error;
use std::path::PathBuf;

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

mod app;
mod conversion;
mod hw;
mod installer;
mod loader;
mod revault;
mod revaultd;
mod ui;

use app::{
    config::{ConfigError, DEFAULT_FILE_NAME},
    context::Context,
    menu::Menu,
    App,
};
use conversion::Converter;
use installer::Installer;
use loader::Loader;
use revault::Role;
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
    Loader(Loader),
    App(App),
}

#[derive(Debug)]
pub enum Message {
    Install(installer::Message),
    Load(loader::Message),
    Run(app::Message),
}

pub enum Config {
    Run(app::Config),
    Install(PathBuf, Option<PathBuf>),
}

impl Application for GUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn title(&self) -> String {
        match self {
            Self::Installer(_) => String::from("Revault Installer"),
            Self::App(_) => String::from("Revault GUI"),
            Self::Loader(_) => String::from("Revault"),
        }
    }

    fn new(config: Config) -> (GUI, Command<Self::Message>) {
        match config {
            Config::Install(config_path, revaultd_path) => {
                let (install, command) = Installer::new(config_path, revaultd_path);
                (GUI::Installer(install), command.map(Message::Install))
            }
            Config::Run(cfg) => {
                let (loader, command) = Loader::new(cfg);
                (GUI::Loader(loader), command.map(Message::Load))
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
            let (loader, command) = Loader::new(cfg);
            *self = GUI::Loader(loader);
            return command.map(Message::Load);
        }

        if let Message::Load(loader::Message::Synced(info, revaultd)) = message {
            if let GUI::Loader(loader) = self {
                let config = loader.gui_config.clone();
                let role = if revaultd.config.stakeholder_config.is_some() {
                    Role::Stakeholder
                } else {
                    Role::Manager
                };

                // The user is both a manager and a stakholder, then role can be modified.
                let edit_role = revaultd.config.stakeholder_config.is_some()
                    && revaultd.config.manager_config.is_some();

                let converter = Converter::new(revaultd.network());
                let network = revaultd.network();

                let context = Context::new(
                    revaultd,
                    converter,
                    network,
                    edit_role,
                    role,
                    Menu::Home,
                    info.managers_threshold,
                );

                let (app, command) = App::new(context, config);
                *self = GUI::App(app);
                return command.map(Message::Run);
            }
            return Command::none();
        }

        match (self, message) {
            (Self::Installer(i), Message::Install(msg)) => {
                i.update(msg, clipboard).map(Message::Install)
            }
            (Self::Loader(loader), Message::Load(msg)) => loader.update(msg).map(Message::Load),
            (Self::App(i), Message::Run(msg)) => i.update(msg, clipboard).map(Message::Run),
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Self::Installer(v) => v.subscription().map(Message::Install),
            Self::App(v) => v.subscription().map(Message::Run),
            _ => Subscription::none(),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match self {
            Self::Installer(v) => v.view().map(Message::Install),
            Self::App(v) => v.view().map(Message::Run),
            Self::Loader(v) => v.view().map(Message::Load),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let revaultd_path = match std::env::var("REVAULTD_PATH") {
        Ok(p) => Some(
            PathBuf::from(p)
                .canonicalize()
                .map_err(|e| ConfigError::Unexpected(format!("REVAULTD_PATH: {}", e)))?,
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            println!("Error: REVAULTD_CONF path has a wrong unicode format");
            std::process::exit(1);
        }
    };

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
                    Config::Install(default_datadir_path, revaultd_path)
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
                Err(ConfigError::NotFound) => Config::Install(datadir_path, revaultd_path),
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
