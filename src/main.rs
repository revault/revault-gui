use std::{error::Error, path::PathBuf, str::FromStr};

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

use revault_gui::{
    app::{self, config::ConfigError, context::Context, menu::Menu, App},
    conversion::Converter,
    daemon::{self, config::default_datadir},
    installer::{self, Installer},
    loader::{self, Loader},
    revault::Role,
};

#[derive(Debug, PartialEq)]
enum Arg {
    ConfigPath(PathBuf),
    DatadirPath(PathBuf),
    Network(bitcoin::Network),
}

fn parse_args(args: Vec<String>) -> Result<Vec<Arg>, Box<dyn Error>> {
    let mut res = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        if arg == "--conf" {
            if let Some(a) = args.get(i + 1) {
                res.push(Arg::ConfigPath(PathBuf::from(a)));
            } else {
                return Err("missing arg to --conf".into());
            }
        } else if arg == "--datadir" {
            if let Some(a) = args.get(i + 1) {
                res.push(Arg::DatadirPath(PathBuf::from(a)));
            } else {
                return Err("missing arg to --datadir".into());
            }
        } else if arg.contains("--") {
            let network = bitcoin::Network::from_str(args[i].trim_start_matches("--"))?;
            res.push(Arg::Network(network));
        }
    }

    Ok(res)
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

pub struct GUI {
    daemon_running: bool,
    state: State,
}

enum State {
    Installer(Installer),
    Loader(Loader),
    App(App<daemon::client::jsonrpc::JsonRPCClient>),
}

#[derive(Debug)]
pub enum Message {
    Install(installer::Message),
    Load(loader::Message),
    Run(app::Message),
}

impl GUI {
    fn exit_requested(&self) -> bool {
        match &self.state {
            State::Installer(v) => v.should_exit(),
            State::Loader(v) => v.should_exit(),
            State::App(v) => v.should_exit(),
        }
    }
}

impl Application for GUI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn title(&self) -> String {
        match self.state {
            State::Installer(_) => String::from("Revault Installer"),
            State::App(_) => String::from("Revault GUI"),
            State::Loader(..) => String::from("Revault"),
        }
    }

    fn new(config: Config) -> (GUI, Command<Self::Message>) {
        match config {
            Config::Install(config_path, network) => {
                let (install, command) = Installer::new(config_path, network);
                (
                    Self {
                        state: State::Installer(install),
                        daemon_running: false,
                    },
                    command.map(Message::Install),
                )
            }
            Config::Run(cfg) => {
                let (loader, command) = Loader::new(cfg);
                (
                    Self {
                        state: State::Loader(loader),
                        daemon_running: false,
                    },
                    command.map(Message::Load),
                )
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
            self.state = State::Loader(loader);
            return command.map(Message::Load);
        }

        if let Message::Load(loader::Message::DaemonStopped) = message {
            tracing::info!("daemon stopped");
            self.daemon_running = false;
            return Command::none();
        }

        if let Message::Load(loader::Message::Failure(e)) = message {
            tracing::info!("daemon panic {}", e);
            self.daemon_running = false;
            return Command::none();
        }

        if let Message::Load(loader::Message::StoppingDaemon(res)) = message {
            tracing::info!("stopping daemon {:?}", res);
            return Command::none();
        }

        if let Message::Run(app::Message::StoppingDaemon(res)) = message {
            tracing::info!("stopping daemon {:?}", res);
            return Command::none();
        }

        if let Message::Load(loader::Message::Synced(info, revaultd)) = message {
            if let State::Loader(loader) = &mut self.state {
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
                    self.daemon_running,
                );

                let (app, command) = App::new(context, config);
                self.state = State::App(app);
                return command.map(Message::Run);
            }
            return Command::none();
        }

        match (&mut self.state, message) {
            (State::Installer(i), Message::Install(msg)) => {
                i.update(msg, clipboard).map(Message::Install)
            }
            (State::Loader(loader), Message::Load(msg)) => {
                let cmd = loader.update(msg).map(Message::Load);
                if loader.daemon_started {
                    self.daemon_running = true;
                }
                cmd
            }
            (State::App(i), Message::Run(msg)) => i.update(msg, clipboard).map(Message::Run),
            _ => Command::none(),
        }
    }

    fn should_exit(&self) -> bool {
        self.exit_requested() && !self.daemon_running
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match &self.state {
            State::Installer(v) => v.subscription().map(Message::Install),
            State::Loader(v) => v.subscription().map(Message::Load),
            State::App(v) => v.subscription().map(Message::Run),
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        match &mut self.state {
            State::Installer(v) => v.view().map(Message::Install),
            State::App(v) => v.view().map(Message::Run),
            State::Loader(v) => v.view().map(Message::Load),
        }
    }
}

