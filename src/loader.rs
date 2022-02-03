use std::convert::From;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{Column, Command, Container, Element, Length, Subscription};
use iced_native::{window, Event};
use log::{debug, info};

use revault_ui::component::{image::revault_colored_logo, text::Text};
use revaultd::config::{Config, ConfigError};

use crate::{
    app::config::{default_datadir, Config as GUIConfig},
    daemon::{
        client,
        embedded::{start_daemon, EmbeddedDaemon},
        model::GetInfoResult,
        Daemon, RevaultDError,
    },
};

type RevaultD = client::RevaultD<client::jsonrpc::JsonRPCClient>;

pub struct Loader {
    pub gui_config: GUIConfig,
    pub daemon_config: Option<Config>,
    pub daemon_started: bool,

    should_exit: bool,
    step: Step,
}

enum Step {
    Connecting,
    StartingDaemon,
    Syncing {
        daemon: Arc<dyn Daemon + Sync + Send>,
        progress: f64,
    },
    Error(Error),
}

#[derive(Debug)]
pub enum Message {
    Event(iced_native::Event),
    Syncing(Result<GetInfoResult, RevaultDError>),
    Synced(GetInfoResult, Arc<dyn Daemon + Sync + Send>),
    Connected(Result<Arc<dyn Daemon + Sync + Send>, Error>),
    Loaded(Result<Arc<dyn Daemon + Sync + Send>, Error>),
    DaemonStopped,
    DaemonStarted(EmbeddedDaemon),
    Failure(RevaultDError),
}

impl Loader {
    pub fn new(gui_config: GUIConfig) -> (Self, Command<Message>) {
        let revaultd_config_path = gui_config.revaultd_config_path.clone();
        let daemon_config = match Config::from_file(Some(revaultd_config_path)) {
            Ok(cfg) => cfg,
            Err(e) => {
                return (
                    Loader {
                        daemon_config: None,
                        gui_config,
                        step: Step::Error(e.into()),
                        should_exit: false,
                        daemon_started: false,
                    },
                    Command::none(),
                )
            }
        };
        let path = socket_path(
            &daemon_config.data_dir,
            daemon_config.bitcoind_config.network,
        )
        .unwrap();
        (
            Loader {
                daemon_config: Some(daemon_config),
                gui_config,
                step: Step::Connecting,
                should_exit: false,
                daemon_started: false,
            },
            Command::perform(connect(path), Message::Loaded),
        )
    }

