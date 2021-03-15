use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::sync::Arc;

use crate::revaultd::{
    model::{RevocationTransactions, Vault, VaultStatus, VaultTransactions},
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
) -> Result<Vec<Vault>, RevaultDError> {
    revaultd.list_vaults(statuses).map(|res| res.vaults)
}

pub async fn get_onchain_txs(
    revaultd: Arc<RevaultD>,
    outpoint: String,
) -> Result<VaultTransactions, RevaultDError> {
    let list = revaultd.list_onchain_transactions(Some(vec![outpoint]))?;
    if list.onchain_transactions.len() == 0 {
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
