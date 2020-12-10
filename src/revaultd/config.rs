use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bitcoind_config: BitcoindConfig,
    /// An optional custom data directory
    pub data_dir: Option<PathBuf>,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
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

    /// default revaultd socket path is .revault/bitcoin/revaultd_rpc
    pub fn socket_path(&self) -> Result<PathBuf, ConfigError> {
        let mut path = if let Some(ref datadir) = self.data_dir {
            datadir.clone()
        } else {
            default_datadir()?
        };
        path.push(&self.bitcoind_config.network);
        path.push("revaultd_rpc");
        Ok(path)
    }
}

impl std::default::Default for Config {
    fn default() -> Self {
        Config {
            bitcoind_config: BitcoindConfig::default(),
            data_dir: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BitcoindConfig {
    pub network: String,
}

impl std::default::Default for BitcoindConfig {
    fn default() -> Self {
        BitcoindConfig {
            network: "bitcoin".to_string(),
        }
    }
}

// From github.com/re-vault/revaultd:
// Get the absolute path to the revault configuration folder.
///
/// This a "revault" directory in the XDG standard configuration directory for all OSes but
/// Linux-based ones, for which it's `~/.revault`.
/// Rationale: we want to have the database, RPC socket, etc.. in the same folder as the
/// configuration file but for Linux the XDG specify a data directory (`~/.local/share/`) different
/// from the configuration one (`~/.config/`).
pub fn default_datadir() -> Result<PathBuf, ConfigError> {
    #[cfg(target_os = "linux")]
    let configs_dir = dirs::home_dir();

    #[cfg(not(target_os = "linux"))]
    let configs_dir = dirs::config_dir();

    if let Some(mut path) = configs_dir {
        #[cfg(target_os = "linux")]
        path.push(".revault");

        #[cfg(not(target_os = "linux"))]
        path.push("Revault");

        return Ok(path);
    }

    Err(ConfigError::Unexpected(
        "Could not locate the configuration directory.".to_owned(),
    ))
}

/// default_config_path returns the default config location of the revault deamon.
pub fn default_config_path() -> Result<PathBuf, ConfigError> {
    let mut datadir = default_datadir()?;
    datadir.push("revault.toml");
    Ok(datadir)
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
            Self::NotFound => write!(f, "Configuration error: not found"),
            Self::ReadingFile(e) => write!(f, "Configuration error while reading file: {}", e),
            Self::Unexpected(e) => write!(f, "Configuration error unexpected: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
