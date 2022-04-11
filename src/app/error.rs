use crate::daemon::RevaultDError;
use revaultd::config::ConfigError;
use std::convert::From;
use std::io::ErrorKind;

#[derive(Debug, Clone)]
pub enum Error {
    Hardware(revault_hwi::HWIError),
    // TODO: add Clone to ConfigError
    Config(String),
    Daemon(RevaultDError),
    Unexpected(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Config(e) => write!(f, "{}", e),
            Self::Hardware(e) => write!(f, "{}", e),
            Self::Daemon(e) => match e {
                RevaultDError::Unexpected(e) => write!(f, "{}", e),
                RevaultDError::NoAnswer => write!(f, "Daemon did not answer"),
                RevaultDError::Transport(Some(ErrorKind::ConnectionRefused), _) => {
                    write!(f, "Failed to connect to daemon")
                }
                RevaultDError::Transport(kind, e) => {
                    if let Some(k) = kind {
                        write!(f, "{} [{:?}]", e, k)
                    } else {
                        write!(f, "{}", e)
                    }
                }
                RevaultDError::Start(e) => {
                    write!(f, "Failed to start daemon: {}", e)
                }
                RevaultDError::Rpc(code, e) => {
                    write!(f, "[{:?}] {}", code, e)
                }
            },
            Self::Unexpected(e) => write!(f, "Unexpected error: {}", e),
        }
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Self {
        Error::Config(error.to_string())
    }
}

impl From<RevaultDError> for Error {
    fn from(error: RevaultDError) -> Self {
        Error::Daemon(error)
    }
}

impl From<revault_hwi::HWIError> for Error {
    fn from(error: revault_hwi::HWIError) -> Self {
        Error::Hardware(error)
    }
}
