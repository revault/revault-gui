use super::error::Error;
use crate::revaultd::{RevaultD, RevaultDError};

#[derive(Debug, Clone)]
pub enum Message {
    Install,
    Syncing(Result<f64, RevaultDError>),
    Synced(RevaultD),
    DaemonStarted(Result<RevaultD, Error>),
    Connected(Result<RevaultD, Error>),
}
