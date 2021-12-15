use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::daemon::{
    client::{Client, RevaultD, RevaultDError},
    model::{
        HistoryEvent, HistoryEventKind, RevocationTransactions, ServersStatuses, SpendTransaction,
        SpendTx, SpendTxStatus, UnvaultTransaction, Vault, VaultStatus, VaultTransactions,
    },
};

/// retrieves a bitcoin address for deposit.
pub async fn get_deposit_address<C: Client>(
    revaultd: Arc<RevaultD<C>>,
) -> Result<bitcoin::Address, RevaultDError> {
    revaultd.get_deposit_address().map(|res| res.address)
}

pub async fn get_blockheight<C: Client>(revaultd: Arc<RevaultD<C>>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

pub async fn list_vaults<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    statuses: Option<&[VaultStatus]>,
    outpoints: Option<Vec<String>>,
) -> Result<Vec<Vault>, RevaultDError> {
    revaultd
        .list_vaults(statuses, outpoints.as_ref())
        .map(|res| res.vaults)
}

pub async fn get_onchain_txs<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
) -> Result<VaultTransactions, RevaultDError> {
    let list = revaultd.list_onchain_transactions(Some(vec![outpoint]))?;
    if list.onchain_transactions.is_empty() {
        return Err(RevaultDError::Unexpected(
            "vault has no onchain_transactions".to_string(),
        ));
    }

    Ok(list.onchain_transactions[0].to_owned())
}

pub async fn get_revocation_txs<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
) -> Result<RevocationTransactions, RevaultDError> {
    revaultd.get_revocation_txs(&outpoint)
}

pub async fn set_revocation_txs<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
    emergency_tx: Psbt,
    emergency_unvault_tx: Psbt,
    cancel_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_revocation_txs(&outpoint, &emergency_tx, &emergency_unvault_tx, &cancel_tx)
}

pub async fn get_unvault_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
) -> Result<UnvaultTransaction, RevaultDError> {
    revaultd.get_unvault_tx(&outpoint)
}

pub async fn set_unvault_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
    unvault_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_unvault_tx(&outpoint, &unvault_tx)
}

pub async fn get_spend_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    inputs: Vec<String>,
    outputs: HashMap<String, u64>,
    feerate: u32,
) -> Result<SpendTransaction, RevaultDError> {
    revaultd.get_spend_tx(&inputs, &outputs, &feerate)
}

pub async fn update_spend_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    psbt: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.update_spend_tx(&psbt)
}

pub async fn list_spend_txs<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    statuses: Option<&[SpendTxStatus]>,
) -> Result<Vec<SpendTx>, RevaultDError> {
    revaultd.list_spend_txs(statuses).map(|res| res.spend_txs)
}

pub async fn delete_spend_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    txid: String,
) -> Result<(), RevaultDError> {
    revaultd.delete_spend_tx(&txid)
}

pub async fn broadcast_spend_tx<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    txid: String,
) -> Result<(), RevaultDError> {
    revaultd.broadcast_spend_tx(&txid)
}

pub async fn revault<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    outpoint: String,
) -> Result<(), RevaultDError> {
    revaultd.revault(&outpoint)
}

pub async fn emergency<C: Client>(revaultd: Arc<RevaultD<C>>) -> Result<(), RevaultDError> {
    revaultd.emergency()
}

pub async fn get_server_status<C: Client>(
    revaultd: Arc<RevaultD<C>>,
) -> Result<ServersStatuses, RevaultDError> {
    revaultd.get_server_status()
}

pub async fn stop<C: Client>(revaultd: Arc<RevaultD<C>>) -> Result<(), RevaultDError> {
    revaultd.stop()
}

pub async fn get_history<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    kind: Vec<HistoryEventKind>,
    start: u64,
    end: u64,
    limit: u64,
) -> Result<Vec<HistoryEvent>, RevaultDError> {
    revaultd
        .get_history(&kind, start, end, limit)
        .map(|res| res.events)
}
