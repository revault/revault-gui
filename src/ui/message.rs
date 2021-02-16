use std::sync::Arc;
use std::time::Instant;

use super::{error::Error, menu::Menu};
use crate::revault::{Role, TransactionKind};
use crate::revaultd::{
    model::{RevocationTransactions, Vault, VaultTransactions},
    RevaultD, RevaultDError,
};
use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

#[derive(Debug, Clone)]
pub enum Message {
    Clipboard(String),
    Install,
    ChangeRole(Role),
    Syncing(Result<f64, RevaultDError>),
    Synced(Arc<RevaultD>),
    Tick(Instant),
    DaemonStarted(Result<Arc<RevaultD>, Error>),
    Vaults(Result<Vec<Vault>, RevaultDError>),
    VaultsWithTransactions(Result<Vec<(Vault, VaultTransactions)>, RevaultDError>),
    SelectVault(String),
    BlockHeight(Result<u64, RevaultDError>),
    Connected(Result<Arc<RevaultD>, Error>),
    Menu(Menu),
    Next,
    Previous,
    Deposit(usize, DepositMessage),
    Recipient(usize, RecipientMessage),
    Input(usize, InputMessage),
    None,
    AddRecipient,
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    ChangeMethod,
    Sign,
    Clipboard(String),
    PsbtEdited(String),
}

#[derive(Debug, Clone)]
pub enum DepositMessage {
    RevocationTransactions(Result<RevocationTransactions, RevaultDError>),
    Sign(SignMessage),
    Signed(Result<(), RevaultDError>),
}

#[derive(Debug, Clone)]
pub enum InputMessage {
    Selected(bool),
}

#[derive(Debug, Clone)]
pub enum RecipientMessage {
    Delete,
    AddressEdited(String),
    AmountEdited(String),
}
