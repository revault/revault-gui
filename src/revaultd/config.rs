use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    /// An optional custom data directory
    pub data_dir: Option<PathBuf>,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let config = std::fs::read(path)
            .map_err(|e| ConfigError(format!("Reading configuration file: {}", e)))
            .and_then(|file_content| {
                toml::from_slice::<Config>(&file_content)
                    .map_err(|e| ConfigError(format!("Parsing configuration file: {}", e)))
            })?;
        Ok(config)
    }
}

impl std::default::Default for Config {
    fn default() -> Self {
        Config { data_dir: None }
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

    Err(ConfigError(
        "Could not locate the configuration directory.".to_owned(),
    ))
}

#[derive(PartialEq, Eq, Debug)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Configuration error: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}
