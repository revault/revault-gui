use crate::revaultd::{config::ConfigError, RevaultDError};
use std::convert::From;

#[derive(Debug, Clone)]
pub enum Error {
    ConfigError(ConfigError),
    RevaultDError(RevaultDError),
    UnexpectedError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "Config error: {}", e),
            Self::RevaultDError(e) => write!(f, "RevaultD error: {}", e),
            Self::UnexpectedError(e) => write!(f, "Unexpected error: {}", e),
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
