use serde_json::json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::process::Command;
use std::time;

use bitcoin::{base64, consensus, util::psbt::PartiallySignedTransaction as Psbt};
use jsonrpc::{simple_uds::UdsTransport, Client};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, span, Level};

pub mod config;
pub mod model;

use config::Config;
use model::{
    DepositAddress, RevocationTransactions, SpendTransaction, SpendTx, SpendTxStatus,
    UnvaultTransaction, Vault, VaultStatus, VaultTransactions,
};

#[derive(Debug, Clone)]
pub enum RevaultDError {
    UnexpectedError(String),
    StartError(String),
    RPCError(String),
    IOError(std::io::ErrorKind),
    NoAnswerError,
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::StartError(e) => write!(f, "Revaultd error while starting: {}", e),
            Self::RPCError(e) => write!(f, "Revaultd error rpc call: {}", e),
            Self::UnexpectedError(e) => write!(f, "Revaultd unexpected error: {}", e),
            Self::NoAnswerError => write!(f, "Revaultd returned no answer"),
            Self::IOError(kind) => write!(f, "Revaultd io error: {:?}", kind),
        }
    }
}

#[derive(Debug)]
pub struct RevaultD {
    client: Client,
    pub config: Config,
}

impl RevaultD {
    pub fn new(config: &Config) -> Result<RevaultD, RevaultDError> {
        let span = span!(Level::INFO, "revaultd");
        let _enter = span.enter();

        let sockpath = config.socket_path().map_err(|e| {
            RevaultDError::UnexpectedError(format!(
                "Failed to find revaultd socket path: {}",
                e.to_string()
            ))
        })?;

        let transport = UdsTransport {
            sockpath,
            timeout: Some(time::Duration::from_secs(60)),
        };
        let client = Client::with_transport(transport);
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
    fn call<U: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: &[Box<serde_json::value::RawValue>],
    ) -> Result<U, RevaultDError> {
        let span = span!(Level::INFO, "request");
        let _guard = span.enter();
        info!(method);
        info!("{:?}", self.client);
        let req = self.client.build_request(method, params);
        self.client
            .send_request(req)
            .and_then(|res| res.result())
            .map_err(|e| {
                error!("method {} failed: {}", method, e);
                match e {
                    _ => RevaultDError::RPCError(format!("method {} failed: {}", method, e)),
                }
            })
    }

    /// get a new deposit address.
    pub fn get_deposit_address(&self) -> Result<DepositAddress, RevaultDError> {
        self.call("getdepositaddress", &[])
    }

    pub fn get_info(&self) -> Result<GetInfoResponse, RevaultDError> {
        self.call("getinfo", &[])
    }

    pub fn list_vaults(
        &self,
        statuses: Option<&[VaultStatus]>,
        outpoints: Option<&Vec<String>>,
    ) -> Result<ListVaultsResponse, RevaultDError> {
        self.call(
            "listvaults",
            &[jsonrpc::arg(statuses), jsonrpc::arg(outpoints)],
        )
    }

    pub fn list_onchain_transactions(
        &self,
        outpoints: Option<Vec<String>>,
    ) -> Result<ListOnchainTransactionsResponse, RevaultDError> {
        self.call("listonchaintransactions", &[jsonrpc::arg(outpoints)])
    }

    pub fn get_revocation_txs(
        &self,
        outpoint: &str,
    ) -> Result<RevocationTransactions, RevaultDError> {
        self.call("getrevocationtxs", &[jsonrpc::arg(outpoint)])
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
            &[
                jsonrpc::arg(outpoint),
                jsonrpc::arg(cancel),
                jsonrpc::arg(emergency),
                jsonrpc::arg(emergency_unvault),
            ],
        )?;
        Ok(())
    }

    pub fn get_unvault_tx(&self, outpoint: &str) -> Result<UnvaultTransaction, RevaultDError> {
        self.call("getunvaulttx", &[jsonrpc::arg(outpoint)])
    }

    pub fn set_unvault_tx(&self, outpoint: &str, unvault_tx: &Psbt) -> Result<(), RevaultDError> {
        let unvault_tx = base64::encode(&consensus::serialize(unvault_tx));
        let _res: serde_json::value::Value = self.call(
            "unvaulttx",
            &[jsonrpc::arg(outpoint), jsonrpc::arg(unvault_tx)],
        )?;
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
            &[
                jsonrpc::arg(inputs),
                jsonrpc::arg(outputs),
                jsonrpc::arg(feerate),
            ],
        )
        .map(|mut res: SpendTransaction| {
            res.feerate = *feerate;
            res
        })
    }

    pub fn update_spend_tx(&self, psbt: &Psbt) -> Result<(), RevaultDError> {
        let spend_tx = base64::encode(&consensus::serialize(psbt));
        let _res: serde_json::value::Value =
            self.call("updatespendtx", &[jsonrpc::arg(spend_tx)])?;
        Ok(())
    }

    pub fn list_spend_txs(
        &self,
        statuses: Option<&[SpendTxStatus]>,
    ) -> Result<ListSpendTransactionsResponse, RevaultDError> {
        self.call("listspendtxs", &[jsonrpc::arg(statuses)])
    }

    pub fn delete_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("delspendtx", &[jsonrpc::arg(txid)])?;
        Ok(())
    }

    pub fn broadcast_spend_tx(&self, txid: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("setspendtx", &[jsonrpc::arg(txid)])?;
        Ok(())
    }

    pub fn revault(&self, outpoint: &str) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("revault", &[jsonrpc::arg(outpoint)])?;
        Ok(())
    }

    pub fn emergency(&self) -> Result<(), RevaultDError> {
        let _res: serde_json::value::Value = self.call("emergency", &[])?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Request {}

/// getinfo

/// getinfo response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetInfoResponse {
    pub blockheight: u64,
    pub network: String,
    pub sync: f64,
    pub version: String,
}

/// list_vaults

/// listvaults response
#[derive(Debug, Clone, Deserialize)]
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

// RevaultD can start only if a config path is given.
pub async fn start_daemon(config_path: &Path, revaultd_path: &Path) -> Result<(), RevaultDError> {
    debug!("starting revaultd daemon");
    let mut child = Command::new(revaultd_path)
        .arg("--conf")
        .arg(config_path.to_path_buf().into_os_string().as_os_str())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            RevaultDError::StartError(format!("Failed to launched revaultd: {}", e.to_string()))
        })?;

    debug!("waiting for revaultd daemon status");

    let tries_timeout = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();

    while start.elapsed() < tries_timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    // FIXME: there should be a better way to collect the output...
                    let output = child.wait_with_output().unwrap();
                    return Err(RevaultDError::StartError(format!(
                        "Error revaultd terminated with status: {} and stderr:\n{:?}",
                        status.to_string(),
                        String::from_utf8_lossy(&output.stderr),
                    )));
                } else {
                    info!("revaultd daemon started");
                    return Ok(());
                }
            }
            Ok(None) => continue,
            Err(e) => {
                return Err(RevaultDError::StartError(format!(
                    "Child did not terminate: {}",
                    e.to_string()
                )));
            }
        }
    }

    return Err(RevaultDError::StartError(
        "Child did not terminate, do you have `daemon=false` in Revault conf?".to_string(),
    ));
}
