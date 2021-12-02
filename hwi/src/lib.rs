use bitcoin::{
    blockdata::transaction::OutPoint, util::psbt::PartiallySignedTransaction as Psbt, Amount,
};
use std::fmt::Debug;

mod dummysigner;
mod specter;
use dummysigner::DummySigner;
use specter::{Specter, SpecterError};
use tokio::net::TcpStream;
use tokio_serial::SerialStream;

#[derive(Debug)]
pub enum Channel {
    DummySigner(DummySigner),
    SpecterSimulator(Specter<TcpStream>),
    Specter(Specter<SerialStream>),
}

impl Channel {
    pub async fn try_connect() -> Result<Channel, Error> {
        if let Ok(device) = DummySigner::try_connect("0.0.0.0:8080").await {
            return Ok(Self::DummySigner(device));
        }
        if let Ok(device) = Specter::try_connect_simulator("127.0.0.1:8789").await {
            return Ok(Self::SpecterSimulator(device));
        }

        if let Ok(device) = Specter::try_connect_serial().await {
            return Ok(Self::Specter(device));
        }
        Err(Error::DeviceNotFound)
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        match self {
            Self::DummySigner(signer) => signer.ping().await,
            Self::Specter(specter) => {
                specter.fingerprint().await.map_err(Error::from)?;
                Ok(())
            }
            Self::SpecterSimulator(specter) => {
                specter.fingerprint().await.map_err(Error::from)?;
                Ok(())
            }
        }
    }

    pub async fn sign_revocation_txs(
        &mut self,
        emergency_tx: Psbt,
        emergency_unvault_tx: Psbt,
        cancel_tx: Psbt,
    ) -> Result<(Psbt, Psbt, Psbt), Error> {
        match self {
            Self::DummySigner(dummy) => {
                dummy
                    .sign_revocation_txs(emergency_tx, emergency_unvault_tx, cancel_tx)
                    .await
            }
            Self::Specter(specter) => {
                let emergency_tx = specter
                    .sign_psbt(&emergency_tx)
                    .await
                    .map_err(Error::from)?;
                let emergency_unvault_tx = specter
                    .sign_psbt(&emergency_unvault_tx)
                    .await
                    .map_err(Error::from)?;
                let cancel_tx = specter.sign_psbt(&cancel_tx).await.map_err(Error::from)?;
                Ok((emergency_tx, emergency_unvault_tx, cancel_tx))
            }
            Self::SpecterSimulator(specter) => {
                let emergency_tx = specter
                    .sign_psbt(&emergency_tx)
                    .await
                    .map_err(Error::from)?;
                let emergency_unvault_tx = specter
                    .sign_psbt(&emergency_unvault_tx)
                    .await
                    .map_err(Error::from)?;
                let cancel_tx = specter.sign_psbt(&cancel_tx).await.map_err(Error::from)?;
                Ok((emergency_tx, emergency_unvault_tx, cancel_tx))
            }
        }
    }

    pub async fn sign_unvault_tx(&mut self, unvault_tx: Psbt) -> Result<Psbt, Error> {
        match self {
            Self::DummySigner(dummy) => dummy.sign_unvault_tx(unvault_tx).await,
            Self::Specter(specter) => specter.sign_psbt(&unvault_tx).await.map_err(|e| e.into()),
            Self::SpecterSimulator(specter) => {
                specter.sign_psbt(&unvault_tx).await.map_err(|e| e.into())
            }
        }
    }

    pub async fn sign_spend_tx(&mut self, spend_tx: Psbt) -> Result<Psbt, Error> {
        match self {
            Self::DummySigner(dummy) => dummy.sign_spend_tx(spend_tx).await,
            Self::Specter(specter) => specter.sign_psbt(&spend_tx).await.map_err(|e| e.into()),
            Self::SpecterSimulator(specter) => {
                specter.sign_psbt(&spend_tx).await.map_err(|e| e.into())
            }
        }
    }

    /// Secure a batch of deposits by giving the utxos to an hardware wallet storing the
    /// descriptors and deriving itself the revocation transactions.
    pub async fn secure_batch(
        &mut self,
        deposits: Vec<(OutPoint, Amount, u32)>,
    ) -> Result<Vec<(Psbt, Psbt, Psbt)>, Error> {
        match self {
            Self::DummySigner(dummy) => dummy.secure_batch(deposits).await,
            Self::Specter(_) | Self::SpecterSimulator(_) => Err(Error::UnimplementedMethod),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    DeviceDisconnected,
    DeviceNotFound,
    UnimplementedMethod,
    Device(String),
}

impl From<SpecterError> for Error {
    fn from(e: SpecterError) -> Error {
        Error::Device(e.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DeviceDisconnected => write!(f, "Hardware device disconnected"),
            Self::DeviceNotFound => write!(f, "Hardware device not found"),
            Self::UnimplementedMethod => write!(f, "Hardware device does not know the command"),
            Self::Device(e) => write!(f, "{}", e),
        }
    }
}
