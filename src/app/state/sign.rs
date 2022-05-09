use std::{collections::BTreeMap, sync::Arc, time::Duration};

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

use revault_hwi::{app::revault::RevaultHWI, HWIError};

use crate::{
    app::{context::Context, error::Error, message::SignMessage, view::sign::SignerView},
    daemon::model::{outpoint, Vault},
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

    pub error: Option<Error>,
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

    pub fn view(&mut self, ctx: &Context) -> Element<SignMessage> {
        self.view.view(
            ctx,
            self.device.is_connected(),
            self.processing,
            self.signed,
        )
    }
}

impl Signer<SpendTransactionTarget> {
    pub fn update(&mut self, ctx: &Context, message: SignMessage) -> Command<SignMessage> {
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
                            let user_manager_xpub =
                                ctx.config.daemon.manager_config.as_ref().unwrap().xpub;
                            for input in &tx.inputs {
                                if !input.partial_sigs.keys().any(|key| {
                                    input
                                        .bip32_derivation
                                        .get(key)
                                        .map(|(fingerprint, _)| {
                                            user_manager_xpub.fingerprint() == *fingerprint
                                        })
                                        .unwrap_or(false)
                                }) {
                                    log::info!("Hardware wallet did not sign the spend tx");
                                    self.error = Some(HWIError::DeviceDidNotSign.into());
                                    return Command::none();
                                }
                            }
                            self.signed = true;
                            self.target.spend_tx = *tx;
                        }
                    }
                    Err(e) => {
                        log::info!("{:?}", e);
                        self.error = Some(e.into());
                    }
                }
            }
            _ => return self.device.update(&ctx, message),
        };
        Command::none()
    }
}

#[derive(Debug, Clone)]
pub struct Device {
    channel: Option<Arc<Mutex<Box<dyn RevaultHWI + Send>>>>,
}

impl Device {
    pub fn new() -> Self {
        Device { channel: None }
    }

    pub fn is_connected(&self) -> bool {
        self.channel.is_some()
    }

    pub fn update(&mut self, ctx: &Context, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::Ping(res) => {
                if res.is_err() {
                    self.channel = None;
                }
            }
            SignMessage::CheckConnection => {
                if let Some(channel) = self.channel.clone() {
                    return Command::perform(
                        async move { channel.lock().await.is_connected().await },
                        SignMessage::Ping,
                    );
                } else {
                    let connect = &ctx.hardware_wallet;
                    return Command::perform(connect(), |res| {
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
        cancel_txs: [Psbt; 5],
    ) -> Result<(Psbt, Psbt, [Psbt; 5]), HWIError> {
        if let Some(channel) = self.channel {
            channel
                .lock()
                .await
                .sign_revocation_txs(&emergency_tx, &emergency_unvault_tx, &cancel_txs)
                .await
        } else {
            Err(HWIError::DeviceDisconnected)
        }
    }

    pub async fn sign_unvault_tx(self, unvault_tx: Psbt) -> Result<Psbt, HWIError> {
        if let Some(channel) = self.channel {
            channel.lock().await.sign_unvault_tx(&unvault_tx).await
        } else {
            Err(HWIError::DeviceDisconnected)
        }
    }

    pub async fn sign_spend_tx(self, spend_tx: Psbt) -> Result<Psbt, HWIError> {
        if let Some(channel) = self.channel {
            let mut res = channel.lock().await;
            return res.sign_tx(&spend_tx).await;
        } else {
            Err(HWIError::DeviceDisconnected)
        }
    }

    pub async fn secure_batch(
        self,
        deposits: &Vec<Vault>,
    ) -> Result<Vec<(Psbt, Psbt, [Psbt; 5])>, HWIError> {
        if let Some(channel) = self.channel {
            let utxos: Vec<(OutPoint, Amount, u32)> = deposits
                .iter()
                .map(|deposit| {
                    (
                        outpoint(deposit),
                        deposit.amount,
                        deposit.derivation_index.into(),
                    )
                })
                .collect();
            channel.lock().await.create_vaults(&utxos).await
        } else {
            Err(HWIError::DeviceDisconnected)
        }
    }

    pub async fn delegate_batch(self, vaults: &Vec<Vault>) -> Result<Vec<Psbt>, HWIError> {
        if let Some(channel) = self.channel {
            let utxos: Vec<(OutPoint, Amount, u32)> = vaults
                .iter()
                .map(|vault| (outpoint(vault), vault.amount, vault.derivation_index.into()))
                .collect();
            channel.lock().await.delegate_vaults(&utxos).await
        } else {
            Err(HWIError::DeviceDisconnected)
        }
    }
}
