use std::sync::Arc;

use super::{error::Error, menu::Menu};
use crate::revault::Role;
use crate::revaultd::{
    model::{RevocationTransactions, UnvaultTransaction, Vault, VaultStatus, VaultTransactions},
    RevaultD, RevaultDError,
};

#[derive(Debug, Clone)]
pub enum Message {
    Clipboard(String),
    Install,
    ChangeRole(Role),
    Syncing(Result<f64, RevaultDError>),
    Synced(Arc<RevaultD>),
    DaemonStarted(Result<Arc<RevaultD>, Error>),
    Vaults(Result<Vec<Vault>, RevaultDError>),
    Vault(VaultMessage),
    FilterVaults(VaultFilterMessage),
    BlockHeight(Result<u64, RevaultDError>),
    Connected(Result<Arc<RevaultD>, Error>),
    Menu(Menu),
    Next,
    Previous,
    DepositAddress(Result<bitcoin::Address, RevaultDError>),
    Deposit(usize, DepositMessage),
    Recipient(usize, RecipientMessage),
    Input(usize, InputMessage),
    AddRecipient,
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    ListOnchainTransaction,
    OnChainTransactions(Result<VaultTransactions, RevaultDError>),
    UnvaultTransaction(Result<UnvaultTransaction, RevaultDError>),
    Sign(SignMessage),
    Signed(Result<(), RevaultDError>),
    Select(String),
    Delegate(String),
}

#[derive(Debug, Clone)]
pub enum VaultFilterMessage {
    Status(Vec<VaultStatus>),
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    ChangeMethod,
    Sign,
    Success,
    SharingStatus(SignatureSharingStatus),
    Clipboard(String),
    PsbtEdited(String),
}

#[derive(Debug, Clone)]
pub enum SignatureSharingStatus {
    Unshared,
    Processing,
    Success,
}

#[derive(Debug, Clone)]
pub enum DepositMessage {
    RevocationTransactions(Result<RevocationTransactions, RevaultDError>),
    Sign(SignMessage),
    Signed(Result<(), RevaultDError>),
    /// Message ask for Deposit State to retry connecting to revaultd.
    Retry,
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