pub enum Config {
    Run(app::Config),
    Install(PathBuf, bitcoin::Network),
}

impl Config {
    pub fn new(datadir_path: PathBuf, network: bitcoin::Network) -> Result<Self, Box<dyn Error>> {
        let mut path = datadir_path.clone();
        path.push(app::Config::file_name(&network));
        match app::Config::from_file(&path) {
            Ok(cfg) => Ok(Config::Run(cfg)),
            Err(ConfigError::NotFound) => Ok(Config::Install(datadir_path, network)),
            Err(e) => Err(format!("Failed to read configuration file: {}", e).into()),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args(std::env::args().collect())?;
    let config = match args.as_slice() {
        [] => {
            let datadir_path = default_datadir().unwrap();
            Config::new(datadir_path, bitcoin::Network::Bitcoin)
        }
        [Arg::Network(network)] => {
            let datadir_path = default_datadir().unwrap();
            Config::new(datadir_path, network.clone())
        }
        [Arg::ConfigPath(path)] => Ok(Config::Run(app::Config::from_file(&path)?)),
        [Arg::DatadirPath(datadir_path)] => {
            Config::new(datadir_path.clone(), bitcoin::Network::Bitcoin)
        }
        [Arg::DatadirPath(datadir_path), Arg::Network(network)]
        | [Arg::Network(network), Arg::DatadirPath(datadir_path)] => {
            Config::new(datadir_path.clone(), network.clone())
        }
        _ => {
            return Err("Unknown args combination".into());
        }
    }?;

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

    let mut settings = Settings::with_flags(config);
    settings.exit_on_close_request = false;

    if let Err(e) = GUI::run(settings) {
        return Err(format!("Failed to launch UI: {}", e).into());
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        assert_eq!(true, parse_args(vec!["--meth".into()]).is_err());
        assert_eq!(true, parse_args(vec!["--datadir".into()]).is_err());
        assert_eq!(true, parse_args(vec!["--conf".into()]).is_err());
        assert_eq!(
            Some(vec![
                Arg::DatadirPath(PathBuf::from(".")),
                Arg::ConfigPath(PathBuf::from("hello.toml")),
            ]),
            parse_args(
                "--datadir . --conf hello.toml"
                    .split(" ")
                    .map(|a| a.to_string())
                    .collect()
            )
            .ok()
        );
        assert_eq!(
            Some(vec![Arg::Network(bitcoin::Network::Regtest)]),
            parse_args(vec!["--regtest".into()]).ok()
        );
        assert_eq!(
            Some(vec![
                Arg::DatadirPath(PathBuf::from("hello")),
                Arg::Network(bitcoin::Network::Testnet)
            ]),
            parse_args(
                "--datadir hello --testnet"
                    .split(" ")
                    .map(|a| a.to_string())
                    .collect()
            )
            .ok()
        );
        assert_eq!(
            Some(vec![
                Arg::Network(bitcoin::Network::Testnet),
                Arg::DatadirPath(PathBuf::from("hello"))
            ]),
            parse_args(
                "--testnet --datadir hello"
                    .split(" ")
                    .map(|a| a.to_string())
                    .collect()
            )
            .ok()
        );
    }
}
