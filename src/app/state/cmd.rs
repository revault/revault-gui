use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::collections::HashMap;
use std::sync::Arc;

use crate::revaultd::{
    model::{
        RevocationTransactions, SpendTransaction, SpendTx, SpendTxStatus, UnvaultTransaction,
        Vault, VaultStatus, VaultTransactions,
    },
    RevaultD, RevaultDError,
};

/// retrieves a bitcoin address for deposit.
pub async fn get_deposit_address(
    revaultd: Arc<RevaultD>,
) -> Result<bitcoin::Address, RevaultDError> {
    revaultd.get_deposit_address().map(|res| res.address)
}

pub async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

pub async fn list_vaults(
    revaultd: Arc<RevaultD>,
    statuses: Option<&[VaultStatus]>,
    outpoints: Option<Vec<String>>,
) -> Result<Vec<Vault>, RevaultDError> {
    revaultd
        .list_vaults(statuses, outpoints.as_ref())
        .map(|res| res.vaults)
}

pub async fn get_onchain_txs(
    revaultd: Arc<RevaultD>,
    outpoint: String,
) -> Result<VaultTransactions, RevaultDError> {
    let list = revaultd.list_onchain_transactions(Some(vec![outpoint]))?;
    if list.onchain_transactions.is_empty() {
        return Err(RevaultDError::UnexpectedError(
            "vault has no onchain_transactions".to_string(),
        ));
    }

    Ok(list.onchain_transactions[0].to_owned())
}

pub async fn get_revocation_txs(
    revaultd: Arc<RevaultD>,
    outpoint: String,
) -> Result<RevocationTransactions, RevaultDError> {
    revaultd.get_revocation_txs(&outpoint)
}

pub async fn set_revocation_txs(
    revaultd: Arc<RevaultD>,
    outpoint: String,
    emergency_tx: Psbt,
    emergency_unvault_tx: Psbt,
    cancel_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_revocation_txs(&outpoint, &emergency_tx, &emergency_unvault_tx, &cancel_tx)
}

pub async fn get_unvault_tx(
    revaultd: Arc<RevaultD>,
    outpoint: String,
) -> Result<UnvaultTransaction, RevaultDError> {
    revaultd.get_unvault_tx(&outpoint)
}

pub async fn set_unvault_tx(
    revaultd: Arc<RevaultD>,
    outpoint: String,
    unvault_tx: Psbt,
) -> Result<(), RevaultDError> {
    revaultd.set_unvault_tx(&outpoint, &unvault_tx)
}

pub async fn get_spend_tx(
    revaultd: Arc<RevaultD>,
    inputs: Vec<String>,
    outputs: HashMap<String, u64>,
    feerate: u32,
) -> Result<SpendTransaction, RevaultDError> {
    revaultd.get_spend_tx(&inputs, &outputs, &feerate)
}

pub async fn update_spend_tx(revaultd: Arc<RevaultD>, psbt: Psbt) -> Result<(), RevaultDError> {
    revaultd.update_spend_tx(&psbt)
}

pub async fn list_spend_txs(
    revaultd: Arc<RevaultD>,
    statuses: Option<&[SpendTxStatus]>,
) -> Result<Vec<SpendTx>, RevaultDError> {
    revaultd.list_spend_txs(statuses).map(|res| res.spend_txs)
}

pub async fn delete_spend_tx(revaultd: Arc<RevaultD>, txid: String) -> Result<(), RevaultDError> {
    revaultd.delete_spend_tx(&txid)
}

pub async fn broadcast_spend_tx(
    revaultd: Arc<RevaultD>,
    txid: String,
) -> Result<(), RevaultDError> {
    revaultd.broadcast_spend_tx(&txid)
}

pub async fn revault(revaultd: Arc<RevaultD>, outpoint: String) -> Result<(), RevaultDError> {
    revaultd.revault(&outpoint)
}

pub async fn emergency(revaultd: Arc<RevaultD>) -> Result<(), RevaultDError> {
    revaultd.emergency()
}
