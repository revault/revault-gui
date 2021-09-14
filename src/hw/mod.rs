use bitcoin::util::{bip32::DerivationPath, psbt::PartiallySignedTransaction as Psbt};
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
        Err(Error("Failed to find device".to_string()))
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
        path: DerivationPath,
        emergency_tx: Psbt,
        emergency_unvault_tx: Psbt,
        cancel_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        match self {
            Self::DummySigner(dummy) => {
                dummy
                    .sign_revocation_txs(path, emergency_tx, emergency_unvault_tx, cancel_tx)
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
                Ok(Box::new(vec![
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                ]))
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
                Ok(Box::new(vec![
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                ]))
            }
        }
    }

    pub async fn sign_unvault_tx(
        &mut self,
        path: DerivationPath,
        unvault_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        match self {
            Self::DummySigner(dummy) => dummy.sign_unvault_tx(path, unvault_tx).await,
            Self::Specter(specter) => {
                let unvault_tx = specter.sign_psbt(&unvault_tx).await?;
                Ok(Box::new(vec![unvault_tx]))
            }
            Self::SpecterSimulator(specter) => {
                let unvault_tx = specter.sign_psbt(&unvault_tx).await?;
                Ok(Box::new(vec![unvault_tx]))
            }
        }
    }

    pub async fn sign_spend_tx(
        &mut self,
        paths: Vec<DerivationPath>,
        spend_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        match self {
            Self::DummySigner(dummy) => dummy.sign_spend_tx(paths, spend_tx).await,
            Self::Specter(specter) => {
                let spend_tx = specter.sign_psbt(&spend_tx).await?;
                Ok(Box::new(vec![spend_tx]))
            }
            Self::SpecterSimulator(specter) => {
                let spend_tx = specter.sign_psbt(&spend_tx).await?;
                Ok(Box::new(vec![spend_tx]))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Error(String);

impl From<SpecterError> for Error {
    fn from(e: SpecterError) -> Error {
        Error(e.to_string())
    }
}
