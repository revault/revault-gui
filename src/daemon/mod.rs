pub mod client;
pub mod embedded;
pub mod model;

use std::collections::BTreeMap;
use std::convert::From;
use std::fmt::Debug;
use std::io::ErrorKind;

use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, OutPoint, Txid};
use revaultd::commands::CommandError;

use model::*;

#[derive(Debug, Clone)]
pub enum RevaultDError {
    /// Something was wrong with the request.
    Rpc(i32, String),
    /// Something was wrong with the communication.
    Transport(Option<ErrorKind>, String),
    /// Something unexpected happened.
    Unexpected(String),
    /// No response.
    NoAnswer,
    // Error at start up.
    Start(String),
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Rpc(code, e) => write!(f, "Revaultd error rpc call: [{:?}] {}", code, e),
            Self::NoAnswer => write!(f, "Revaultd returned no answer"),
            Self::Transport(kind, e) => write!(f, "Revaultd transport error: [{:?}] {}", kind, e),
            Self::Unexpected(e) => write!(f, "Revaultd unexpected error: {}", e),
            Self::Start(e) => write!(f, "Revaultd did not start: {}", e),
        }
    }
}

impl From<CommandError> for RevaultDError {
    fn from(error: CommandError) -> Self {
        RevaultDError::Rpc(error.code() as i32, error.to_string())
    }
}

pub trait Daemon: Debug {
    fn is_external(&self) -> bool;

    fn stop(&mut self) -> Result<(), RevaultDError>;

    fn get_deposit_address(&self) -> Result<bitcoin::Address, RevaultDError>;

    fn get_info(&self) -> Result<GetInfoResult, RevaultDError>;

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&[OutPoint]>,
    ) -> Result<Vec<Vault>, RevaultDError>;

    fn list_onchain_transactions(
        &self,
        outpoints: &[OutPoint],
    ) -> Result<Vec<VaultTransactions>, RevaultDError>;

    fn get_revocation_txs(
        &self,
        outpoint: &OutPoint,
    ) -> Result<RevocationTransactions, RevaultDError>;

    fn set_revocation_txs(
        &self,
        outpoint: &OutPoint,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(), RevaultDError>;

    fn get_unvault_tx(&self, outpoint: &OutPoint) -> Result<Psbt, RevaultDError>;

    fn set_unvault_tx(&self, outpoint: &OutPoint, unvault_tx: &Psbt) -> Result<(), RevaultDError>;

    fn get_spend_tx(
        &self,
        inputs: &[OutPoint],
        outputs: &BTreeMap<bitcoin::Address, u64>,
        feerate: u64,
    ) -> Result<Psbt, RevaultDError>;

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError>;

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<Vec<SpendTx>, RevaultDError>;

    fn delete_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError>;

    fn broadcast_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError>;

    fn revault(&self, outpoint: &OutPoint) -> Result<(), RevaultDError>;

    fn emergency(&self) -> Result<(), RevaultDError>;

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError>;

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u32,
        end: u32,
        limit: u64,
    ) -> Result<Vec<HistoryEvent>, RevaultDError>;
}
