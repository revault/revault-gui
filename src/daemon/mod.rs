use std::path::Path;
use std::process::Command;

use tracing::{debug, info};

pub mod client;
pub mod config;
pub mod model;

#[derive(Debug)]
pub struct StartDaemonError(String);
impl std::fmt::Display for StartDaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Revaultd error while starting: {}", self.0)
    }
}

// RevaultD can start only if a config path is given.
pub async fn start_daemon(
    config_path: &Path,
    revaultd_path: &Path,
) -> Result<(), StartDaemonError> {
    debug!("starting revaultd daemon");
    let mut child = Command::new(revaultd_path)
        .arg("--conf")
        .arg(config_path.to_path_buf().into_os_string().as_os_str())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| StartDaemonError(format!("Failed to launched revaultd: {}", e.to_string())))?;

    debug!("waiting for revaultd daemon status");

    let tries_timeout = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();

    while start.elapsed() < tries_timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    // FIXME: there should be a better way to collect the output...
                    let output = child.wait_with_output().unwrap();
                    return Err(StartDaemonError(format!(
                        "Error revaultd terminated with status: {} and stderr:\n{:?}",
                        status.to_string(),
                        String::from_utf8_lossy(&output.stderr),
                    )));
                } else {
                    info!("revaultd daemon started");
                    return Ok(());
                }
            }
            Ok(None) => continue,
            Err(e) => {
                return Err(StartDaemonError(format!(
                    "Child did not terminate: {}",
                    e.to_string()
                )));
            }
        }
    }

    Err(StartDaemonError(
        "Child did not terminate, do you have `daemon=false` in Revault conf?".to_string(),
    ))
}
