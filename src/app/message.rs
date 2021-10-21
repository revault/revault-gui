use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;

use crate::{
    app::menu::Menu,
    daemon::{
        client::RevaultDError,
        model::{
            RevocationTransactions, ServersStatuses, SpendTransaction, SpendTx, UnvaultTransaction,
            Vault, VaultStatus, VaultTransactions,
        },
    },
    revault::Role,
};

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    StoppingDaemon(Result<(), RevaultDError>),
    Event(iced_native::Event),
    Clipboard(String),
    ChangeRole(Role),
    Vaults(Result<Vec<Vault>, RevaultDError>),
    SelectVault(String),
    DelegateVault(String),
    Vault(VaultMessage),
    FilterVaults(VaultFilterMessage),
    BlockHeight(Result<u64, RevaultDError>),
    ServerStatus(Result<ServersStatuses, RevaultDError>),
    Menu(Menu),
    Next,
    Previous,
    DepositAddress(Result<bitcoin::Address, RevaultDError>),
    Recipient(usize, RecipientMessage),
    Input(usize, InputMessage),
    AddRecipient,
    SpendTransaction(Result<SpendTransaction, RevaultDError>),
    SpendTransactions(Result<Vec<SpendTx>, RevaultDError>),
    SpendTx(SpendTxMessage),
    Emergency,
    EmergencyBroadcasted(Result<(), RevaultDError>),
}

#[derive(Debug, Clone)]
pub enum SpendTxMessage {
    FeerateEdited(String),
    PsbtEdited(String),
    Import,
    Generate,
    /// Select the SpendTxMessage with the given psbt.
    Select(Psbt),
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
    Broadcast,
    Broadcasted(Result<(), RevaultDError>),
    Update,
    Updated(Result<(), RevaultDError>),
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    Tick(Instant),
    ListOnchainTransaction,
    RevocationTransactions(Result<RevocationTransactions, RevaultDError>),
    OnChainTransactions(Result<VaultTransactions, RevaultDError>),
    UnvaultTransaction(Result<UnvaultTransaction, RevaultDError>),
    SelectDelegate,
    Delegate(SignMessage),
    Delegated(Result<(), RevaultDError>),
    SelectSecure,
    Secure(SignMessage),
    Secured(Result<(), RevaultDError>),
    SelectRevault,
    Revault,
    Revaulted(Result<(), RevaultDError>),
}

#[derive(Debug, Clone)]
pub enum VaultFilterMessage {
    Status(&'static [VaultStatus]),
}

#[derive(Debug, Clone)]
pub enum SignMessage {
    CheckConnection,
    Ping(Result<(), revault_hwi::Error>),
    SelectSign,
    Connected(Result<Arc<Mutex<revault_hwi::Channel>>, revault_hwi::Error>),
    Signed(Result<Box<Vec<Psbt>>, revault_hwi::Error>),
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
