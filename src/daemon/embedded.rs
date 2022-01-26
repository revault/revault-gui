use std::path::PathBuf;

use log::debug;

use revaultd::{config::Config, revault_net::sodiumoxide, DaemonHandle};

#[derive(Debug)]
pub enum DaemonError {
    StartError(String),
    PanicError(String),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::StartError(e) => write!(f, "daemon error while starting: {}", e),
            Self::PanicError(e) => write!(f, "daemon had a panic: {}", e),
        }
    }
}

// RevaultD can start only if a config path is given.
pub async fn start_daemon(config_path: PathBuf) -> Result<EmbeddedDaemon, DaemonError> {
    debug!("starting revaultd daemon");

    sodiumoxide::init().map_err(|_| DaemonError::StartError("sodiumoxide::init".to_string()))?;

    let config = Config::from_file(Some(config_path))
        .map_err(|e| DaemonError::StartError(format!("Error parsing config: {}", e)))?;

    let handle = DaemonHandle::start(config)
        .map_err(|e| DaemonError::StartError(format!("Error creating global state: {}", e)))?;
    Ok(EmbeddedDaemon { handle })
}

pub struct EmbeddedDaemon {
    handle: DaemonHandle,
}

impl std::fmt::Debug for EmbeddedDaemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonHandle").finish()
    }
}
