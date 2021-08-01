use revault_tx::bitcoin::util::{bip32::DerivationPath, psbt::PartiallySignedTransaction};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
pub struct SignRequest {
    #[serde(deserialize_with = "deserialize_fromstr")]
    pub derivation_path: DerivationPath,
    pub target: SignTarget,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SignTarget {
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

fn deserialize_fromstr<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let string = String::deserialize(deserializer)?;
    T::from_str(&string)
        .map_err(|e| de::Error::custom(format!("Error parsing descriptor '{}': '{}'", string, e)))
}

#[derive(Debug)]
pub struct Error(String);

pub struct Signer {}

impl Signer {
    pub fn new() -> Signer {
        Self {}
    }

    pub fn sign_unvault_tx(
        &self,
        derivation_path: &DerivationPath,
        unvault_tx: &mut UnvaultTransaction,
    ) -> Result<(), Error> {
        Ok(())
    }
}
