use std::convert::From;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{Column, Command, Container, Element, Length};

use crate::{
    app::config::Config as GUIConfig,
    daemon::{
        client::{self, GetInfoResponse, RevaultDError},
        config::{Config, ConfigError},
        start_daemon, StartDaemonError,
    },
    ui::component::{image::revault_colored_logo, text::Text},
};

type RevaultD = client::RevaultD<client::jsonrpc::JsonRPCClient>;

pub struct Loader {
    pub gui_config: GUIConfig,
    revaultd: Option<Arc<RevaultD>>,
    step: Step,
}

enum Step {
    Connecting,
    StartingDaemon,
    Syncing { progress: f64 },
    Error { error: String },
}

#[derive(Debug)]
pub enum Message {
    DaemonStarted(Result<Arc<RevaultD>, Error>),
    Syncing(Result<GetInfoResponse, RevaultDError>),
    Synced(GetInfoResponse, Arc<RevaultD>),
    Connected(Result<Arc<RevaultD>, Error>),
}

impl Loader {
    pub fn new(gui_config: GUIConfig) -> (Self, Command<Message>) {
        let revaultd_config_path = gui_config.revaultd_config_path.clone();
        (
            Loader {
                gui_config,
                revaultd: None,
                step: Step::Connecting,
            },
            Command::perform(connect(revaultd_config_path), Message::Connected),
        )
    }

    fn on_connect(&mut self, res: Result<Arc<RevaultD>, Error>) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                self.step = Step::Syncing { progress: 0.0 };
                self.revaultd = Some(revaultd.clone());
                return Command::perform(sync(revaultd, false), Message::Syncing);
            }
            Err(e) => match e {
                Error::ConfigError(ConfigError::NotFound) => {
                    self.step = Step::Error {
                        error: format!(
                            "config not found at path: {:?}",
                            self.gui_config.revaultd_config_path
                        ),
                    };
                }
                Error::RevaultDError(RevaultDError::IOError(ErrorKind::ConnectionRefused))
                | Error::RevaultDError(RevaultDError::IOError(ErrorKind::NotFound)) => {
                    self.step = Step::StartingDaemon;
                    return Command::perform(
                        start_daemon_and_connect(
                            self.gui_config.revaultd_config_path.to_owned(),
                            self.gui_config.revaultd_path.to_owned(),
                        ),
                        Message::DaemonStarted,
                    );
                }
                _ => return self.on_error(&e),
            },
        }
        Command::none()
    }

    fn on_daemon_started(&mut self, res: Result<Arc<RevaultD>, Error>) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                self.step = Step::Syncing { progress: 0.0 };
                self.revaultd = Some(revaultd.clone());
                Command::perform(sync(revaultd, false), Message::Syncing)
            }
            Err(e) => self.on_error(&e),
        }
    }

    fn on_error(&mut self, e: &dyn std::fmt::Display) -> Command<Message> {
        self.step = Step::Error {
            error: format!("error: {}", e),
        };
        Command::none()
    }

    #[allow(unused_variables, unused_assignments)]
    fn on_sync(&mut self, res: Result<GetInfoResponse, RevaultDError>) -> Command<Message> {
        match self.step {
            Step::Syncing { mut progress } => {
                match res {
                    Err(e) => return self.on_error(&e),
                    Ok(info) => {
                        if (info.sync - 1.0_f64).abs() < f64::EPSILON {
                            return Command::perform(
                                synced(info, self.revaultd.as_ref().unwrap().clone()),
                                |res| Message::Synced(res.0, res.1),
                            );
                        } else {
                            progress = info.sync
                        }
                    }
                };
                Command::perform(
                    sync(self.revaultd.as_ref().unwrap().clone(), true),
                    Message::Syncing,
                )
            }
            _ => Command::none(),
        }
    }
}

impl Loader {
    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connected(res) => self.on_connect(res),
            Message::Syncing(res) => self.on_sync(res),
            Message::DaemonStarted(res) => self.on_daemon_started(res),
            _ => Command::none(),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match &mut self.step {
            Step::StartingDaemon => cover(Text::new("Starting daemon...")),
            Step::Connecting => cover(Text::new("Connecting to daemon...")),
            Step::Syncing { progress, .. } => {
                cover(Text::new(&format!("Syncing... {}%", progress)))
            }
            Step::Error { error } => cover(Text::new(&format!("Error: {}", error))),
        }
    }

    pub fn load(&self) -> Command<Message> {
        Command::perform(
            connect(self.gui_config.revaultd_config_path.clone()),
            Message::Connected,
        )
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

async fn synced(
    info: GetInfoResponse,
    revaultd: Arc<RevaultD>,
) -> (GetInfoResponse, Arc<RevaultD>) {
    (info, revaultd)
}

async fn connect(revaultd_config_path: PathBuf) -> Result<Arc<RevaultD>, Error> {
    let cfg = Config::from_file(&revaultd_config_path)?;
    let socket_path = cfg.socket_path().map_err(|e| {
        RevaultDError::UnexpectedError(format!(
            "Failed to find revaultd socket path: {}",
            e.to_string()
        ))
    })?;

    let client = client::jsonrpc::JsonRPCClient::new(socket_path);
    let revaultd = RevaultD::new(&cfg, client)?;

    Ok(Arc::new(revaultd))
}

async fn sync(revaultd: Arc<RevaultD>, sleep: bool) -> Result<GetInfoResponse, RevaultDError> {
    if sleep {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    revaultd.get_info()
}

async fn start_daemon_and_connect(
    revaultd_config_path: PathBuf,
    revaultd_path: Option<PathBuf>,
) -> Result<Arc<RevaultD>, Error> {
    let revaultd_path = revaultd_path.unwrap_or_else(|| PathBuf::from("revaultd"));

    start_daemon(&revaultd_config_path, &revaultd_path).await?;

    let cfg = Config::from_file(&revaultd_config_path)?;

    fn try_connect_to_revault(cfg: &Config, i: i32) -> Result<Arc<RevaultD>, Error> {
        std::thread::sleep(std::time::Duration::from_secs(3));
        let socket_path = cfg.socket_path().map_err(|e| {
            RevaultDError::UnexpectedError(format!(
                "Failed to find revaultd socket path: {}",
                e.to_string()
            ))
        })?;

        let client = client::jsonrpc::JsonRPCClient::new(socket_path);
        RevaultD::new(cfg, client).map(Arc::new).map_err(|e| {
            tracing::warn!("Failed to connect to revaultd ({} more try): {}", i, e);
            e.into()
        })
    }

    try_connect_to_revault(&cfg, 5)
        .or_else(|_| try_connect_to_revault(&cfg, 4))
        .or_else(|_| try_connect_to_revault(&cfg, 3))
        .or_else(|_| try_connect_to_revault(&cfg, 2))
        .or_else(|_| try_connect_to_revault(&cfg, 1))
        .or_else(|_| try_connect_to_revault(&cfg, 0))
}

#[derive(Debug)]
pub enum Error {
    ConfigError(ConfigError),
    RevaultDError(RevaultDError),
    StartingDaemonError(StartDaemonError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "Config error: {}", e),
            Self::RevaultDError(e) => write!(f, "RevaultD error: {}", e),
            Self::StartingDaemonError(e) => write!(f, "{}", e),
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

impl From<StartDaemonError> for Error {
    fn from(error: StartDaemonError) -> Self {
        Error::StartingDaemonError(error)
    }
}
