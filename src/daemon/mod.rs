pub mod client;
pub mod embedded;
pub mod model;

use std::collections::HashMap;
use std::io::ErrorKind;

use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

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
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Rpc(code, e) => write!(f, "Revaultd error rpc call: [{:?}] {}", code, e),
            Self::NoAnswer => write!(f, "Revaultd returned no answer"),
            Self::Transport(kind, e) => write!(f, "Revaultd transport error: [{:?}] {}", kind, e),
            Self::Unexpected(e) => write!(f, "Revaultd unexpected error: {}", e),
        }
    }
}

pub trait Daemon {
    fn get_deposit_address(&self) -> Result<DepositAddress, RevaultDError>;

    fn get_info(&self) -> Result<GetInfoResponse, RevaultDError>;

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&Vec<String>>,
    ) -> Result<ListVaultsResponse, RevaultDError>;

    fn list_onchain_transactions(
        &self,
        outpoints: Option<Vec<String>>,
    ) -> Result<ListOnchainTransactionsResponse, RevaultDError>;

    fn get_revocation_txs(&self, outpoint: &str) -> Result<RevocationTransactions, RevaultDError>;

    fn set_revocation_txs(
        &self,
        outpoint: &str,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(), RevaultDError>;

    fn get_unvault_tx(&self, outpoint: &str) -> Result<UnvaultTransaction, RevaultDError>;

    fn set_unvault_tx(&self, outpoint: &str, unvault_tx: &Psbt) -> Result<(), RevaultDError>;

    fn get_spend_tx(
        &self,
        inputs: &[String],
        outputs: &HashMap<String, u64>,
        feerate: &u32,
    ) -> Result<SpendTransaction, RevaultDError>;

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError>;

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<ListSpendTransactionsResponse, RevaultDError>;

    fn delete_spend_tx(&self, txid: &str) -> Result<(), RevaultDError>;

    fn broadcast_spend_tx(&self, txid: &str) -> Result<(), RevaultDError>;

    fn revault(&self, outpoint: &str) -> Result<(), RevaultDError>;

    fn emergency(&self) -> Result<(), RevaultDError>;

    fn stop(&self) -> Result<(), RevaultDError>;

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError>;

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u64,
        end: u64,
        limit: u64,
    ) -> Result<GetHistoryResponse, RevaultDError>;
}
