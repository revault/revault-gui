use bitcoin::{util::psbt::PartiallySignedTransaction, Transaction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vault {
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
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VaultTransactions {
    pub outpoint: String,
    pub deposit: VaultTransaction,
    pub unvault: VaultTransaction,
    pub spend: VaultTransaction,
    pub cancel: VaultTransaction,
    pub emergency: VaultTransaction,
    pub unvault_emergency: VaultTransaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum VaultTransaction {
    Signed {
        #[serde(rename = "hex")]
        tx: Transaction,
    },
    Broadcasted {
        /// Height of the block containing the transaction.
        blockheight: u64,
        #[serde(rename = "hex")]
        tx: Transaction,
        /// reception time as Unix Epoch timestamp
        received_at: u64,
    },
    Unsigned {
        psbt: PartiallySignedTransaction,
    },
}