    fn on_load(&mut self, res: Result<Arc<dyn Daemon + Send + Sync>, Error>) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                self.step = Step::Syncing {
                    daemon: revaultd.clone(),
                    progress: 0.0,
                };
                return Command::perform(sync(revaultd, false), Message::Syncing);
            }
            Err(e) => match e {
                Error::ConfigError(_) => {
                    self.step = Step::Error(e);
                }
                Error::RevaultDError(RevaultDError::Transport(
                    Some(ErrorKind::ConnectionRefused),
                    _,
                ))
                | Error::RevaultDError(RevaultDError::Transport(Some(ErrorKind::NotFound), _)) => {
                    self.step = Step::StartingDaemon;
                    self.daemon_started = true;
                    return Command::batch(vec![
                        Command::perform(
                            start_daemon(self.gui_config.revaultd_config_path.clone()),
                            |res| match res {
                                Ok(daemon) => Message::DaemonStarted(daemon),
                                Err(e) => Message::Failure(e),
                            },
                        ),
                        Command::perform(
                            try_connect(
                                socket_path(
                                    &self.daemon_config.as_ref().unwrap().data_dir,
                                    self.daemon_config.as_ref().unwrap().bitcoind_config.network,
                                )
                                .unwrap(),
                            ),
                            Message::Connected,
                        ),
                    ]);
                }
                _ => {
                    self.step = Step::Error(e);
                }
            },
        }
        Command::none()
    }

    fn on_connect(
        &mut self,
        res: Result<Arc<dyn Daemon + Send + Sync>, Error>,
    ) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                self.step = Step::Syncing {
                    daemon: revaultd.clone(),
                    progress: 0.0,
                };
                Command::perform(sync(revaultd, false), Message::Syncing)
            }
            Err(e) => {
                self.step = Step::Error(e);
                Command::none()
            }
        }
    }

    fn on_sync(&mut self, res: Result<GetInfoResult, RevaultDError>) -> Command<Message> {
        match &mut self.step {
            Step::Syncing { daemon, progress } => {
                match res {
                    Ok(info) => {
                        if (info.sync - 1.0_f64).abs() < f64::EPSILON {
                            let daemon = daemon.clone();
                            return Command::perform(async move { (info, daemon) }, |res| {
                                Message::Synced(res.0, res.1)
                            });
                        } else {
                            *progress = info.sync
                        }
                    }
                    Err(e) => {
                        self.step = Step::Error(e.into());
                        return Command::none();
                    }
                };
                Command::perform(sync(daemon.clone(), true), Message::Syncing)
            }
            _ => Command::none(),
        }
    }

    pub fn stop(&mut self) {
        log::info!("Close requested");
        if let Step::Syncing { daemon, .. } = &mut self.step {
            if !daemon.is_external() {
                log::info!("Stopping internal daemon...");
                if let Some(d) = Arc::get_mut(daemon) {
                    d.stop().expect("Daemon is internal");
                    log::info!("Internal daemon stopped");
                    self.should_exit = true;
                }
            } else {
                self.should_exit = true;
            }
        } else {
            self.should_exit = true;
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connected(res) => self.on_connect(res),
            Message::Loaded(res) => self.on_load(res),
            Message::Syncing(res) => self.on_sync(res),
            Message::Failure(_) => {
                self.daemon_started = false;
                Command::none()
            }
            Message::Event(Event::Window(window::Event::CloseRequested)) => {
                self.stop();
                Command::none()
            }
            _ => Command::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn view(&mut self) -> Element<Message> {
        match &mut self.step {
            Step::StartingDaemon => cover(Text::new("Starting daemon...")),
            Step::Connecting => cover(Text::new("Connecting to daemon...")),
            Step::Syncing { progress, .. } => {
                cover(Text::new(&format!("Syncing... {}%", progress)))
            }
            Step::Error(error) => cover(Text::new(&format!("Error: {}", error))),
        }
    }
}

pub fn cover<'a, T: 'a, C: Into<Element<'a, T>>>(content: C) -> Element<'a, T> {
    Column::new()
        .push(Container::new(
            revault_colored_logo()
                .width(Length::Units(300))
                .height(Length::Fill),
        ))
        .push(content)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .padding(50)
        .spacing(50)
        .align_items(iced::Align::Center)
        .into()
}

async fn connect(socket_path: PathBuf) -> Result<Arc<dyn Daemon + Sync + Send>, Error> {
    let client = client::jsonrpc::JsonRPCClient::new(socket_path);
    let revaultd = RevaultD::new(client);

    debug!("Connecting to revaultd");
    revaultd.get_info()?;
    info!("Connected to revaultd");

    Ok(Arc::new(revaultd))
}

async fn sync(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    sleep: bool,
) -> Result<GetInfoResult, RevaultDError> {
    if sleep {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    revaultd.get_info()
}

async fn try_connect(socket_path: PathBuf) -> Result<Arc<dyn Daemon + Send + Sync>, Error> {
    fn try_connect_to_revault(socket_path: &PathBuf, i: i32) -> Result<Arc<RevaultD>, Error> {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let client = client::jsonrpc::JsonRPCClient::new(socket_path);
        let revaultd = RevaultD::new(client);
        if let Err(e) = revaultd.get_info() {
            log::warn!("Failed to connect to revaultd ({} more try): {}", i, e);
            return Err(e.into());
        };

        Ok(Arc::new(revaultd))
    }

    let client = try_connect_to_revault(&socket_path, 5)
        .or_else(|_| try_connect_to_revault(&socket_path, 4))
        .or_else(|_| try_connect_to_revault(&socket_path, 3))
        .or_else(|_| try_connect_to_revault(&socket_path, 2))
        .or_else(|_| try_connect_to_revault(&socket_path, 1))
        .or_else(|_| try_connect_to_revault(&socket_path, 0))?;

    Ok(client)
}

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    RevaultDError(RevaultDError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "Config error: {}", e),
            Self::RevaultDError(e) => write!(f, "RevaultD error: {}", e),
        }
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Self {
        Error::ConfigError(error)
    }
}

impl From<RevaultDError> for Error {
    fn from(error: RevaultDError) -> Self {
        Error::RevaultDError(error)
    }
}

/// default revaultd socket path is .revault/bitcoin/revaultd_rpc
fn socket_path(
    datadir: &Option<PathBuf>,
    network: bitcoin::Network,
) -> Result<PathBuf, ConfigError> {
    let mut path = if let Some(ref datadir) = datadir {
        datadir.clone()
    } else {
        default_datadir().map_err(|_| ConfigError::DatadirNotFound)?
    };
    path.push(network.to_string());
    path.push("revaultd_rpc");
    Ok(path)
}
