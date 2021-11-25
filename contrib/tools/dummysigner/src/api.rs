use revault_tx::bitcoin::{
    blockdata::transaction::OutPoint,
    util::{bip32::ChildNumber, psbt::PartiallySignedTransaction},
    Amount,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Request {
    RevocationTransactions(RevocationTransactions),
    UnvaultTransaction(UnvaultTransaction),
    SpendTransaction(SpendTransaction),
    SecureBatch(SecureBatch),
    DelegateBatch(DelegateBatch),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecureBatch {
    pub deposits: Vec<UTXO>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DelegateBatch {
    pub vaults: Vec<UTXO>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationTransactions {
    #[serde(with = "bitcoin_psbt")]
    pub cancel_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_tx: PartiallySignedTransaction,

    #[serde(with = "bitcoin_psbt")]
    pub emergency_unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnvaultTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub unvault_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendTransaction {
    #[serde(with = "bitcoin_psbt")]
    pub spend_tx: PartiallySignedTransaction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UTXO {
    #[serde(with = "bitcoin_outpoint")]
    pub outpoint: OutPoint,
    #[serde(with = "bitcoin_amount")]
    pub amount: Amount,
    #[serde(with = "bitcoin_derivation_index")]
    pub derivation_index: ChildNumber,
}

mod bitcoin_outpoint {
    use revault_tx::bitcoin::blockdata::transaction::OutPoint;
    use serde::{self, Deserialize, Deserializer};
    use std::str::FromStr;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OutPoint, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)
            .and_then(|s| OutPoint::from_str(&s).map_err(serde::de::Error::custom))
    }
}

mod bitcoin_amount {
    use revault_tx::bitcoin::Amount;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Amount, D::Error>
    where
        D: Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(|a| Amount::from_sat(a))
    }
}

mod bitcoin_derivation_index {
    use revault_tx::bitcoin::util::bip32::ChildNumber;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ChildNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        u32::deserialize(deserializer)
            .and_then(|i| ChildNumber::from_normal_idx(i).map_err(serde::de::Error::custom))
    }
}

mod bitcoin_psbt {
    use revault_tx::bitcoin::{consensus::encode, util::psbt::PartiallySignedTransaction};
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
        serializer.serialize_str(&base64::encode(encode::serialize(&psbt)))
    }
}
