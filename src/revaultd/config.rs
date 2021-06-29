use bitcoin::{util::bip32, Network};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

// This file is adapted from github.com/re-vault/revaultd:

/// Everything we need to know for talking to bitcoind serenely
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BitcoindConfig {
    /// The network we are operating on, one of "bitcoin", "testnet", "regtest"
    pub network: Network,
    /// Path to bitcoind's cookie file, to authenticate the RPC connection
    pub cookie_path: PathBuf,
    /// The IP:port bitcoind's RPC is listening on
    pub addr: SocketAddr,
    /// The poll interval for bitcoind
    pub poll_interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WatchtowerConfig {
    pub host: String,
    pub noise_key: String,
}

/// If we are a stakeholder, we need to connect to our watchtower(s)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StakeholderConfig {
    pub xpub: bip32::ExtendedPubKey,
    pub watchtowers: Vec<WatchtowerConfig>,
    pub emergency_address: String,
}

// Same fields as the WatchtowerConfig struct for now, but leave them separate.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CosignerConfig {
    pub host: String,
    pub noise_key: String,
}

/// If we are a manager, we need to connect to cosigning servers
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManagerConfig {
    pub xpub: bip32::ExtendedPubKey,
    pub cosigners: Vec<CosignerConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScriptsConfig {
    pub deposit_descriptor: String,
    pub unvault_descriptor: String,
    pub cpfp_descriptor: String,
}

/// Static informations we require to operate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Everything we need to know to talk to bitcoind
    pub bitcoind_config: BitcoindConfig,
    /// Some() if we are a stakeholder
    pub stakeholder_config: Option<StakeholderConfig>,
    /// Some() if we are a manager
    pub manager_config: Option<ManagerConfig>,
    /// Descriptors
    pub scripts_config: ScriptsConfig,
    /// The host of the sync server (may be an IP or a hidden service)
    pub coordinator_host: String,
    /// The Noise static public key of the sync server
    pub coordinator_noise_key: String,
    /// The poll intervals for signature fetching (default: 1min)
    pub coordinator_poll_seconds: Option<u64>,
    /// An optional custom data directory
    pub data_dir: Option<PathBuf>,
    /// Whether to daemonize the process
    pub daemon: Option<bool>,
    /// What messages to log
    pub log_level: Option<String>,
}

pub const DEFAULT_FILE_NAME: &str = "revaultd.toml";

impl Config {
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

    /// default revaultd socket path is .revault/bitcoin/revaultd_rpc
    pub fn socket_path(&self) -> Result<PathBuf, ConfigError> {
        let mut path = if let Some(ref datadir) = self.data_dir {
            datadir.clone()
        } else {
            default_datadir().map_err(|_| {
                ConfigError::Unexpected("Could not locate the default datadir.".to_owned())
            })?
        };
        path.push(&self.bitcoind_config.network.to_string());
        path.push("revaultd_rpc");
        Ok(path)
    }

    /// default_config_path returns the default config location of the revault deamon.
    pub fn default_path() -> Result<PathBuf, ConfigError> {
        let mut datadir = default_datadir().map_err(|_| {
            ConfigError::Unexpected("Could not locate the default datadir.".to_owned())
        })?;
        datadir.push(DEFAULT_FILE_NAME);
        Ok(datadir)
    }

    /// returns a revaultd config with empty or dummy values
    pub fn new() -> Config {
        Self {
            bitcoind_config: BitcoindConfig {
                network: Network::Bitcoin,
                cookie_path: PathBuf::new(),
                addr: SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                    8080,
                ),
                poll_interval_secs: None,
            },
            stakeholder_config: None,
            manager_config: None,
            scripts_config: ScriptsConfig {
                deposit_descriptor: "".to_string(),
                unvault_descriptor: "".to_string(),
                cpfp_descriptor: "".to_string(),
            },
            coordinator_host: "".to_string(),
            coordinator_noise_key: "".to_string(),
            coordinator_poll_seconds: None,
            data_dir: None,
            daemon: None,
            log_level: None,
        }
    }
}

// From github.com/revault/revaultd:
// Get the absolute path to the revault configuration folder.
///
/// This a "revault" directory in the XDG standard configuration directory for all OSes but
/// Linux-based ones, for which it's `~/.revault`.
/// Rationale: we want to have the database, RPC socket, etc.. in the same folder as the
/// configuration file but for Linux the XDG specify a data directory (`~/.local/share/`) different
/// from the configuration one (`~/.config/`).
pub fn default_datadir() -> Result<PathBuf, ()> {
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

    Err(())
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
            Self::NotFound => write!(f, "Revaultd Configuration error: not found"),
            Self::ReadingFile(e) => {
                write!(f, "Revaultd Configuration error while reading file: {}", e)
            }
            Self::Unexpected(e) => write!(f, "Revaultd Configuration error unexpected: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}
