use serde_json::json;
use std::collections::HashMap;
use std::fmt::Debug;

use bitcoin::{base64, consensus, util::psbt::PartiallySignedTransaction as Psbt, OutPoint};
use log::{debug, error, info};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod error;
pub mod jsonrpc;

use super::{model::*, Daemon, RevaultDError};

pub trait Client {
    type Error: Into<RevaultDError> + Debug;
    fn request<S: Serialize + Debug, D: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Option<S>,
    ) -> Result<D, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RevaultD<C: Client> {
    client: C,
}

impl<C: Client> RevaultD<C> {
    pub fn new(client: C) -> Result<RevaultD<C>, RevaultDError> {
        let revaultd = RevaultD { client };

        debug!("Connecting to revaultd");

        revaultd.get_info()?;

        info!("Connected to revaultd");

        Ok(revaultd)
    }

    /// Generic call function for RPC calls.
    fn call<T: Serialize + Debug, U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        input: Option<T>,
    ) -> Result<U, RevaultDError> {
        info!("{}", method);
        self.client.request(method, input).map_err(|e| {
            error!("method {} failed: {:?}", method, e);
            e.into()
        })
    }
}

impl<C: Client> Daemon for RevaultD<C> {
    /// get a new deposit address.
    fn get_deposit_address(&self) -> Result<DepositAddress, RevaultDError> {
        self.call("getdepositaddress", Option::<Request>::None)
    }

    fn get_info(&self) -> Result<GetInfoResponse, RevaultDError> {
        self.call("getinfo", Option::<Request>::None)
    }

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&Vec<OutPoint>>,
    ) -> Result<ListVaultsResponse, RevaultDError> {
        let statuses: Vec<String> = statuses
            .unwrap_or(&[])
            .iter()
            .map(|s| s.to_string())
            .collect();
        let mut args = vec![json!(statuses)];
        if let Some(outpoints) = outpoints {
            let outpoints: Vec<String> = outpoints.iter().map(|o| o.to_string()).collect();
            args.push(json!(outpoints));
        }
        self.call("listvaults", Some(args))
    }

    fn list_onchain_transactions(
        &self,
        outpoints: Option<Vec<OutPoint>>,
    ) -> Result<ListOnchainTransactionsResponse, RevaultDError> {
        match outpoints {
            Some(list) => {
                let outpoints: Vec<String> = list.iter().map(|o| o.to_string()).collect();
                self.call(
                    "listonchaintransactions",
                    Some(vec![ListTransactionsRequest(outpoints)]),
                )
            }
            None => self.call("listonchaintransactions", Option::<Request>::None),
        }
    }

    fn get_revocation_txs(
        &self,
        outpoint: &OutPoint,
    ) -> Result<RevocationTransactions, RevaultDError> {
        self.call("getrevocationtxs", Some(vec![outpoint.to_string()]))
    }

    fn set_revocation_txs(
        &self,
        outpoint: &OutPoint,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(), RevaultDError> {
        let emergency = base64::encode(&consensus::serialize(emergency_tx));
        let emergency_unvault = base64::encode(&consensus::serialize(emergency_unvault_tx));
        let cancel = base64::encode(&consensus::serialize(cancel_tx));
        let _res: serde_json::value::Value = self.call(
            "revocationtxs",
            Some(vec![
                outpoint.to_string(),
                cancel,
                emergency,
                emergency_unvault,
            ]),
        )?;
        Ok(())
    }

    fn get_unvault_tx(&self, outpoint: &OutPoint) -> Result<UnvaultTransaction, RevaultDError> {
        self.call("getunvaulttx", Some(vec![outpoint.to_string()]))
    }

    fn set_unvault_tx(&self, outpoint: &OutPoint, unvault_tx: &Psbt) -> Result<(), RevaultDError> {
        let unvault_tx = base64::encode(&consensus::serialize(unvault_tx));
        let _res: serde_json::value::Value =
            self.call("unvaulttx", Some(vec![outpoint.to_string(), unvault_tx]))?;
        Ok(())
    }

    fn get_spend_tx(
        &self,
        inputs: &[OutPoint],
        outputs: &HashMap<String, u64>,
        feerate: &u32,
    ) -> Result<SpendTransaction, RevaultDError> {
        self.call(
            "getspendtx",
            Some(vec![json!(inputs), json!(outputs), json!(feerate)]),
        )
        .map(|mut res: SpendTransaction| {
            res.feerate = *feerate;
            res
        })
    }

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend_tx = base64::encode(&consensus::serialize(psbt));
        let _res: serde_json::value::Value = self.call("updatespendtx", Some(vec![spend_tx]))?;
        Ok(())
    }

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<ListSpendTransactionsResponse, RevaultDError> {
        self.call("listspendtxs", Some(vec![statuses]))
    }

    fn delete_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("delspendtx", Some(vec![txid]))?;
        Ok(())
    }

    fn broadcast_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("setspendtx", Some(vec![txid]))?;
        Ok(())
    }

    fn revault(&self, outpoint: &OutPoint) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value =
            self.call("revault", Some(vec![outpoint.to_string()]))?;
        Ok(())
    }

    fn emergency(&self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("emergency", Option::<Request>::None)?;
        Ok(())
    }

    fn stop(&self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("stop", Option::<Request>::None)?;
        Ok(())
    }

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError> {
        self.call("getserverstatus", Option::<Request>::None)
    }

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u64,
        end: u64,
        limit: u64,
    ) -> Result<GetHistoryResponse, RevaultDError> {
        self.call(
            "gethistory",
            Some(vec![json!(kind), json!(start), json!(end), json!(limit)]),
        )
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {}

/// listtransactions request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTransactionsRequest(Vec<String>);
