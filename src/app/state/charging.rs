use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;

use iced::{Command, Element};

use super::State;
use crate::app::{
    error::Error,
    message::Message,
    view::{charging::*, Context},
};
use crate::revaultd::{
    config::{Config, ConfigError},
    start_daemon, GetInfoResponse, RevaultD, RevaultDError,
};

#[derive(Debug, Clone)]
pub struct ChargingState {
    revaultd_config_path: PathBuf,
    revaultd_path: Option<PathBuf>,
    revaultd: Option<Arc<RevaultD>>,
    step: ChargingStep,
}

#[derive(Debug, Clone)]
enum ChargingStep {
    Connecting,
    StartingDaemon,
    Syncing { progress: f64 },
    Error { error: String },
}

impl ChargingState {
    pub fn new(revaultd_config_path: PathBuf, revaultd_path: Option<PathBuf>) -> Self {
        ChargingState {
            revaultd_config_path,
            revaultd_path,
            revaultd: None,
            step: ChargingStep::Connecting,
        }
    }

    fn on_connect(&mut self, res: Result<Arc<RevaultD>, Error>) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                self.step = ChargingStep::Syncing { progress: 0.0 };
                self.revaultd = Some(revaultd.clone());
                return Command::perform(sync(revaultd, false), Message::Syncing);
            }
            Err(e) => match e {
                Error::ConfigError(ConfigError::NotFound) => {
                    self.step = ChargingStep::Error {
                        error: format!("config not found at path: {:?}", self.revaultd_config_path),
                    };
                }
                Error::RevaultDError(RevaultDError::IOError(ErrorKind::ConnectionRefused))
                | Error::RevaultDError(RevaultDError::IOError(ErrorKind::NotFound)) => {
                    self.step = ChargingStep::StartingDaemon;
                    return Command::perform(
                        start_daemon_and_connect(
                            self.revaultd_config_path.to_owned(),
                            self.revaultd_path.to_owned(),
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
                self.step = ChargingStep::Syncing { progress: 0.0 };
                self.revaultd = Some(revaultd.clone());
                Command::perform(sync(revaultd, false), Message::Syncing)
            }
            Err(e) => self.on_error(&e),
        }
    }

    fn on_error(&mut self, e: &dyn std::fmt::Display) -> Command<Message> {
        self.step = ChargingStep::Error {
            error: format!("error: {}", e),
        };
        Command::none()
    }

    #[allow(unused_variables, unused_assignments)]
    fn on_sync(&mut self, res: Result<GetInfoResponse, RevaultDError>) -> Command<Message> {
        match self.step {
            ChargingStep::Syncing { mut progress } => {
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

impl State for ChargingState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connected(res) => self.on_connect(res),
            Message::Syncing(res) => self.on_sync(res),
            Message::DaemonStarted(res) => self.on_daemon_started(res),
            _ => Command::none(),
        }
    }

    fn view(&mut self, _ctx: &Context) -> Element<Message> {
        match &mut self.step {
            ChargingStep::StartingDaemon => charging_starting_daemon_view(),
            ChargingStep::Connecting => charging_connect_view(),
            ChargingStep::Syncing { progress, .. } => charging_syncing_view(&progress),
            ChargingStep::Error { error } => charging_error_view(&error),
        }
    }

    fn load(&self) -> Command<Message> {
        Command::perform(
            connect(self.revaultd_config_path.clone()),
            Message::Connected,
        )
    }
}

impl From<ChargingState> for Box<dyn State> {
    fn from(s: ChargingState) -> Box<dyn State> {
        Box::new(s)
    }
}

async fn synced(
    info: GetInfoResponse,
    revaultd: Arc<RevaultD>,
) -> (GetInfoResponse, Arc<RevaultD>) {
    (info, revaultd)
}

async fn connect(revaultd_config_path: PathBuf) -> Result<Arc<RevaultD>, Error> {
    let cfg = Config::from_file(&revaultd_config_path)?;
    let revaultd = RevaultD::new(&cfg)?;

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
        RevaultD::new(cfg).map(Arc::new).map_err(|e| {
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
