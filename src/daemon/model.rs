use bitcoin::{util::psbt::PartiallySignedTransaction, Transaction, Txid};
use serde::{Deserialize, Serialize};

use revaultd::commands::ListVaultsEntry;
pub use revaultd::commands::{GetInfoResult, ServerStatus, ServersStatuses, VaultStatus};

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

pub type Vault = ListVaultsEntry;

pub const DEPOSIT_AND_CURRENT_VAULT_STATUSES: [VaultStatus; 11] = [
    VaultStatus::Funded,
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
    VaultStatus::Active,
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const CURRENT_VAULT_STATUSES: [VaultStatus; 10] = [
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
    VaultStatus::Active,
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const ACTIVE_VAULT_STATUSES: [VaultStatus; 1] = [VaultStatus::Active];

pub const INACTIVE_VAULT_STATUSES: [VaultStatus; 4] = [
    VaultStatus::Funded,
    VaultStatus::Securing,
    VaultStatus::Secured,
    VaultStatus::Activating,
];

pub const MOVING_VAULT_STATUSES: [VaultStatus; 6] = [
    VaultStatus::Unvaulting,
    VaultStatus::Unvaulted,
    VaultStatus::Canceling,
    VaultStatus::EmergencyVaulting,
    VaultStatus::UnvaultEmergencyVaulting,
    VaultStatus::Spending,
];

pub const MOVED_VAULT_STATUSES: [VaultStatus; 4] = [
    VaultStatus::Canceled,
    VaultStatus::EmergencyVaulted,
    VaultStatus::UnvaultEmergencyVaulted,
    VaultStatus::Spent,
];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpendTxStatus {
    #[serde(rename = "non_final")]
    NonFinal,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "broadcasted")]
    Broadcasted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultTransactions {
    pub vault_outpoint: String,
    pub deposit: BroadcastedTransaction,
    pub unvault: Option<BroadcastedTransaction>,
    pub spend: Option<BroadcastedTransaction>,
    pub cancel: Option<BroadcastedTransaction>,
    pub emergency: Option<BroadcastedTransaction>,
    pub unvault_emergency: Option<BroadcastedTransaction>,
}

impl VaultTransactions {
    pub fn last_broadcasted_tx(&self) -> &BroadcastedTransaction {
        if let Some(tx) = &self.spend {
            return tx;
        }
        if let Some(tx) = &self.cancel {
            return tx;
        }
        if let Some(tx) = &self.unvault_emergency {
            return tx;
        }
        if let Some(tx) = &self.emergency {
            return tx;
        }
        if let Some(tx) = &self.unvault {
            return tx;
        }
        &self.deposit
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastedTransaction {
    /// Height of the block containing the transaction.
    pub blockheight: Option<u64>,
    #[serde(rename = "hex", with = "bitcoin_transaction")]
    pub tx: Transaction,
    /// reception time as Unix Epoch timestamp
    pub received_at: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignedTransaction {
    #[serde(rename = "hex", with = "bitcoin_transaction")]
    pub tx: Transaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnsignedTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub psbt: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RevocationTransactions {
    #[serde(with = "bitcoin_psbt")]
    pub cancel_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnvaultTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpendTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub spend_tx: PartiallySignedTransaction,

    #[serde(skip)]
    pub feerate: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpendTx {
    #[serde(with = "bitcoin_psbt")]
    pub psbt: PartiallySignedTransaction,
    pub deposit_outpoints: Vec<String>,
    pub change_index: Option<usize>,
    pub cpfp_index: usize,
}

mod bitcoin_transaction {
    use bitcoin::{
        consensus::encode,
        hashes::hex::{FromHex, ToHex},
        Transaction,
    };
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Transaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = Vec::from_hex(&s).map_err(serde::de::Error::custom)?;
        encode::deserialize::<Transaction>(&bytes).map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(tx: &Transaction, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&encode::serialize(&tx).to_hex())
    }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub blockheight: u32,
    pub txid: Txid,
    pub date: i64,
    pub kind: HistoryEventKind,
    pub amount: Option<u64>,
    pub fee: Option<u64>,
    pub vaults: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HistoryEventKind {
    #[serde(rename = "cancel")]
    Cancel,
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "spend")]
    Spend,
}

impl HistoryEventKind {
    pub const ALL: [HistoryEventKind; 3] = [Self::Cancel, Self::Deposit, Self::Spend];
}

impl std::fmt::Display for HistoryEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Deposit => write!(f, "Deposit"),
            Self::Spend => write!(f, "Spend"),
        }
    }
}
