use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, OutPoint};
use revault_hwi::{app::revault::RevaultHWI, HWIError};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    app::{error::Error, menu::Menu},
    daemon::{
        model::{
            HistoryEvent, HistoryEventKind, ServersStatuses, SpendTx, Vault, VaultStatus,
            VaultTransactions,
        },
        RevaultDError,
    },
    revault::Role,
};

#[derive(Debug, Clone)]
pub enum Message {
    Reload,
    Tick,
    Event(iced_native::Event),
    Clipboard(String),
    ChangeRole(Role),
    Vaults(Result<Vec<Vault>, RevaultDError>),
    SelectVault(OutPoint),
    SelectHistoryEvent(usize),
    DelegateVault(OutPoint),
    Sign(SignMessage),
    DepositsSecured(Result<Vec<OutPoint>, Error>),
    VaultsDelegated(Result<Vec<OutPoint>, Error>),
    Vault(VaultMessage),
    FilterVaults(VaultFilterMessage),
    BlockHeight(Result<i32, RevaultDError>),
    ServerStatus(Result<ServersStatuses, RevaultDError>),
    HistoryEvents(Result<Vec<HistoryEvent>, RevaultDError>),
    HistoryEvent(HistoryEventMessage),
    FilterHistoryEvents(Option<HistoryEventKind>),
    Menu(Menu),
    Next,
    Previous,
    DepositAddress(Result<bitcoin::Address, RevaultDError>),
    Recipient(usize, RecipientMessage),
    Input(usize, InputMessage),
    AddRecipient,
    SpendTransaction(Result<(Psbt, u64), RevaultDError>),
    SpendTransactions(Result<Vec<SpendTx>, RevaultDError>),
    SpendTx(SpendTxMessage),
    Emergency,
    EmergencyBroadcasted(Result<(), RevaultDError>),
    Close,
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
    SelectDelete,
    UnselectDelete,
    Delete,
    Deleted(Result<(), RevaultDError>),
    Broadcast,
    Broadcasted(Result<(), RevaultDError>),
    Update,
    Updated(Result<(), RevaultDError>),
    WithPriority(bool),
}

#[derive(Debug, Clone)]
pub enum HistoryEventMessage {
    OnChainTransactions(Result<Vec<VaultTransactions>, RevaultDError>),
}

#[derive(Debug, Clone)]
pub enum VaultMessage {
    ListOnchainTransaction,
    OnChainTransactions(Result<VaultTransactions, RevaultDError>),
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
    Ping(Result<(), HWIError>),
    SelectSign,
    Connected(Result<Arc<Mutex<Box<dyn RevaultHWI + Send>>>, HWIError>),
    RevocationTxsSigned(Result<Box<Vec<(Psbt, Psbt, Psbt)>>, HWIError>),
    PsbtSigned(Result<Box<Psbt>, HWIError>),
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
