use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, OutPoint, Txid};
use std::sync::Arc;

use crate::daemon::{
    model::{
        RevocationTransactions, ServersStatuses, SpendTx, SpendTxStatus, Vault, VaultStatus,
        VaultTransactions,
    },
    Daemon, RevaultDError,
};

/// retrieves a bitcoin address for deposit.
pub async fn get_deposit_address(
    revaultd: Arc<dyn Daemon + Send + Sync>,
) -> Result<bitcoin::Address, RevaultDError> {
    revaultd.get_deposit_address()
}

pub async fn list_vaults(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    statuses: Option<&[VaultStatus]>,
    outpoints: Option<Vec<OutPoint>>,
) -> Result<Vec<Vault>, RevaultDError> {
    revaultd.list_vaults(statuses, outpoints.as_deref())
}

pub async fn get_onchain_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<VaultTransactions, RevaultDError> {
    let list = revaultd.list_onchain_transactions(&[outpoint])?;
    if list.is_empty() {
        return Err(RevaultDError::Unexpected(
            "vault has no onchain_transactions".to_string(),
        ));
    }

    Ok(list[0].to_owned())
}

pub async fn get_revocation_txs(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<RevocationTransactions, RevaultDError> {
    revaultd.get_revocation_txs(&outpoint)
}

pub async fn get_unvault_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
) -> Result<Psbt, RevaultDError> {
    revaultd.get_unvault_tx(&outpoint)
}

pub async fn set_unvault_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    outpoint: OutPoint,
    unvault_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_unvault_tx(&outpoint, &unvault_tx)
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
    revaultd.list_spend_txs(statuses)
}

pub async fn delete_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    txid: Txid,
) -> Result<(), RevaultDError> {
    revaultd.delete_spend_tx(&txid)
}

pub async fn broadcast_spend_tx(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    txid: Txid,
    with_priority: bool,
) -> Result<(), RevaultDError> {
    revaultd.broadcast_spend_tx(&txid, with_priority)
}

pub async fn cpfp(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    txids: Vec<Txid>,
    fee_rate: f64,
) -> Result<(), RevaultDError> {
    revaultd.cpfp(&txids, fee_rate)
}

pub async fn emergency(revaultd: Arc<dyn Daemon + Send + Sync>) -> Result<(), RevaultDError> {
    revaultd.emergency()
}

pub async fn get_server_status(
    revaultd: Arc<dyn Daemon + Send + Sync>,
) -> Result<ServersStatuses, RevaultDError> {
    revaultd.get_server_status()
}
