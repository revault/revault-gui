use revault_tx::bitcoin::util::psbt::PartiallySignedTransaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Request {
    RevocationTransactions(RevocationTransactions),
    UnvaultTransaction(UnvaultTransaction),
    SpendTransaction(SpendTransaction),
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
