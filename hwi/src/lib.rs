use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

pub mod app;

#[cfg(feature = "dummysigner")]
pub mod dummysigner;

#[cfg(feature = "specter")]
pub mod specter;

use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum HWIError {
    UnimplementedMethod,
    DeviceDisconnected,
    DeviceNotFound,
    Device(String),
}

impl std::fmt::Display for HWIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HWIError::UnimplementedMethod => write!(f, "unimplemented method"),
            HWIError::DeviceDisconnected => write!(f, "device disconnected"),
            HWIError::DeviceNotFound => write!(f, "device not found"),
            HWIError::Device(e) => write!(f, "{}", e),
        }
    }
}

/// HWI is the common Hardware Wallet Interface.
#[async_trait]
pub trait HWI: Debug {
    /// Check that the device is connected but not necessarily available.
    async fn is_connected(&mut self) -> Result<(), HWIError>;
    /// Sign a partially signed bitcoin transaction (PSBT).
    async fn sign_tx(&mut self, tx: &Psbt) -> Result<Psbt, HWIError>;
}
