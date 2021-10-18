use std::path::PathBuf;
use std::thread::{Builder, JoinHandle};

use log::{debug, info};

pub mod client;
pub mod config;
pub mod model;

use revaultd::{
    common::config::Config,
    daemon::{daemon_main, RevaultD},
    revault_net::sodiumoxide,
    revault_tx::bitcoin::hashes::hex::ToHex,
};

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
pub async fn start_daemon(config_path: PathBuf) -> Result<(), DaemonError> {
    debug!("starting revaultd daemon");

    sodiumoxide::init().map_err(|_| DaemonError::StartError("sodiumoxide::init".to_string()))?;

    let config = Config::from_file(Some(config_path))
        .map_err(|e| DaemonError::StartError(format!("Error parsing config: {}", e)))?;

    let revaultd = RevaultD::from_config(config)
        .map_err(|e| DaemonError::StartError(format!("Error creating global state: {}", e)))?;

    info!(
        "Using Noise static public key: '{}'",
        revaultd.noise_pubkey().0.to_hex()
    );
    debug!(
        "Coordinator static public key: '{}'",
        revaultd.coordinator_noisekey.0.to_hex()
    );

    let handle: JoinHandle<_> = Builder::new()
        .spawn(|| {
            std::panic::set_hook(Box::new(move |panic_info| {
                let file = panic_info
                    .location()
                    .map(|l| l.file())
                    .unwrap_or_else(|| "'unknown'");
                let line = panic_info
                    .location()
                    .map(|l| l.line().to_string())
                    .unwrap_or_else(|| "'unknown'".to_string());

                let bt = backtrace::Backtrace::new();
                if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                    log::error!(
                        "panic occurred at line {} of file {}: {:?}\n{:?}",
                        line,
                        file,
                        s,
                        bt
                    );
                } else {
                    log::error!("panic occurred at line {} of file {}\n{:?}", line, file, bt);
                }
            }));

            daemon_main(revaultd);
        })
        .map_err(|e| DaemonError::StartError(format!("{}", e)))?;

    handle
        .join()
        .map_err(|e| DaemonError::PanicError(format!("{:?}", e)))?;
    Ok(())
}
