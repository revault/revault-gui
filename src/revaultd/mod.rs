use std::fmt::Debug;
use std::path::PathBuf;
use std::process::Command;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod config;
use config::Config;
mod client;
use client::Client;

#[derive(Debug)]
pub struct RevaultDError(std::string::String);

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Revauld error: {}", self.0)
    }
}

pub struct RevaultD {
    client: Client,
    config: Config,
}

impl RevaultD {
    pub fn new(config_path: PathBuf) -> Result<RevaultD, RevaultDError> {
        let config = Config::from_file(&config_path).map_err(|e| {
            RevaultDError(format!("Failed to read revaultd config: {}", e.to_string()))
        })?;

        let socket_path = config.socket_path().map_err(|e| {
            RevaultDError(format!(
                "Failed to find revaultd socket path: {}",
                e.to_string()
            ))
        })?;

        let client = Client::new(socket_path);
        let revaultd = RevaultD { client, config };

        log::debug!("Connecting to revaultd");

        revaultd.get_info().map_err(|e| {
            RevaultDError(format!(
                "Failed to connect to revaultd with socket path: {}",
                e.to_string(),
            ))
        })?;

        log::info!("Connected to revaultd");

        Ok(revaultd)
    }

    /// Generic call function for RPC calls.
    fn call<T: Serialize + Debug, U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        input: T,
    ) -> Result<U, client::error::Error> {
        self.client
            .send_request(method, input)
            .and_then(|res| res.into_result())
    }

    pub fn get_info(&self) -> Result<GetInfoResponse, client::error::Error> {
        self.call("getinfo", GetInfoRequest {})
    }
}

/// getinfo

/// getinfo request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInfoRequest {}

/// getinfo response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInfoResponse {
    pub blockheight: u64,
    pub network: String,
    pub sync: u32,
    pub version: String,
}

// RevaultD can start only if a config path is given.
pub fn start_daemon(config_path: PathBuf) -> Result<(), RevaultDError> {
    log::debug!("starting revaultd daemon");
    let child = Command::new("revaultd")
        .arg("--conf")
        .arg(config_path.into_os_string().as_os_str())
        .spawn()
        .map_err(|e| RevaultDError(format!("Failed to launched revaultd: {}", e.to_string())))?;

    log::debug!("waiting for revaultd daemon status");

    // daemon binary should fork and then terminate.
    let output = child
        .wait_with_output()
        .map_err(|e| RevaultDError(format!("Child did not terminate: {}", e.to_string())))?;

    if !output.status.success() {
        return Err(RevaultDError(format!(
            "Error revaultd terminated with status: {} and stderr: {}",
            output.status.to_string(),
            String::from_utf8_lossy(&output.stderr),
        )));
    }

    log::info!("revaultd daemon started");

    Ok(())
}
