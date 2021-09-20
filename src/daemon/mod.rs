use std::path::PathBuf;
use std::thread::{Builder, JoinHandle};

use tracing::{debug, info};

pub mod client;
pub mod config;
pub mod model;

use revaultd::{
    common::config::Config,
    daemon::{daemon_main, setup_logger, setup_panic_hook, RevaultD},
    revault_net::sodiumoxide,
    revault_tx::bitcoin::hashes::hex::ToHex,
};

#[derive(Debug)]
pub struct StartDaemonError(String);
impl std::fmt::Display for StartDaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Revaultd error while starting: {}", self.0)
    }
}

// RevaultD can start only if a config path is given.
pub async fn start_daemon(config_path: PathBuf) -> Result<JoinHandle<()>, StartDaemonError> {
    debug!("starting revaultd daemon");

    sodiumoxide::init().map_err(|_| StartDaemonError("sodiumoxide::init".to_string()))?;

    let config = Config::from_file(Some(config_path))
        .map_err(|e| StartDaemonError(format!("Error parsing config: {}", e)))?;

    setup_logger(config.log_level)
        .map_err(|e| StartDaemonError(format!("Error setting up logger: {}", e)))?;

    let revaultd = RevaultD::from_config(config)
        .map_err(|e| StartDaemonError(format!("Error creating global state: {}", e)))?;

    info!(
        "Using Noise static public key: '{}'",
        revaultd.noise_pubkey().0.to_hex()
    );
    debug!(
        "Coordinator static public key: '{}'",
        revaultd.coordinator_noisekey.0.to_hex()
    );

    setup_panic_hook();

    let handle: JoinHandle<_> = Builder::new()
        .spawn(|| {
            daemon_main(revaultd);
        })
        .map_err(|e| StartDaemonError(format!("{}", e)))?;
    Ok(handle)
}
