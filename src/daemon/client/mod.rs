use serde_json::json;
use std::collections::HashMap;
use std::fmt::Debug;

use bitcoin::{base64, consensus, util::psbt::PartiallySignedTransaction as Psbt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, span, Level};

pub mod jsonrpc;

use super::config::Config;
use super::model::*;

#[derive(Debug, Clone)]
pub enum RevaultDError {
    UnexpectedError(String),
    RPCError(String),
    IOError(std::io::ErrorKind),
    NoAnswerError,
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RPCError(e) => write!(f, "Revaultd error rpc call: {}", e),
            Self::UnexpectedError(e) => write!(f, "Revaultd unexpected error: {}", e),
            Self::NoAnswerError => write!(f, "Revaultd returned no answer"),
            Self::IOError(kind) => write!(f, "Revaultd io error: {:?}", kind),
        }
    }
}

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
    pub config: Config,
}

impl<C: Client> RevaultD<C> {
    pub fn new(config: &Config, client: C) -> Result<RevaultD<C>, RevaultDError> {
        let span = span!(Level::INFO, "revaultd");
        let _enter = span.enter();

        let revaultd = RevaultD {
            client,
            config: config.to_owned(),
        };

        debug!("Connecting to revaultd");

        revaultd.get_info()?;

        info!("Connected to revaultd");

        Ok(revaultd)
    }

    pub fn network(&self) -> bitcoin::Network {
        self.config.bitcoind_config.network
    }

    /// Generic call function for RPC calls.
    fn call<T: Serialize + Debug, U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        input: Option<T>,
    ) -> Result<U, RevaultDError> {
        let span = span!(Level::INFO, "request");
        let _guard = span.enter();
        info!(method);
        self.client.request(method, input).map_err(|e| {
            error!("method {} failed: {:?}", method, e);
            e.into()
        })
    }

    /// get a new deposit address.
    pub fn get_deposit_address(&self) -> Result<DepositAddress, RevaultDError> {
        self.call("getdepositaddress", Option::<Request>::None)
    }

    pub fn get_info(&self) -> Result<GetInfoResponse, RevaultDError> {
        self.call("getinfo", Option::<Request>::None)
    }

    pub fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&Vec<String>>,
    ) -> Result<ListVaultsResponse, RevaultDError> {
        let mut args = vec![json!(statuses.unwrap_or(&[]))];
        if let Some(outpoints) = outpoints {
            args.push(json!(outpoints));
        }
        self.call("listvaults", Some(args))
    }

    pub fn list_onchain_transactions(
        &self,
        outpoints: Option<Vec<String>>,
    ) -> Result<ListOnchainTransactionsResponse, RevaultDError> {
        match outpoints {
            Some(list) => self.call(
                "listonchaintransactions",
                Some(vec![ListTransactionsRequest(list)]),
            ),
            None => self.call("listonchaintransactions", Option::<Request>::None),
        }
    }

    pub fn get_revocation_txs(
        &self,
        outpoint: &str,
    ) -> Result<RevocationTransactions, RevaultDError> {
        self.call("getrevocationtxs", Some(vec![outpoint]))
    }

    pub fn set_revocation_txs(
        &self,
        outpoint: &str,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(), RevaultDError> {
        let emergency = base64::encode(&consensus::serialize(emergency_tx));
        let emergency_unvault = base64::encode(&consensus::serialize(emergency_unvault_tx));
        let cancel = base64::encode(&consensus::serialize(cancel_tx));
        let _res: serde_json::value::Value = self.call(
            "revocationtxs",
            Some(vec![outpoint, &cancel, &emergency, &emergency_unvault]),
        )?;
        Ok(())
    }

    pub fn get_unvault_tx(&self, outpoint: &str) -> Result<UnvaultTransaction, RevaultDError> {
        self.call("getunvaulttx", Some(vec![outpoint]))
    }

    pub fn set_unvault_tx(&self, outpoint: &str, unvault_tx: &Psbt) -> Result<(), RevaultDError> {
        let unvault_tx = base64::encode(&consensus::serialize(unvault_tx));
        let _res: serde_json::value::Value =
            self.call("unvaulttx", Some(vec![outpoint, &unvault_tx]))?;
        Ok(())
    }

    pub fn get_spend_tx(
        &self,
        inputs: &[String],
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

    pub fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend_tx = base64::encode(&consensus::serialize(psbt));
        let _res: serde_json::value::Value = self.call("updatespendtx", Some(vec![spend_tx]))?;
        Ok(())
    }

    pub fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<ListSpendTransactionsResponse, RevaultDError> {
        self.call("listspendtxs", Some(vec![statuses]))
    }

    pub fn delete_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("delspendtx", Some(vec![txid]))?;
        Ok(())
    }

    pub fn broadcast_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("setspendtx", Some(vec![txid]))?;
        Ok(())
    }

    pub fn revault(&self, outpoint: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("revault", Some(vec![outpoint]))?;
        Ok(())
    }

    pub fn emergency(&self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("emergency", Option::<Request>::None)?;
        Ok(())
    }

    pub fn stop(&self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("stop", Option::<Request>::None)?;
        Ok(())
    }

    pub fn get_server_status(&self) -> Result<ServersStatuses, RevaultDError> {
        self.call("getserverstatus", Option::<Request>::None)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Request {}

/// getinfo

/// getinfo response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInfoResponse {
    pub blockheight: u64,
    pub network: String,
    pub sync: f64,
    pub version: String,
    pub managers_threshold: usize,
}

/// list_vaults

/// listvaults response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListVaultsResponse {
    pub vaults: Vec<Vault>,
}

/// list_transactions

/// listtransactions request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListTransactionsRequest(Vec<String>);

/// listtransactions response
#[derive(Debug, Clone, Deserialize)]
pub struct ListOnchainTransactionsResponse {
    pub onchain_transactions: Vec<VaultTransactions>,
}

/// list_spend_txs
#[derive(Debug, Clone, Deserialize)]
pub struct ListSpendTransactionsResponse {
    pub spend_txs: Vec<SpendTx>,
}
