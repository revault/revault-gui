use revault_tx::bitcoin::{
    secp256k1,
    util::{bip143::SigHashCache, bip32::ExtendedPrivKey, psbt::PartiallySignedTransaction},
    SigHashType,
};

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

    pub fn sign_psbt(&self, psbt: &mut PartiallySignedTransaction) -> Result<(), Error> {
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

            for (pubkey, (fingerprint, derivation_path)) in &input.bip32_derivation {
                for xkey in &self.keys {
                    if xkey.fingerprint(&self.curve) == *fingerprint {
                        let pkey = xkey
                            .derive_priv(&self.curve, &derivation_path)
                            .map_err(|e| Error(e.to_string()))?
                            .private_key;

                        let mut signature = self
                            .curve
                            .sign(&sighash, &pkey.key)
                            .serialize_der()
                            .to_vec();
                        signature.push(sighash_type.as_u32() as u8);

                        if *pubkey == pkey.public_key(&self.curve) {
                            input.partial_sigs.insert(pubkey.clone(), signature);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
