use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, OutPoint};
use std::collections::HashMap;
use std::sync::Arc;

use crate::daemon::{
    model::{
        RevocationTransactions, ServersStatuses, SpendTransaction, SpendTx, SpendTxStatus,
        UnvaultTransaction, Vault, VaultStatus, VaultTransactions,
    },
    Daemon, RevaultDError,
};

/// retrieves a bitcoin address for deposit.
pub async fn get_deposit_address(
    revaultd: Arc<dyn Daemon + Send + Sync>,
) -> Result<bitcoin::Address, RevaultDError> {
    revaultd.get_deposit_address().map(|res| res.address)
}

pub async fn list_vaults(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    statuses: Option<&[VaultStatus]>,
    outpoints: Option<Vec<OutPoint>>,
) -> Result<Vec<Vault>, RevaultDError> {
    revaultd
        .list_vaults(statuses, outpoints.as_ref())
        .map(|res| res.vaults)
}

pub async fn get_onchain_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<VaultTransactions, RevaultDError> {
    let list = revaultd.list_onchain_transactions(Some(vec![outpoint]))?;
    if list.onchain_transactions.is_empty() {
        return Err(RevaultDError::Unexpected(
            "vault has no onchain_transactions".to_string(),
        ));
    }

    Ok(list.onchain_transactions[0].to_owned())
}

pub async fn get_revocation_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<RevocationTransactions, RevaultDError> {
    revaultd.get_revocation_txs(&outpoint)
}

pub async fn set_revocation_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
    emergency_tx: Psbt,
    emergency_unvault_tx: Psbt,
    cancel_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_revocation_txs(&outpoint, &emergency_tx, &emergency_unvault_tx, &cancel_tx)
}

pub async fn get_unvault_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<UnvaultTransaction, RevaultDError> {
    revaultd.get_unvault_tx(&outpoint)
}

pub async fn set_unvault_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
    unvault_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_unvault_tx(&outpoint, &unvault_tx)
}

pub async fn get_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    inputs: Vec<OutPoint>,
    outputs: HashMap<String, u64>,
    feerate: u32,
) -> Result<SpendTransaction, RevaultDError> {
    revaultd.get_spend_tx(&inputs, &outputs, &feerate)
}

pub async fn update_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    psbt: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.update_spend_tx(&psbt)
}

pub async fn list_spend_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    statuses: Option<&[SpendTxStatus]>,
) -> Result<Vec<SpendTx>, RevaultDError> {
    revaultd.list_spend_txs(statuses).map(|res| res.spend_txs)
}

pub async fn delete_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    txid: String,
) -> Result<(), RevaultDError> {
    revaultd.delete_spend_tx(&txid)
}

pub async fn broadcast_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    txid: String,
) -> Result<(), RevaultDError> {
    revaultd.broadcast_spend_tx(&txid)
}

pub async fn revault(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<(), RevaultDError> {
    revaultd.revault(&outpoint)
}

pub async fn emergency(revaultd: Arc<dyn Daemon + Send + Sync>) -> Result<(), RevaultDError> {
    revaultd.emergency()
}

pub async fn get_server_status(
    revaultd: Arc<dyn Daemon + Send + Sync>,
) -> Result<ServersStatuses, RevaultDError> {
    revaultd.get_server_status()
}

pub async fn stop(revaultd: Arc<dyn Daemon + Send + Sync>) -> Result<(), RevaultDError> {
    revaultd.stop()
}
