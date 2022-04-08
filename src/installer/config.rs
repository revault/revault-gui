use bitcoin::{util::bip32, Network};
use revaultd::config::{BitcoindConfig, ManagerConfig, WatchtowerConfig};

use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, time::Duration};

/// If we are a stakeholder, we need to connect to our watchtower(s)
#[derive(Debug, Clone, Serialize)]
pub struct StakeholderConfig {
    pub xpub: bip32::ExtendedPubKey,
    pub watchtowers: Vec<WatchtowerConfig>,
    pub emergency_address: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptsConfig {
    pub deposit_descriptor: String,
    pub unvault_descriptor: String,
    pub cpfp_descriptor: String,
}

/// Static informations we require to operate
#[derive(Debug, Clone, Serialize)]
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

impl Config {
    pub const DEFAULT_FILE_NAME: &'static str = "revaultd.toml";
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
                poll_interval_secs: Duration::from_secs(30),
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

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
