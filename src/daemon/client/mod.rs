use serde_json::json;
use std::collections::BTreeMap;
use std::fmt::Debug;

use bitcoin::{base64, consensus, util::psbt::PartiallySignedTransaction as Psbt, OutPoint, Txid};
use log::{error, info};
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
    pub fn new(client: C) -> RevaultD<C> {
        RevaultD { client }
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
    fn is_external(&self) -> bool {
        true
    }

    /// get a new deposit address.
    fn get_deposit_address(&self) -> Result<bitcoin::Address, RevaultDError> {
        let deposit_address: DepositAddress =
            self.call("getdepositaddress", Option::<Request>::None)?;
        Ok(deposit_address.address)
    }

    fn get_info(&self) -> Result<GetInfoResult, RevaultDError> {
        self.call("getinfo", Option::<Request>::None)
    }

    fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&[OutPoint]>,
    ) -> Result<Vec<Vault>, RevaultDError> {
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
        let response: ListVaultsResponse = self.call("listvaults", Some(args))?;
        Ok(response.vaults)
    }

    fn list_onchain_transactions(
        &self,
        outpoints: &[OutPoint],
    ) -> Result<Vec<VaultTransactions>, RevaultDError> {
        let outpoints: Vec<String> = outpoints.iter().map(|o| o.to_string()).collect();
        let response: ListOnchainTransactionsResponse = self.call(
            "listonchaintransactions",
            Some(vec![ListTransactionsRequest(outpoints)]),
        )?;
        Ok(response.onchain_transactions)
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

    fn get_unvault_tx(&self, outpoint: &OutPoint) -> Result<Psbt, RevaultDError> {
        let resp: UnvaultTransaction =
            self.call("getunvaulttx", Some(vec![outpoint.to_string()]))?;
        Ok(resp.unvault_tx)
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
        outputs: &BTreeMap<bitcoin::Address, u64>,
        feerate: u64,
    ) -> Result<Psbt, RevaultDError> {
        let resp: SpendTransaction = self.call(
            "getspendtx",
            Some(vec![json!(inputs), json!(outputs), json!(feerate)]),
        )?;
        Ok(resp.spend_tx)
    }

    fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend_tx = base64::encode(&consensus::serialize(psbt));
        let _res: serde_json::value::Value = self.call("updatespendtx", Some(vec![spend_tx]))?;
        Ok(())
    }

    fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<Vec<SpendTx>, RevaultDError> {
        let resp: ListSpendTransactionsResponse =
            self.call("listspendtxs", Some(vec![statuses]))?;
        Ok(resp.spend_txs)
    }

    fn delete_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("delspendtx", Some(vec![txid]))?;
        Ok(())
    }

    fn broadcast_spend_tx(&self, txid: &Txid) -> Result<(), RevaultDError> {
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

    fn stop(&mut self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("stop", Option::<Request>::None)?;
        Ok(())
    }

    fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError> {
        self.call("getserverstatus", Option::<Request>::None)
    }

    fn get_history(
        &self,
        kind: &[HistoryEventKind],
        start: u32,
        end: u32,
        limit: u64,
    ) -> Result<Vec<HistoryEvent>, RevaultDError> {
        let resp: GetHistoryResponse = self.call(
            "gethistory",
            Some(vec![json!(kind), json!(start), json!(end), json!(limit)]),
        )?;
        Ok(resp.events)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {}

/// listtransactions request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTransactionsRequest(Vec<String>);

/// listvaults response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListVaultsResponse {
    pub vaults: Vec<Vault>,
}

/// gethistory response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetHistoryResponse {
    pub events: Vec<HistoryEvent>,
}

/// listtransactions response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListOnchainTransactionsResponse {
    pub onchain_transactions: Vec<VaultTransactions>,
}

/// list_spend_txs
#[derive(Debug, Clone, Deserialize)]
pub struct ListSpendTransactionsResponse {
    pub spend_txs: Vec<SpendTx>,
}

/// getdepositaddress response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAddress {
    pub address: bitcoin::Address,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnvaultTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub unvault_tx: Psbt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpendTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub spend_tx: Psbt,
}

mod bitcoin_psbt {
    use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PartiallySignedTransaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes: Vec<u8> = base64::decode(&s).map_err(serde::de::Error::custom)?;
        encode::deserialize(&bytes).map_err(serde::de::Error::custom)
    }

    pub fn serialize<'se, S>(
        psbt: &PartiallySignedTransaction,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(&encode::serialize(&psbt)))
    }
}
