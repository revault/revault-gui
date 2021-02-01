use bitcoin::{util::psbt::PartiallySignedTransaction, Transaction};
use serde::Deserialize;

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
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum VaultStatus {
    /// The deposit transaction is less than 6 blocks deep in the chain.
    #[serde(rename = "unconfirmed")]
    Unconfirmed,
    /// The deposit transaction is confirmed
    #[serde(rename = "funded")]
    Funded,
    /// The emergency transaction is signed
    #[serde(rename = "secured")]
    Secured,
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

#[derive(Debug, Clone, Deserialize)]
pub struct VaultTransactions {
    pub outpoint: String,
    pub deposit: BroadcastedTransaction,
    pub unvault: VaultTransaction,
    pub spend: Option<VaultTransaction>,
    pub cancel: VaultTransaction,
    pub emergency: VaultTransaction,
    pub unvault_emergency: VaultTransaction,
}

impl VaultTransactions {
    pub fn last_broadcasted_tx(&self) -> &BroadcastedTransaction {
        if let Some(VaultTransaction::Broadcasted(tx)) = &self.spend {
            return tx;
        }

        if let VaultTransaction::Broadcasted(tx) = &self.cancel {
            return tx;
        }

        if let VaultTransaction::Broadcasted(tx) = &self.unvault_emergency {
            return tx;
        }

        if let VaultTransaction::Broadcasted(tx) = &self.emergency {
            return tx;
        }

        if let VaultTransaction::Broadcasted(tx) = &self.unvault {
            return tx;
        }

        &self.deposit
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum VaultTransaction {
    Broadcasted(BroadcastedTransaction),
    Signed(SignedTransaction),
    Unsigned(UnsignedTransaction),
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
