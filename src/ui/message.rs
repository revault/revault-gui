use std::sync::Arc;
use std::time::Instant;

use super::{error::Error, menu::Menu};
use crate::revault::Role;
use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD, RevaultDError,
};

#[derive(Debug, Clone)]
pub enum Message {
    Clipboard(String),
    Install,
    ChangeRole(Role),
    Syncing(Result<f64, RevaultDError>),
    Synced(Arc<RevaultD>),
    Tick(Instant),
    DaemonStarted(Result<Arc<RevaultD>, Error>),
    Vaults(Result<Vec<(Vault, VaultTransactions)>, RevaultDError>),
    SelectVault(String),
    BlockHeight(Result<u64, RevaultDError>),
    Connected(Result<Arc<RevaultD>, Error>),
    Menu(Menu),
    Next,
    Previous,
    Recipient(usize, RecipientMessage),
    Input(usize, InputMessage),
    None,
    AddRecipient,
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
