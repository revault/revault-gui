use revault_tx::bitcoin::{
    secp256k1,
    util::{
        bip143::SigHashCache,
        bip32::{DerivationPath, ExtendedPrivKey},
        psbt::PartiallySignedTransaction,
    },
    SigHashType,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SignRequest {
    RevocationTransactions {
        #[serde(deserialize_with = "deserialize_fromstr")]
        derivation_path: DerivationPath,
        target: RevocationTransactions,
    },
    UnvaultTransaction {
        #[serde(deserialize_with = "deserialize_fromstr")]
        derivation_path: DerivationPath,
        target: UnvaultTransaction,
    },
    SpendTransaction {
        /// vec of derivation path following the order of each spend psbt input
        #[serde(deserialize_with = "deserialize_vec_fromstr")]
        derivation_paths: Vec<DerivationPath>,
        target: SpendTransaction,
    },
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

fn deserialize_vec_fromstr<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    let v = Vec::<String>::deserialize(deserializer)?;
    let res: Vec<T> = v.iter().filter_map(|s| T::from_str(&s).ok()).collect();
    if res.len() != v.len() {
        return Err(de::Error::custom("Failed to deserialize vec"));
    }
    Ok(res)
}

#[derive(Debug)]
pub struct Error(String);

pub struct Signer {
    keys: Vec<ExtendedPrivKey>,
    curve: secp256k1::Secp256k1<secp256k1::SignOnly>,
}

impl Signer {
    pub fn new(keys: Vec<ExtendedPrivKey>) -> Signer {
        Self {
            keys,
            curve: secp256k1::Secp256k1::signing_only(),
        }
    }

    pub fn sign_spend_tx(
        &self,
        derivation_path: &Vec<DerivationPath>,
        spend_tx: &mut SpendTransaction,
    ) -> Result<(), Error> {
        for (index, path) in derivation_path.iter().enumerate() {
            self.sign_psbt_input(path, &mut spend_tx.spend_tx, index)?;
        }
        Ok(())
    }

    pub fn sign_unvault_tx(
        &self,
        derivation_path: &DerivationPath,
        unvault_tx: &mut UnvaultTransaction,
    ) -> Result<(), Error> {
        self.sign_psbt_input(derivation_path, &mut unvault_tx.unvault_tx, 0)
    }

    pub fn sign_revocation_txs(
        &self,
        derivation_path: &DerivationPath,
        revocation_txs: &mut RevocationTransactions,
    ) -> Result<(), Error> {
        self.sign_psbt_input(derivation_path, &mut revocation_txs.emergency_tx, 0)?;
        self.sign_psbt_input(derivation_path, &mut revocation_txs.emergency_unvault_tx, 0)?;
        self.sign_psbt_input(derivation_path, &mut revocation_txs.cancel_tx, 0)
    }

    fn sign_psbt_input(
        &self,
        derivation_path: &DerivationPath,
        psbt: &mut PartiallySignedTransaction,
        input_index: usize,
    ) -> Result<(), Error> {
        let input = psbt
            .inputs
            .get_mut(input_index)
            .ok_or_else(|| Error(format!("Psbt has no input at index {}", input_index)))?;

        let prev_value = input
            .witness_utxo
            .as_ref()
            .ok_or_else(|| Error(format!("Psbt has no witness utxo for input '{:?}'", input)))?
            .value;

        let script_code = input.witness_script.as_ref().ok_or_else(|| {
            Error("Psbt input has no witness Script. P2WSH is only supported".to_string())
        })?;

        let sighash_type = input.sighash_type.unwrap_or(SigHashType::All);

        let sighash = SigHashCache::new(&psbt.global.unsigned_tx).signature_hash(
            input_index,
            &script_code,
            prev_value,
            sighash_type,
        );

        let sighash = secp256k1::Message::from_slice(&sighash).expect("Sighash is 32 bytes");

        for xkey in &self.keys {
            let pkey = xkey
                .derive_priv(&self.curve, derivation_path)
                .map_err(|e| Error(e.to_string()))?
                .private_key;

            let mut signature = self
                .curve
                .sign(&sighash, &pkey.key)
                .serialize_der()
                .to_vec();
            signature.push(sighash_type.as_u32() as u8);

            let pubkey = pkey.public_key(&self.curve);

            input.partial_sigs.insert(pubkey, signature);
        }

        Ok(())
    }
}
