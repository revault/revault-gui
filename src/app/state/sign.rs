use bitcoin::util::{
    bip32::{ChildNumber, DerivationPath},
    psbt::PartiallySignedTransaction as Psbt,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use iced::{time, Command, Element, Subscription};

use crate::{
    app::{context::Context, message::SignMessage, view::sign::SignerView},
    hw,
};

#[derive(Debug)]
pub struct RevocationTransactionsTarget {
    pub derivation_index: u32,
    pub cancel_tx: Psbt,
    pub emergency_tx: Psbt,
    pub emergency_unvault_tx: Psbt,
}

#[derive(Debug)]
pub struct UnvaultTransactionTarget {
    pub derivation_index: u32,
    pub unvault_tx: Psbt,
}

#[derive(Debug)]
pub struct SpendTransactionTarget {
    pub derivation_indexes: Vec<u32>,
    pub spend_tx: Psbt,
}

#[derive(Debug)]
pub struct Signer<T> {
    channel: Option<Arc<Mutex<hw::Channel>>>,
    processing: bool,
    signed: bool,
    error: Option<hw::Error>,

    pub target: T,

    view: SignerView,
}

impl<T> Signer<T> {
    pub fn new(target: T) -> Self {
        Signer {
            channel: None,
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

    pub fn check_channel(&mut self) -> Command<SignMessage> {
        if let Some(channel) = &self.channel {
            Command::perform(ping(channel.clone()), SignMessage::Ping)
        } else {
            Command::perform(hw::Channel::try_connect(), |res| {
                SignMessage::Connected(res.map(|channel| Arc::new(Mutex::new(channel))))
            })
        }
    }

    pub fn update_channel(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::Ping(res) => {
                if res.is_err() {
                    self.channel = None;
                }
            }
            SignMessage::CheckConnection => return self.check_channel(),
            SignMessage::Connected(Ok(channel)) => self.channel = Some(channel),
            _ => {}
        };
        Command::none()
    }

    pub fn subscription(&self) -> Subscription<SignMessage> {
        if !self.signed {
            time::every(Duration::from_secs(1)).map(|_| SignMessage::CheckConnection)
        } else {
            Subscription::none()
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<SignMessage> {
        self.view
            .view(ctx, self.channel.is_some(), self.processing, self.signed)
    }
}

pub async fn ping(channel: Arc<Mutex<hw::Channel>>) -> Result<(), hw::Error> {
    channel.clone().lock().await.ping().await
}

impl Signer<SpendTransactionTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::SelectSign => {
                if let Some(channel) = &self.channel {
                    self.processing = true;
                    return Command::perform(
                        sign_spend_tx(
                            channel.clone(),
                            self.target
                                .derivation_indexes
                                .iter()
                                .map(|index| {
                                    DerivationPath::master().child(
                                        ChildNumber::from_normal_idx(index.clone())
                                            .expect("index will be not too high"),
                                    )
                                })
                                .collect(),
                            self.target.spend_tx.clone(),
                        ),
                        SignMessage::Signed,
                    );
                }
            }
            SignMessage::Signed(res) => {
                self.processing = false;
                match res {
                    Ok(txs) => {
                        self.signed = true;
                        let txs = *txs;
                        if let Some(tx) = txs.into_iter().find(|psbt| {
                            psbt.global.unsigned_tx.txid()
                                == self.target.spend_tx.global.unsigned_tx.txid()
                        }) {
                            self.target.spend_tx = tx;
                        }
                    }
                    Err(e) => {
                        tracing::info!("{:?}", e);
                        self.error = Some(e);
                    }
                }
            }
            _ => return self.update_channel(message),
        };
        Command::none()
    }
}

pub async fn sign_spend_tx(
    channel: Arc<Mutex<hw::Channel>>,
    paths: Vec<DerivationPath>,
    spend_tx: Psbt,
) -> Result<Box<Vec<Psbt>>, hw::Error> {
    channel
        .clone()
        .lock()
        .await
        .sign_spend_tx(paths, spend_tx)
        .await
}

impl Signer<UnvaultTransactionTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::SelectSign => {
                if let Some(channel) = &self.channel {
                    self.processing = true;
                    return Command::perform(
                        sign_unvault_tx(
                            channel.clone(),
                            DerivationPath::master().child(
                                ChildNumber::from_normal_idx(self.target.derivation_index.clone())
                                    .expect("index will be not too high"),
                            ),
                            self.target.unvault_tx.clone(),
                        ),
                        SignMessage::Signed,
                    );
                }
            }
            SignMessage::Signed(res) => {
                self.processing = false;
                match res {
                    Ok(txs) => {
                        self.signed = true;
                        let txs = *txs;
                        if let Some(tx) = txs.into_iter().find(|psbt| {
                            psbt.global.unsigned_tx.txid()
                                == self.target.unvault_tx.global.unsigned_tx.txid()
                        }) {
                            self.target.unvault_tx = tx;
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            _ => return self.update_channel(message),
        };
        Command::none()
    }
}

pub async fn sign_unvault_tx(
    channel: Arc<Mutex<hw::Channel>>,
    path: DerivationPath,
    unvault_tx: Psbt,
) -> Result<Box<Vec<Psbt>>, hw::Error> {
    channel
        .clone()
        .lock()
        .await
        .sign_unvault_tx(path, unvault_tx)
        .await
}

impl Signer<RevocationTransactionsTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::SelectSign => {
                if let Some(channel) = &self.channel {
                    self.processing = true;
                    return Command::perform(
                        sign_revocation_txs(
                            channel.clone(),
                            DerivationPath::master().child(
                                ChildNumber::from_normal_idx(self.target.derivation_index.clone())
                                    .expect("index will be not too high"),
                            ),
                            self.target.emergency_tx.clone(),
                            self.target.emergency_unvault_tx.clone(),
                            self.target.cancel_tx.clone(),
                        ),
                        SignMessage::Signed,
                    );
                }
            }
            SignMessage::Signed(res) => {
                self.processing = false;
                match res {
                    Ok(txs) => {
                        self.signed = true;
                        let txs = *txs;
                        if let Some(tx) = txs.iter().find(|psbt| {
                            psbt.global.unsigned_tx.txid()
                                == self.target.cancel_tx.global.unsigned_tx.txid()
                        }) {
                            self.target.cancel_tx = tx.clone();
                        }
                        if let Some(tx) = txs.iter().find(|psbt| {
                            psbt.global.unsigned_tx.txid()
                                == self.target.emergency_tx.global.unsigned_tx.txid()
                        }) {
                            self.target.emergency_tx = tx.clone();
                        }
                        if let Some(tx) = txs.into_iter().find(|psbt| {
                            psbt.global.unsigned_tx.txid()
                                == self.target.emergency_unvault_tx.global.unsigned_tx.txid()
                        }) {
                            self.target.emergency_unvault_tx = tx;
                        }
                    }
                    Err(e) => self.error = Some(e),
                }
            }
            _ => return self.update_channel(message),
        };
        Command::none()
    }
}

pub async fn sign_revocation_txs(
    channel: Arc<Mutex<hw::Channel>>,
    path: DerivationPath,
    emergency_tx: Psbt,
    emergency_unvault_tx: Psbt,
    cancel_tx: Psbt,
) -> Result<Box<Vec<Psbt>>, hw::Error> {
    channel
        .clone()
        .lock()
        .await
        .sign_revocation_txs(path, emergency_tx, emergency_unvault_tx, cancel_tx)
        .await
}
