use bitcoin::{
    consensus::encode, hashes::hex::FromHex, util::psbt::PartiallySignedTransaction, Transaction,
};
use serde::{Deserialize, Serialize};

use revaultd::commands::ListVaultsEntry;
pub use revaultd::commands::{
    GetInfoResult, HistoryEvent, HistoryEventKind, ListOnchainTxEntry, ListSpendEntry,
    ListSpendStatus, RevocationTransactions, ServerStatus, ServersStatuses, VaultStatus,
    WalletTransaction,
};

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

pub type SpendTxStatus = ListSpendStatus;

pub type VaultTransactions = ListOnchainTxEntry;

pub fn transaction_from_hex(hex: &str) -> Transaction {
    let bytes = Vec::from_hex(&hex).unwrap();
    encode::deserialize::<Transaction>(&bytes).unwrap()
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

pub type SpendTx = ListSpendEntry;

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

pub const ALL_HISTORY_EVENTS: [HistoryEventKind; 3] = [
    HistoryEventKind::Cancel,
    HistoryEventKind::Deposit,
    HistoryEventKind::Spend,
];
