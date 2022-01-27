use std::{error::Error, path::PathBuf, str::FromStr};

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
extern crate serde;
extern crate serde_json;

use revault_hwi::{
    app::revault::RevaultHWI,
    dummysigner::{DummySigner, DUMMYSIGNER_DEFAULT_ADDRESS},
    specter::{Specter, SPECTER_SIMULATOR_DEFAULT_ADDRESS},
    HWIError,
};

use revaultd::config::Config as DaemonConfig;

use revault_gui::{
    app::{
        self,
        config::{default_datadir, ConfigError},
        context::{ConfigContext, Context},
        menu::Menu,
        App,
    },
    conversion::Converter,
    daemon,
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

fn log_level_from_config(config: &app::Config) -> Result<log::LevelFilter, Box<dyn Error>> {
    if let Some(level) = &config.log_level {
        match level.as_ref() {
            "info" => Ok(log::LevelFilter::Info),
            "debug" => Ok(log::LevelFilter::Debug),
            "trace" => Ok(log::LevelFilter::Trace),
            _ => Err(format!("Unknown loglevel '{:?}'.", level).into()),
        }
    } else if let Some(true) = config.debug {
        Ok(log::LevelFilter::Debug)
    } else {
        Ok(log::LevelFilter::Info)
    }
}

pub struct GUI {
    daemon_running: bool,
    state: State,
}

enum State {
    Installer(Installer),
    Loader(Loader),
    App(App),
}

#[derive(Debug)]
pub enum Message {
    CtrlC,
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

async fn ctrl_c() -> Result<(), ()> {
    if let Err(e) = tokio::signal::ctrl_c().await {
        log::error!("{}", e);
    };
    log::info!("Signal received, exiting");
    Ok(())
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
                    Command::batch(vec![
                        command.map(Message::Install),
                        Command::perform(ctrl_c(), |_| Message::CtrlC),
                    ]),
                )
            }
            Config::Run(cfg) => {
                let (loader, command) = Loader::new(cfg);
                (
                    Self {
                        state: State::Loader(loader),
                        daemon_running: false,
                    },
                    Command::batch(vec![
                        command.map(Message::Load),
                        Command::perform(ctrl_c(), |_| Message::CtrlC),
                    ]),
                )
            }
        }
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        if matches!(message, Message::CtrlC) {
            return match &mut self.state {
                State::Installer(v) => v.stop().map(Message::Install),
                State::Loader(v) => v.stop().map(Message::Load),
                State::App(v) => v.stop().map(Message::Run),
            };
        }
        if let Message::Install(installer::Message::Exit(path)) = message {
            let cfg = app::Config::from_file(&path).unwrap();
            let (loader, command) = Loader::new(cfg);
            self.state = State::Loader(loader);
            return command.map(Message::Load);
        }

        if let Message::Load(loader::Message::DaemonStopped) = message {
            log::info!("daemon stopped");
            self.daemon_running = false;
            return Command::none();
        }

        if let Message::Load(loader::Message::Failure(e)) = &message {
            log::info!("daemon panic {}", e);
            self.daemon_running = false;
        }

        if let Message::Load(loader::Message::StoppingDaemon(res)) = message {
            log::info!("stopping daemon {:?}", res);
            return Command::none();
        }

        if let Message::Run(app::Message::StoppingDaemon(res)) = message {
            log::info!("stopping daemon {:?}", res);
            return Command::none();
        }

        if let Message::Load(loader::Message::Synced(info, revaultd)) = message {
            if let State::Loader(loader) = &mut self.state {
                let daemon_config =
                    DaemonConfig::from_file(Some(loader.gui_config.revaultd_config_path.clone()))
                        .unwrap();
                let config = ConfigContext {
                    gui: loader.gui_config.clone(),
                    daemon: daemon_config,
                };

                let role = if config.daemon.stakeholder_config.is_some() {
                    Role::Stakeholder
                } else {
                    Role::Manager
                };

                let converter = Converter::new(config.daemon.bitcoind_config.network);

                let mut context = Context::new(
                    config,
                    revaultd,
                    converter,
                    role,
                    Menu::Home,
                    self.daemon_running,
                    Box::new(|| Box::pin(connect_hardware_wallet())),
                );

                context.blockheight = info.blockheight;
                context.managers_threshold = info.managers_threshold;

                let (app, command) = App::new(context);
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

    fn scale_factor(&self) -> f64 {
        1.0
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
        log::LevelFilter::Info
    };
    setup_logger(level)?;

    let mut settings = Settings::with_flags(config);
    settings.exit_on_close_request = false;

    if let Err(e) = GUI::run(settings) {
        return Err(format!("Failed to launch UI: {}", e).into());
    };
    Ok(())
}

// This creates the log file automagically if it doesn't exist, and logs on stdout
// if None is given
pub fn setup_logger(log_level: log::LevelFilter) -> Result<(), fern::InitError> {
    let dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_else(|e| {
                        println!("Can't get time since epoch: '{}'. Using a dummy value.", e);
                        std::time::Duration::from_secs(0)
                    })
                    .as_secs(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .level_for("iced_wgpu", log::LevelFilter::Off)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Off)
        .level_for("naga", log::LevelFilter::Off)
        .level_for("mio", log::LevelFilter::Off);

    dispatcher.chain(std::io::stdout()).apply()?;

    Ok(())
}

pub async fn connect_hardware_wallet() -> Result<Box<dyn RevaultHWI + Send>, HWIError> {
    if let Ok(device) = DummySigner::try_connect(DUMMYSIGNER_DEFAULT_ADDRESS).await {
        return Ok(device.into());
    }
    if let Ok(device) = Specter::try_connect_simulator(SPECTER_SIMULATOR_DEFAULT_ADDRESS).await {
        return Ok(device.into());
    }

    if let Ok(device) = Specter::try_connect_serial() {
        return Ok(device.into());
    }

    Err(HWIError::DeviceDisconnected)
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
