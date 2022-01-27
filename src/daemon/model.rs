use bitcoin::{util::psbt::PartiallySignedTransaction, OutPoint, Transaction, Txid};
use serde::{Deserialize, Serialize};

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

/// gethistory response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetHistoryResponse {
    pub events: Vec<HistoryEvent>,
}

/// list_transactions



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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    /// Address of the vault deposit
    pub address: String,
    /// Amount of the vault in satoshis
    pub amount: u64,
    /// derivation_index is the index used to create scriptPubKey of the deposit address
    pub derivation_index: u32,
    /// Status of the vault
    pub status: VaultStatus,
    /// Deposit txid of the vault deposit transaction
    pub txid: Txid,
    /// Deposit vout of the vault deposit transaction
    pub vout: u32,
}

impl Vault {
    // Todo: return OutPoint
    pub fn outpoint(&self) -> String {
        OutPoint::new(self.txid, self.vout).to_string()
    }
}

/// The status of a [Vault], depends both on the block chain and the set of pre-signed
/// transactions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VaultStatus {
    /// The deposit transaction is less than 6 blocks deep in the chain.
    #[serde(rename = "unconfirmed")]
    Unconfirmed,
    /// The deposit transaction is confirmed
    #[serde(rename = "funded")]
    Funded,
    /// The emergency transaction is signed by us
    #[serde(rename = "securing")]
    Securing,
    /// The emergency transaction is fully signed
    #[serde(rename = "secured")]
    Secured,
    /// The unvault transaction is signed by the stakeholder.
    #[serde(rename = "activating")]
    Activating,
    /// The unvault transaction is signed (implies that the second emergency and the
    /// cancel transaction are signed).
    #[serde(rename = "active")]
    Active,
    /// The unvault transaction has been broadcast
    #[serde(rename = "unvaulting")]
    Unvaulting,
    /// The unvault transaction is confirmed
    #[serde(rename = "unvaulted")]
    Unvaulted,
    /// The cancel transaction has been broadcast
    #[serde(rename = "canceling")]
    Canceling,
    /// The cancel transaction is confirmed
    #[serde(rename = "canceled")]
    Canceled,
    /// One of the emergency transactions has been broadcast
    #[serde(rename = "emergencyvaulting")]
    EmergencyVaulting,
    /// One of the emergency transactions is confirmed
    #[serde(rename = "emergencyvaulted")]
    EmergencyVaulted,
    /// The unvault emergency transactions has been broadcast
    #[serde(rename = "unvaultemergencyvaulting")]
    UnvaultEmergencyVaulting,
    /// The unvault emergency transactions is confirmed
    #[serde(rename = "unvaultemergencyvaulted")]
    UnvaultEmergencyVaulted,
    /// The spend transaction has been broadcast
    #[serde(rename = "spending")]
    Spending,
    /// The spend transaction is confirmed
    #[serde(rename = "spent")]
    Spent,
}

impl std::fmt::Display for VaultStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unconfirmed => write!(f, "Deposit unconfirmed"),
            Self::Funded => write!(f, "Deposit funded"),
            Self::Securing => write!(f, "Securing"),
            Self::Secured => write!(f, "Secured"),
            Self::Activating => write!(f, "Activating"),
            Self::Active => write!(f, "Active"),
            Self::Unvaulting => write!(f, "Unvaulting"),
            Self::Unvaulted => write!(f, "Unvaulted"),
            Self::Canceling => write!(f, "Canceling"),
            Self::Canceled => write!(f, "Canceled"),
            Self::EmergencyVaulting => write!(f, "Emergency vaulting"),
            Self::EmergencyVaulted => write!(f, "Emergency vaulted"),
            Self::UnvaultEmergencyVaulting => write!(f, "Unvault Emergency vaulting"),
            Self::UnvaultEmergencyVaulted => write!(f, "Unvault Emergency vaulted"),
            Self::Spending => write!(f, "Spending"),
            Self::Spent => write!(f, "Spent"),
        }
    }
}

impl VaultStatus {
    pub const DEPOSIT_AND_CURRENT: [VaultStatus; 11] = [
        Self::Funded,
        Self::Securing,
        Self::Secured,
        Self::Activating,
        Self::Active,
        Self::Unvaulting,
        Self::Unvaulted,
        Self::Canceling,
        Self::EmergencyVaulting,
        Self::UnvaultEmergencyVaulting,
        Self::Spending,
    ];

    pub const CURRENT: [VaultStatus; 10] = [
        Self::Securing,
        Self::Secured,
        Self::Activating,
        Self::Active,
        Self::Unvaulting,
        Self::Unvaulted,
        Self::Canceling,
        Self::EmergencyVaulting,
        Self::UnvaultEmergencyVaulting,
        Self::Spending,
    ];

    pub const ACTIVE: [VaultStatus; 1] = [Self::Active];

    pub const INACTIVE: [VaultStatus; 4] = [
        Self::Funded,
        Self::Securing,
        Self::Secured,
        Self::Activating,
    ];

    pub const MOVING: [VaultStatus; 6] = [
        Self::Unvaulting,
        Self::Unvaulted,
        Self::Canceling,
        Self::EmergencyVaulting,
        Self::UnvaultEmergencyVaulting,
        Self::Spending,
    ];

    pub const MOVED: [VaultStatus; 4] = [
        Self::Canceled,
        Self::EmergencyVaulted,
        Self::UnvaultEmergencyVaulted,
        Self::Spent,
    ];
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct ServersStatuses {
    pub coordinator: ServerStatus,
    pub cosigners: Vec<ServerStatus>,
    pub watchtowers: Vec<ServerStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStatus {
    pub host: String,
    pub reachable: bool,
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
