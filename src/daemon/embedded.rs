use std::collections::BTreeMap;
use std::sync::Mutex;

use bitcoin::{consensus::encode, util::psbt::PartiallySignedTransaction as Psbt, OutPoint, Txid};

use super::{model::*, Daemon, RevaultDError};
use revaultd::{
    commands::CommandError,
    config::Config,
    revault_tx::transactions::{
        CancelTransaction, EmergencyTransaction, RevaultTransaction, SpendTransaction,
        UnvaultEmergencyTransaction, UnvaultTransaction,
    },
    DaemonHandle,
};

impl From<CommandError> for RevaultDError {
    fn from(error: CommandError) -> Self {
        RevaultDError::Rpc(error.code() as i32, error.to_string())
    }
}

pub struct EmbeddedDaemon {
    handle: Option<Mutex<DaemonHandle>>,
}

impl EmbeddedDaemon {
    pub fn new() -> Self {
        Self { handle: None }
    }

    pub fn start(&mut self, config: Config) -> Result<(), RevaultDError> {
        let handle =
            DaemonHandle::start(config).map_err(|e| RevaultDError::Start(e.to_string()))?;
        self.handle = Some(Mutex::new(handle));
        Ok(())
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

    fn load_config(&mut self, cfg: Config) -> Result<(), RevaultDError> {
        if self.handle.is_none() {
            return Ok(());
        }

        let next = DaemonHandle::start(cfg).map_err(|e| RevaultDError::Start(e.to_string()))?;
        self.handle.take().unwrap().into_inner().unwrap().shutdown();
        self.handle = Some(Mutex::new(next));
        Ok(())
    }

    fn stop(&mut self) -> Result<(), RevaultDError> {
        if let Some(h) = self.handle.take() {
            let handle = h.into_inner().unwrap();
            handle.shutdown();
        }
        Ok(())
    }

    fn get_deposit_address(&self) -> Result<bitcoin::Address, RevaultDError> {
        Ok(self
            .handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_deposit_address())
    }

    fn get_info(&self) -> Result<GetInfoResult, RevaultDError> {
        Ok(self
            .handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_info())
    }

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&[OutPoint]>,
    ) -> Result<Vec<Vault>, RevaultDError> {
        Ok(self
            .handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .list_vaults(statuses, outpoints))
    }

    fn list_onchain_transactions(
        &self,
        outpoints: &[OutPoint],
    ) -> Result<Vec<VaultTransactions>, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .list_onchain_txs(outpoints)
            .map_err(|e| e.into())
    }

    fn list_presigned_transactions(
        &self,
        outpoints: &[OutPoint],
    ) -> Result<Vec<VaultPresignedTransactions>, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .list_presigned_txs(outpoints)
            .map_err(|e| e.into())
    }

    fn get_revocation_txs(
        &self,
        outpoint: &OutPoint,
    ) -> Result<RevocationTransactions, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_revocation_txs(*outpoint)
            .map_err(|e| e.into())
    }

    fn set_revocation_txs(
        &self,
        outpoint: &OutPoint,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_txs: &[Psbt; 5],
    ) -> Result<(), RevaultDError> {
        let cancel_txs: [CancelTransaction; 5] = [
            CancelTransaction::from_raw_psbt(&encode::serialize(&cancel_txs[0])).unwrap(),
            CancelTransaction::from_raw_psbt(&encode::serialize(&cancel_txs[1])).unwrap(),
            CancelTransaction::from_raw_psbt(&encode::serialize(&cancel_txs[2])).unwrap(),
            CancelTransaction::from_raw_psbt(&encode::serialize(&cancel_txs[3])).unwrap(),
            CancelTransaction::from_raw_psbt(&encode::serialize(&cancel_txs[4])).unwrap(),
        ];
        let emergency_tx =
            EmergencyTransaction::from_raw_psbt(&encode::serialize(emergency_tx)).unwrap();
        let emergency_unvault_tx =
            UnvaultEmergencyTransaction::from_raw_psbt(&encode::serialize(emergency_unvault_tx))
                .unwrap();
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .set_revocation_txs(
                *outpoint,
                RevocationTransactions {
                    cancel_txs,
                    emergency_tx,
                    emergency_unvault_tx,
                },
            )
            .map_err(|e| e.into())
    }

    fn get_unvault_tx(&self, outpoint: &OutPoint) -> Result<Psbt, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_unvault_tx(*outpoint)
            .map(|tx| tx.into_psbt())
            .map_err(|e| e.into())
    }

    fn set_unvault_tx(&self, outpoint: &OutPoint, unvault_tx: &Psbt) -> Result<(), RevaultDError> {
        let unvault = UnvaultTransaction::from_raw_psbt(&encode::serialize(unvault_tx)).unwrap();
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .set_unvault_tx(*outpoint, unvault)
            .map_err(|e| e.into())
    }

    fn get_spend_tx(
        &self,
        inputs: &[OutPoint],
        outputs: &BTreeMap<bitcoin::Address, u64>,
        feerate: u64,
    ) -> Result<SpendTx, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_spend_tx(inputs, outputs, feerate)
            .map_err(|e| e.into())
    }

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend = SpendTransaction::from_raw_psbt(&encode::serialize(psbt)).unwrap();
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .update_spend_tx(spend)
            .map_err(|e| e.into())
    }

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<Vec<SpendTx>, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .list_spend_txs(statuses)
            .map_err(|e| e.into())
    }

    fn delete_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .del_spend_tx(txid)
            .map_err(|e| e.into())
    }

    fn broadcast_spend_tx(&self, txid: &Txid, priority: bool) -> Result<(), RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .set_spend_tx(txid, priority)
            .map_err(|e| e.into())
    }

    fn revault(&self, outpoint: &OutPoint) -> Result<(), RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .revault(*outpoint)
            .map_err(|e| e.into())
    }

    fn emergency(&self) -> Result<(), RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .emergency()
            .map_err(|e| e.into())
    }

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError> {
        Ok(self
            .handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_servers_statuses())
    }

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u32,
        end: u32,
        limit: u64,
    ) -> Result<Vec<HistoryEvent>, RevaultDError> {
        self.handle
            .as_ref()
            .ok_or(RevaultDError::NoAnswer)?
            .lock()
            .unwrap()
            .control
            .get_history(start, end, limit, kind)
            .map_err(|e| e.into())
    }
    fn cpfp(&self, txids: &[Txid], feerate: f64) -> Result<(), RevaultDError> {
        Ok(())
        // Here we implement the call to the backend.
        // self.handle
        //     .as_ref()
        //     .ok_or(RevaultDError::NoAnswer)?
        //     .lock()
        //     .unwrap()
        //     .control
        //     .cpfp(txids, feerate)
        //     .map_err(|e| e.into())
    }
}
