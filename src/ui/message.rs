use super::error::Error;
use crate::revaultd::{RevaultD, RevaultDError};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Install,
    Syncing(Result<f64, RevaultDError>),
    Synced(RevaultD),
    DaemonStarted(Result<RevaultD, Error>),
    Connected((Option<PathBuf>, Result<RevaultD, Error>)),
}
