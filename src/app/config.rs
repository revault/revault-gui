use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::revaultd::config::default_datadir;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Path to revaultd configuration file.
    pub revaultd_config_path: PathBuf,
    /// Path to revaultd binary.
    pub revaultd_path: Option<PathBuf>,
    /// log level, can be "info", "debug", "trace".
    pub log_level: Option<String>,
    /// Use iced debug feature if true.
    pub debug: Option<bool>,
}

pub const DEFAULT_FILE_NAME: &'static str = "revault_gui.toml";

impl Config {
    pub fn new(revaultd_config_path: PathBuf) -> Self {
        Self {
            revaultd_config_path,
            revaultd_path: None,
            log_level: None,
            debug: None,
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let config = std::fs::read(path)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ConfigError::NotFound,
                _ => ConfigError::ReadingFile(format!("Reading configuration file: {}", e)),
            })
            .and_then(|file_content| {
                toml::from_slice::<Config>(&file_content).map_err(|e| {
                    ConfigError::ReadingFile(format!("Parsing configuration file: {}", e))
                })
            })?;
        Ok(config)
    }

    pub fn default_path() -> Result<PathBuf, ConfigError> {
        let mut datadir = default_datadir().map_err(|_| {
            ConfigError::Unexpected("Could not locate the default datadir directory.".to_owned())
        })?;
        datadir.push(DEFAULT_FILE_NAME);
        Ok(datadir)
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ConfigError {
    NotFound,
    ReadingFile(String),
    Unexpected(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Config file not found"),
            Self::ReadingFile(e) => write!(f, "Error while reading file: {}", e),
            Self::Unexpected(e) => write!(f, "Unexpected error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
