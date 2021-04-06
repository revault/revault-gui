use bitcoin::{util::psbt::PartiallySignedTransaction, Transaction};
use serde::{Deserialize, Serialize};

/// getdepositaddress response
#[derive(Debug, Clone, Deserialize)]
pub struct DepositAddress {
    pub address: bitcoin::Address,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Vault {
    /// Address of the vault deposit
    pub address: String,
    /// Amount of the vault in satoshis
    pub amount: u64,
    /// Status of the vault
    pub status: VaultStatus,
    /// Deposit txid of the vault deposit transaction
    pub txid: String,
    /// Timestamp of the last vault update.
    pub updated_at: i64,
    /// Deposit vout of the vault deposit transaction
    pub vout: u32,
}

impl Vault {
    pub fn outpoint(&self) -> String {
        format!("{}:{}", self.txid, self.vout)
    }
}

/// The status of a [Vault], depends both on the block chain and the set of pre-signed
/// transactions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// The unvault transaction CSV is expired
    #[serde(rename = "spendable")]
    Spendable,
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
            Self::Spendable => write!(f, "Spendable"),
            Self::Spending => write!(f, "Spending"),
            Self::Spent => write!(f, "Spent"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct BroadcastedTransaction {
    /// Height of the block containing the transaction.
    pub blockheight: Option<u64>,
    #[serde(rename = "hex", with = "bitcoin_transaction")]
    pub tx: Transaction,
    /// reception time as Unix Epoch timestamp
    pub received_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SignedTransaction {
    #[serde(rename = "hex", with = "bitcoin_transaction")]
    pub tx: Transaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnsignedTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub psbt: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RevocationTransactions {
    #[serde(with = "bitcoin_psbt")]
    pub cancel_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnvaultTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpendTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub spend_tx: PartiallySignedTransaction,

    #[serde(skip)]
    pub feerate: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpendTx {
    #[serde(with = "bitcoin_psbt")]
    pub psbt: PartiallySignedTransaction,
    pub deposit_outpoints: Vec<String>,
}

mod bitcoin_transaction {
    use bitcoin::{consensus::encode, hashes::hex::FromHex, Transaction};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Transaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = Vec::from_hex(&s).map_err(serde::de::Error::custom)?;
        encode::deserialize::<Transaction>(&bytes).map_err(serde::de::Error::custom)
    }
}

mod bitcoin_psbt {
    use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PartiallySignedTransaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes: Vec<u8> = base64::decode(&s).map_err(serde::de::Error::custom)?;
        encode::deserialize(&bytes).map_err(serde::de::Error::custom)
    }
}
