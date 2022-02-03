use std::collections::BTreeMap;
use std::path::PathBuf;

use bitcoin::{consensus::encode, util::psbt::PartiallySignedTransaction as Psbt, OutPoint, Txid};
use log::debug;

use super::{model::*, Daemon, RevaultDError};
use revaultd::{
    config::Config,
    revault_net::sodiumoxide,
    revault_tx::transactions::{
        CancelTransaction, EmergencyTransaction, RevaultTransaction, SpendTransaction,
        UnvaultEmergencyTransaction, UnvaultTransaction,
    },
    DaemonControl, DaemonHandle,
};

// RevaultD can start only if a config path is given.
pub async fn start_daemon(config_path: PathBuf) -> Result<EmbeddedDaemon, RevaultDError> {
    debug!("starting revaultd daemon");

    sodiumoxide::init().map_err(|_| RevaultDError::Start("sodiumoxide::init".to_string()))?;

    let mut config = Config::from_file(Some(config_path))
        .map_err(|e| RevaultDError::Start(format!("Error parsing config: {}", e)))?;
    config.daemon = Some(false);

    let mut daemon = EmbeddedDaemon::new();
    daemon.start(config)?;

    Ok(daemon)
}

pub struct EmbeddedDaemon {
    handle: Option<DaemonHandle>,
}

impl EmbeddedDaemon {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start(&mut self, config: Config) -> Result<(), RevaultDError> {
        let handle =
            DaemonHandle::start(config).map_err(|e| RevaultDError::Start(e.to_string()))?;
        self.handle = Some(handle);
        Ok(())
    }

    fn command(&self) -> Result<&DaemonControl, RevaultDError> {
        self.handle
            .as_ref()
            .map(|h| &h.control)
            .ok_or(RevaultDError::NoAnswer)
    }
}

impl std::fmt::Debug for EmbeddedDaemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DaemonHandle").finish()
    }
}

impl Daemon for EmbeddedDaemon {
    fn is_external(&self) -> bool {
        false
    }

    fn stop(&mut self) -> Result<(), RevaultDError> {
        if let Some(h) = self.handle.take() {
            h.shutdown();
        }
        Ok(())
    }

    fn get_deposit_address(&self) -> Result<bitcoin::Address, RevaultDError> {
        Ok(self.command()?.get_deposit_address())
    }

    fn get_info(&self) -> Result<GetInfoResult, RevaultDError> {
        Ok(self.command()?.get_info())
    }

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&[OutPoint]>,
    ) -> Result<Vec<Vault>, RevaultDError> {
        Ok(self.command()?.list_vaults(statuses, outpoints))
    }

    fn list_onchain_transactions(
        &self,
        outpoints: &[OutPoint],
    ) -> Result<Vec<VaultTransactions>, RevaultDError> {
        self.command()?
            .list_onchain_txs(outpoints)
            .map_err(|e| e.into())
    }

    fn get_revocation_txs(
        &self,
        outpoint: &OutPoint,
    ) -> Result<RevocationTransactions, RevaultDError> {
        self.command()?
            .get_revocation_txs(*outpoint)
            .map_err(|e| e.into())
    }

    fn set_revocation_txs(
        &self,
        outpoint: &OutPoint,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(), RevaultDError> {
        let cancel = CancelTransaction::from_raw_psbt(&encode::serialize(cancel_tx)).unwrap();
        let emergency =
            EmergencyTransaction::from_raw_psbt(&encode::serialize(emergency_tx)).unwrap();
        let unvault_emergency =
            UnvaultEmergencyTransaction::from_raw_psbt(&encode::serialize(emergency_unvault_tx))
                .unwrap();
        self.command()?
            .set_revocation_txs(*outpoint, cancel, emergency, unvault_emergency)
            .map_err(|e| e.into())
    }

    fn get_unvault_tx(&self, outpoint: &OutPoint) -> Result<Psbt, RevaultDError> {
        self.command()?
            .get_unvault_tx(*outpoint)
            .map(|tx| tx.into_psbt())
            .map_err(|e| e.into())
    }

    fn set_unvault_tx(&self, outpoint: &OutPoint, unvault_tx: &Psbt) -> Result<(), RevaultDError> {
        let unvault = UnvaultTransaction::from_raw_psbt(&encode::serialize(unvault_tx)).unwrap();
        self.command()?
            .set_unvault_tx(*outpoint, unvault)
            .map_err(|e| e.into())
    }

    fn get_spend_tx(
        &self,
        inputs: &[OutPoint],
        outputs: &BTreeMap<bitcoin::Address, u64>,
        feerate: u64,
    ) -> Result<Psbt, RevaultDError> {
        self.command()?
            .get_spend_tx(inputs, outputs, feerate)
            .map(|tx| tx.into_psbt())
            .map_err(|e| e.into())
    }

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend = SpendTransaction::from_raw_psbt(&encode::serialize(psbt)).unwrap();
        self.command()?.update_spend_tx(spend).map_err(|e| e.into())
    }

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<Vec<SpendTx>, RevaultDError> {
        self.command()?
            .list_spend_txs(statuses)
            .map_err(|e| e.into())
    }

    fn delete_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError> {
        self.command()?.del_spend_tx(txid).map_err(|e| e.into())
    }

    fn broadcast_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError> {
        self.command()?
            .set_spend_tx(txid, false)
            .map_err(|e| e.into())
    }

    fn revault(&self, outpoint: &OutPoint) -> Result<(), RevaultDError> {
        self.command()?.revault(outpoint).map_err(|e| e.into())
    }

    fn emergency(&self) -> Result<(), RevaultDError> {
        self.command()?.emergency().map_err(|e| e.into())
    }

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError> {
        Ok(self.command()?.get_servers_statuses())
    }

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u32,
        end: u32,
        limit: u64,
    ) -> Result<Vec<HistoryEvent>, RevaultDError> {
        self.command()?
            .get_history(start, end, limit, kind)
            .map_err(|e| e.into())
    }
}
