use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::sync::Arc;

use super::{error::Error, menu::Menu};
use crate::revault::Role;
use crate::revaultd::{
    model::{
        RevocationTransactions, SpendTransaction, SpendTx, UnvaultTransaction, Vault, VaultStatus,
        VaultTransactions,
    },
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
    SpendTransaction(Result<SpendTransaction, RevaultDError>),
    SpendTransactions(Result<Vec<SpendTx>, RevaultDError>),
    SpendTx(SpendTxMessage),
}

#[derive(Debug, Clone)]
pub enum SpendTxMessage {
    FeerateEdited(u32),
    PsbtEdited(String),
    Import,
    Generate,
    /// Select the SpendTxMessage with the given psbt.
    Select(Psbt),
    Updated(Result<(), RevaultDError>),
    Sign(SignMessage),
    Signed(Result<(), RevaultDError>),
    Inputs(Result<Vec<Vault>, RevaultDError>),
    SpendTransactions(Result<Vec<SpendTx>, RevaultDError>),
    SelectShare,
    SelectDelete,
    SelectSign,
    SelectBroadcast,
    Delete,
    Deleted(Result<(), RevaultDError>),
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
