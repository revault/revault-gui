use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::sync::Arc;

use iced::{Command, Element, Subscription};

use crate::{
    app::{
        message::SignMessage,
        view::{sign::SignerView, Context},
    },
    hw,
};

#[derive(Debug)]
pub struct RevocationTransactionsTarget {
    pub cancel_tx: Psbt,
    pub emergency_tx: Psbt,
    pub emergency_unvault_tx: Psbt,
}

#[derive(Debug)]
pub struct UnvaultTransactionTarget {
    pub unvault_tx: Psbt,
}

#[derive(Debug)]
pub struct SpendTransactionTarget {
    pub spend_tx: Psbt,
}

#[derive(Debug)]
pub struct Signer<T> {
    channel: Option<Arc<hw::Channel>>,
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

    pub fn check_channel(&self) -> Command<SignMessage> {
        if self.channel.is_none() {
            Command::perform(hw::Channel::try_connect(), |res| {
                SignMessage::Connected(res.map(|channel| Arc::new(channel)))
            })
        } else {
            Command::none()
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<SignMessage> {
        self.view.view(ctx, self.channel.is_some(), self.processing)
    }
}

impl Signer<SpendTransactionTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::Connected(Ok(channel)) => self.channel = Some(channel),
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
                    Err(e) => self.error = Some(e),
                }
            }
            _ => {}
        };
        Command::none()
    }
}

impl Signer<UnvaultTransactionTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
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
            _ => {}
        };
        Command::none()
    }
}

impl Signer<RevocationTransactionsTarget> {
    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
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
            _ => {}
        };
        Command::none()
    }
}
