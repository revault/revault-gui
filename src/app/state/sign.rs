use std::{collections::BTreeMap, str::FromStr, sync::Arc, time::Duration};

use bitcoin::{
    blockdata::transaction::OutPoint,
    util::{
        bip32::{Fingerprint, KeySource},
        psbt::PartiallySignedTransaction as Psbt,
    },
    Amount, PublicKey,
};
use tokio::sync::Mutex;

use iced::{time, Command, Element, Subscription};

use revault_hwi::{Channel, Error};

use crate::{
    app::{context::Context, message::SignMessage, view::sign::SignerView},
    daemon::{client::Client, model::Vault},
};

#[derive(Debug)]
pub struct SpendTransactionTarget {
    pub spend_tx: Psbt,
}

impl SpendTransactionTarget {
    /// Creates a new SpendTransactionTarget to sign with only the corresponding keys of the given
    /// xpubs. The bip32_derivation of the psbt is filtered to possess only the given xpub
    /// fingerprints.
    pub fn new(fingerprints: &Vec<Fingerprint>, mut spend_tx: Psbt) -> Self {
        for input in &mut spend_tx.inputs {
            let mut new_derivation: BTreeMap<PublicKey, KeySource> = BTreeMap::new();
            for (key, source) in &input.bip32_derivation {
                if fingerprints.contains(&source.0) {
                    new_derivation.insert(*key, source.clone());
                }
            }
            input.bip32_derivation = new_derivation;
        }
        Self { spend_tx }
    }
}

#[derive(Debug)]
pub struct Signer<T> {
    device: Device,
    processing: bool,
    signed: bool,
    error: Option<Error>,

    pub target: T,

    view: SignerView,
}

impl<T> Signer<T> {
    pub fn new(target: T) -> Self {
        Signer {
            device: Device::new(),
            processing: false,
            signed: false,
            error: None,
            target,
            view: SignerView::new(),
        }
    }

    pub fn signed(&self) -> bool {
        self.signed
    }

    pub fn subscription(&self) -> Subscription<SignMessage> {
        if !self.signed && !self.processing {
            self.device.subscription()
        } else {
            Subscription::none()
        }
    }

    pub fn view<C: Client>(&mut self, ctx: &Context<C>) -> Element<SignMessage> {
        self.view.view(
            ctx,
            self.device.is_connected(),
            self.processing,
            self.signed,
        )
    }
}

impl Signer<SpendTransactionTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::SelectSign => {
                self.processing = true;
                return Command::perform(
                    self.device
                        .clone()
                        .sign_spend_tx(self.target.spend_tx.clone()),
                    |tx| SignMessage::PsbtSigned(tx.map(Box::new)),
                );
            }
            SignMessage::PsbtSigned(res) => {
                self.processing = false;
                match res {
                    Ok(tx) => {
                        if tx.global.unsigned_tx.txid()
                            == self.target.spend_tx.global.unsigned_tx.txid()
                        {
                            self.signed = true;
                            self.target.spend_tx = *tx;
                        }
                    }
                    Err(e) => {
                        log::info!("{:?}", e);
                        self.error = Some(e);
                    }
                }
            }
            _ => return self.device.update(message),
        };
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    channel: Option<Arc<Mutex<Channel>>>,
}

impl Device {
    pub fn new() -> Self {
        Device { channel: None }
    }

    pub fn is_connected(&self) -> bool {
        self.channel.is_some()
    }

    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::Ping(res) => {
                if res.is_err() {
                    self.channel = None;
                }
            }
            SignMessage::CheckConnection => {
                if let Some(channel) = self.channel.clone() {
                    return Command::perform(
                        async move { channel.lock().await.ping().await },
                        SignMessage::Ping,
                    );
                } else {
                    return Command::perform(Channel::try_connect(), |res| {
                        SignMessage::Connected(res.map(|channel| Arc::new(Mutex::new(channel))))
                    });
                }
            }
            SignMessage::Connected(Ok(channel)) => self.channel = Some(channel),
            _ => {}
        };
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<SignMessage> {
        time::every(Duration::from_secs(1)).map(|_| SignMessage::CheckConnection)
    }

    pub async fn sign_revocation_txs(
        self,
        emergency_tx: Psbt,
        emergency_unvault_tx: Psbt,
        cancel_tx: Psbt,
    ) -> Result<(Psbt, Psbt, Psbt), Error> {
        if let Some(channel) = self.channel {
            channel
                .lock()
                .await
                .sign_revocation_txs(emergency_tx, emergency_unvault_tx, cancel_tx)
                .await
        } else {
            Err(Error::DeviceDisconnected)
        }
    }

    pub async fn sign_unvault_tx(self, unvault_tx: Psbt) -> Result<Psbt, Error> {
        if let Some(channel) = self.channel {
            channel.lock().await.sign_unvault_tx(unvault_tx).await
        } else {
            Err(Error::DeviceDisconnected)
        }
    }

    pub async fn sign_spend_tx(self, spend_tx: Psbt) -> Result<Psbt, Error> {
        if let Some(channel) = self.channel {
            channel.lock().await.sign_spend_tx(spend_tx).await
        } else {
            Err(Error::DeviceDisconnected)
        }
    }

    pub async fn secure_batch(
        self,
        deposits: &Vec<Vault>,
    ) -> Result<Vec<(Psbt, Psbt, Psbt)>, Error> {
        if let Some(channel) = self.channel {
            let utxos: Vec<(OutPoint, Amount, u32)> = deposits
                .iter()
                .map(|deposit| {
                    (
                        OutPoint::from_str(&deposit.outpoint())
                            .expect("OutPoint has the good format"),
                        Amount::from_sat(deposit.amount),
                        deposit.derivation_index,
                    )
                })
                .collect();
            channel.lock().await.secure_batch(utxos).await
        } else {
            Err(Error::DeviceDisconnected)
        }
    }

    pub async fn delegate_batch(self, vaults: &Vec<Vault>) -> Result<Vec<Psbt>, Error> {
        if let Some(channel) = self.channel {
            let utxos: Vec<(OutPoint, Amount, u32)> = vaults
                .iter()
                .map(|vault| {
                    (
                        OutPoint::from_str(&vault.outpoint())
                            .expect("OutPoint has the good format"),
                        Amount::from_sat(vault.amount),
                        vault.derivation_index,
                    )
                })
                .collect();
            channel.lock().await.delegate_batch(utxos).await
        } else {
            Err(Error::DeviceDisconnected)
        }
    }
}
