use bitcoin::util::{bip32::DerivationPath, psbt::PartiallySignedTransaction as Psbt};
use std::fmt::Debug;

mod dummysigner;
use dummysigner::DummySigner;

#[derive(Debug)]
pub struct Channel {
    device: DummySigner,
}

impl Channel {
    pub async fn try_connect() -> Result<Channel, Error> {
        let device = DummySigner::try_connect("0.0.0.0:8080").await?;
        Ok(Channel { device })
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.device.ping().await
    }

    pub async fn sign_revocation_txs(
        &mut self,
        path: DerivationPath,
        emergency_tx: Psbt,
        emergency_unvault_tx: Psbt,
        cancel_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        self.device
            .sign_revocation_txs(path, emergency_tx, emergency_unvault_tx, cancel_tx)
            .await
    }

    pub async fn sign_unvault_tx(
        &mut self,
        path: DerivationPath,
        unvault_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        self.device.sign_unvault_tx(path, unvault_tx).await
    }

    pub async fn sign_spend_tx(
        &mut self,
        paths: Vec<DerivationPath>,
        spend_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        self.device.sign_spend_tx(paths, spend_tx).await
    }
}

#[derive(Debug, Clone)]
pub struct Error(String);
