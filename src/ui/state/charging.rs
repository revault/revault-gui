use std::io::ErrorKind;
use std::path::PathBuf;

use iced::{Command, Element};

use super::State;
use crate::revaultd::{
    config::{default_config_path, Config, ConfigError},
    start_daemon, RevaultD, RevaultDError,
};
use crate::ui::{error::Error, message::Message, view::charging::*};

#[derive(Debug, Clone)]
pub enum ChargingState {
    Connecting,
    StartingDaemon,
    Syncing { revaultd: RevaultD, progress: f64 },
    Error { error: String },
    AskInstall { view: ChargingAskInstallView },
}

impl ChargingState {
    fn on_connect(
        &mut self,
        revaultd_config_path: Option<PathBuf>,
        res: Result<RevaultD, Error>,
    ) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                let state = Self::Syncing {
                    revaultd: revaultd.to_owned(),
                    progress: 0.0,
                };
                *self = state;
                return Command::perform(sync(revaultd, false), Message::Syncing);
            }
            Err(e) => match e {
                Error::ConfigError(ConfigError::NotFound) => {
                    if let Some(path) = revaultd_config_path {
                        *self = Self::Error {
                            error: format!("config not found at path: {:?}", path),
                        };
                    } else {
                        *self = Self::AskInstall {
                            view: ChargingAskInstallView::new(),
                        };
                    }
                }
                Error::RevaultDError(RevaultDError::IOError(ErrorKind::ConnectionRefused))
                | Error::RevaultDError(RevaultDError::IOError(ErrorKind::NotFound)) => {
                    *self = Self::StartingDaemon;
                    return Command::perform(
                        start_daemon_and_connect(revaultd_config_path),
                        Message::DaemonStarted,
                    );
                }
                _ => return self.on_error(&e),
            },
        }
        Command::none()
    }

    fn on_daemon_started(&mut self, res: Result<RevaultD, Error>) -> Command<Message> {
        match res {
            Ok(revaultd) => {
                let state = Self::Syncing {
                    revaultd: revaultd.to_owned(),
                    progress: 0.0,
                };
                *self = state;
                Command::perform(sync(revaultd, false), Message::Syncing)
            }
            Err(e) => self.on_error(&e),
        }
    }

    fn on_error(&mut self, e: &dyn std::fmt::Display) -> Command<Message> {
        *self = Self::Error {
            error: format!("error: {}", e),
        };
        Command::none()
    }

    #[allow(unused_variables, unused_assignments)]
    fn on_sync(&mut self, res: Result<f64, RevaultDError>) -> Command<Message> {
        match self {
            Self::Syncing {
                revaultd,
                mut progress,
            } => {
                match res {
                    Err(e) => return self.on_error(&e),
                    Ok(p) => {
                        if (p - 1.0_f64).abs() < f64::EPSILON {
                            return Command::perform(synced(revaultd.to_owned()), Message::Synced);
                        } else {
                            progress = p
                        }
                    }
                };
                Command::perform(sync(revaultd.to_owned(), true), Message::Syncing)
            }
            _ => Command::none(),
        }
    }
}

impl State for ChargingState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connected(res) => self.on_connect(res.0, res.1),
            Message::Syncing(res) => self.on_sync(res),
            Message::DaemonStarted(res) => self.on_daemon_started(res),
            _ => Command::none(),
        }
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Self::StartingDaemon => charging_starting_daemon_view(),
            Self::Connecting => charging_connect_view(),
            Self::Syncing { progress, .. } => charging_syncing_view(progress),
            Self::Error { error } => charging_error_view(error),
            Self::AskInstall { view } => view.view(),
        }
    }
}

pub async fn synced(revaultd: RevaultD) -> RevaultD {
    revaultd
}

pub async fn connect(
    revaultd_config_path: Option<PathBuf>,
) -> (Option<PathBuf>, Result<RevaultD, Error>) {
    let res = try_connect(&revaultd_config_path).await;
    return (revaultd_config_path, res);
}

async fn try_connect(revaultd_config_path: &Option<PathBuf>) -> Result<RevaultD, Error> {
    let path = if let Some(ref p) = revaultd_config_path {
        p.to_owned()
    } else {
        default_config_path().map_err(|e| Error::UnexpectedError(e.to_string()))?
    };

    let cfg = Config::from_file(&path)?;
    let revaultd = RevaultD::new(cfg)?;

    return Ok(revaultd);
}

pub async fn sync(revaultd: RevaultD, sleep: bool) -> Result<f64, RevaultDError> {
    if sleep {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let resp = revaultd.get_info()?;
    Ok(resp.sync)
}

pub async fn start_daemon_and_connect(
    revaultd_config_path: Option<PathBuf>,
) -> Result<RevaultD, Error> {
    let path = if let Some(ref p) = revaultd_config_path {
        p.to_owned()
    } else {
        default_config_path().map_err(|e| Error::UnexpectedError(e.to_string()))?
    };

    start_daemon(&path).await?;

    std::thread::sleep(std::time::Duration::from_secs(5));

    let cfg = Config::from_file(&path)?;
    let revaultd = RevaultD::new(cfg)?;
    Ok(revaultd)
}
