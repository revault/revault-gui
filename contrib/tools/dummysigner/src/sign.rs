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
        derivation_path: &DerivationPath,
        spend_tx: &mut SpendTransaction,
    ) -> Result<(), Error> {
        self.sign_psbt(derivation_path, &mut spend_tx.spend_tx)
    }

    pub fn sign_unvault_tx(
        &self,
        derivation_path: &DerivationPath,
        unvault_tx: &mut UnvaultTransaction,
    ) -> Result<(), Error> {
        self.sign_psbt(derivation_path, &mut unvault_tx.unvault_tx)
    }

    pub fn sign_revocation_txs(
        &self,
        derivation_path: &DerivationPath,
        revocation_txs: &mut RevocationTransactions,
    ) -> Result<(), Error> {
        self.sign_psbt(derivation_path, &mut revocation_txs.emergency_tx)?;
        self.sign_psbt(derivation_path, &mut revocation_txs.emergency_unvault_tx)?;
        self.sign_psbt(derivation_path, &mut revocation_txs.cancel_tx)
    }

    fn sign_psbt(
        &self,
        derivation_path: &DerivationPath,
        psbt: &mut PartiallySignedTransaction,
    ) -> Result<(), Error> {
        for (input_index, input) in psbt.inputs.iter_mut().enumerate() {
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
        }

        Ok(())
    }
}
