use std::sync::Arc;
use std::time::Instant;

use super::error::Error;
use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD, RevaultDError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Manager,
    Stakeholder,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Role::Manager => "Manager",
                Role::Stakeholder => "Stakeholder",
            }
        )
    }
}

impl Role {
    pub const ALL: [Role; 2] = [Role::Manager, Role::Stakeholder];
}

#[derive(Debug, Clone)]
pub enum Message {
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

#[derive(Debug, Clone)]
pub enum Menu {
    Home,
    History,
    Send,
}
