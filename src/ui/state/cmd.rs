use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::sync::Arc;

use crate::revaultd::{
    model::{RevocationTransactions, Vault, VaultTransactions},
    RevaultD, RevaultDError,
};

pub async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

pub async fn list_vaults(revaultd: Arc<RevaultD>) -> Result<Vec<Vault>, RevaultDError> {
    revaultd.list_vaults().map(|res| res.vaults)
}

pub async fn list_vaults_with_transactions(
    revaultd: Arc<RevaultD>,
) -> Result<Vec<(Vault, VaultTransactions)>, RevaultDError> {
    let vaults = revaultd.list_vaults().map(|res| res.vaults)?;
    let outpoints = vaults.iter().map(|vlt| vlt.outpoint()).collect();
    let txs = revaultd.list_onchain_transactions(Some(outpoints))?;

    let mut vec = Vec::new();
    for vlt in vaults {
        if let Some(i) = txs
            .onchain_transactions
            .iter()
            .position(|tx| tx.vault_outpoint == vlt.outpoint())
        {
            vec.push((vlt, txs.onchain_transactions[i].to_owned()));
        }
    }
    Ok(vec)
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
