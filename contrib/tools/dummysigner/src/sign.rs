use revault_tx::{
    bitcoin::{
        blockdata::transaction::OutPoint,
        secp256k1,
        util::{
            bip143::SigHashCache,
            bip32::{ChildNumber, ExtendedPrivKey},
            psbt::PartiallySignedTransaction,
        },
        Amount, SigHashType,
    },
    scripts::{CpfpDescriptor, DepositDescriptor, EmergencyAddress, UnvaultDescriptor},
    transactions::{transaction_chain, RevaultTransaction, UnvaultTransaction},
    txins::DepositTxIn,
    txouts::DepositTxOut,
};

#[derive(Debug)]
pub struct Error(String);

pub struct Signer {
    descriptors: Option<Descriptors>,
    emergency_address: Option<EmergencyAddress>,
    curve: secp256k1::Secp256k1<secp256k1::All>,
}

pub struct Descriptors {
    pub deposit_descriptor: DepositDescriptor,
    pub unvault_descriptor: UnvaultDescriptor,
    pub cpfp_descriptor: CpfpDescriptor,
}

impl Signer {
    pub fn new(
        descriptors: Option<Descriptors>,
        emergency_address: Option<EmergencyAddress>,
    ) -> Signer {
        Self {
            descriptors,
            emergency_address,
            curve: secp256k1::Secp256k1::new(),
        }
    }

    pub fn has_descriptors(&self) -> bool {
        self.descriptors.is_some()
    }

    pub fn has_emergency_address(&self) -> bool {
        self.emergency_address.is_some()
    }

    pub fn requires_key_for_psbt(
        &self,
        key: &ExtendedPrivKey,
        psbt: &PartiallySignedTransaction,
    ) -> bool {
        let key_fingerprint = key.fingerprint(&self.curve);
        for input in &psbt.inputs {
            for (_, (fingerprint, _)) in &input.bip32_derivation {
                if key_fingerprint == *fingerprint {
                    return true;
                }
            }
        }

        false
    }

    /// Sign the psbt with the given keys.
    pub fn sign_psbt(
        &self,
        keys: &Vec<ExtendedPrivKey>,
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

            for (pubkey, (fingerprint, derivation_path)) in &input.bip32_derivation {
                for xkey in keys {
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

    pub fn derive_revocation_txs(
        &self,
        outpoint: OutPoint,
        amount: Amount,
        derivation_index: ChildNumber,
    ) -> Result<RevocationTransactions, Error> {
        let descriptors = self
            .descriptors
            .as_ref()
            .ok_or(Error("Wallet does not have the descriptors".to_string()))?;
        let emer_address = self
            .emergency_address
            .as_ref()
            .ok_or(Error("Wallet does not have emergency_address".to_string()))?;
        let (_, cancel_txs, emergency_tx, emergency_unvault_tx) = transaction_chain(
            outpoint,
            amount,
            &descriptors.deposit_descriptor,
            &descriptors.unvault_descriptor,
            &descriptors.cpfp_descriptor,
            derivation_index,
            emer_address.clone(),
            &self.curve,
        )
        .map_err(|e| Error(e.to_string()))?;
        let cancel_txs = cancel_txs.all_feerates();

        Ok(RevocationTransactions {
            cancel_txs: [
                cancel_txs[0].psbt().clone(),
                cancel_txs[1].psbt().clone(),
                cancel_txs[2].psbt().clone(),
                cancel_txs[3].psbt().clone(),
                cancel_txs[4].psbt().clone(),
            ],
            emergency_tx: emergency_tx.into_psbt(),
            emergency_unvault_tx: emergency_unvault_tx.into_psbt(),
        })
    }

    pub fn derive_unvault_tx(
        &self,
        outpoint: OutPoint,
        amount: Amount,
        derivation_index: ChildNumber,
    ) -> Result<PartiallySignedTransaction, Error> {
        let descriptors = self
            .descriptors
            .as_ref()
            .ok_or(Error("Wallet does not have the descriptors".to_string()))?;

        let deposit_descriptor = descriptors
            .deposit_descriptor
            .derive(derivation_index, &self.curve);
        let deposit_txin =
            DepositTxIn::new(outpoint, DepositTxOut::new(amount, &deposit_descriptor));
        let unvault_descriptor = descriptors
            .unvault_descriptor
            .derive(derivation_index, &self.curve);
        let cpfp_descriptor = descriptors
            .cpfp_descriptor
            .derive(derivation_index, &self.curve);

        let unvault_tx =
            UnvaultTransaction::new(deposit_txin, &unvault_descriptor, &cpfp_descriptor)
                .map_err(|e| Error(e.to_string()))?;

        Ok(unvault_tx.into_psbt())
    }
}

pub struct RevocationTransactions {
    pub cancel_txs: [PartiallySignedTransaction; 5],
    pub emergency_tx: PartiallySignedTransaction,
    pub emergency_unvault_tx: PartiallySignedTransaction,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use revault_tx::bitcoin::{
        blockdata::transaction::OutPoint, consensus::encode, util::bip32::ChildNumber, Amount,
    };
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    fn derive_revocation_txs() {
        let cfg = Config::from_file(&PathBuf::from("examples/examples_cfg.toml")).unwrap();
        let signer = Signer::new(
            cfg.descriptors.map(|d| Descriptors {
                deposit_descriptor: d.deposit_descriptor,
                unvault_descriptor: d.unvault_descriptor,
                cpfp_descriptor: d.cpfp_descriptor,
            }),
            cfg.emergency_address,
        );

        let revocation_txs = signer
            .derive_revocation_txs(
                OutPoint::from_str(
                    "899aecbc9a3b06feaf096fc576b35da352a1ca1aa0c34db23ccfa944f30fae47:1",
                )
                .unwrap(),
                Amount::from_sat(120000000),
                ChildNumber::from_normal_idx(0).unwrap(),
            )
            .unwrap();

        assert_eq!(
            base64::encode(encode::serialize(&revocation_txs.cancel_txs[0])).to_string(),
            "cHNidP8BAF4CAAAAATdzv51EXeeNc1fv6E852OhRxc67KNaWd+BrA3qN1a/1AAAAAAD9////ASp5JgcAAAAAIgAgdfJpF3TIFneDGEawKCIA4oiyxZcQtY90MYPUklUH28UAAAAAAAEBK7iGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwBBWEhAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmrFGHZHapFHKpXyKvmhuuuFL5qVJy+MIdmPJkiKxrdqkUtsmtuJyMk3Jsg+KhtdlHidd7lWGIrGyTUodnWLJoIgYCApM6sQN+rQqkp4WMbUnWcF4fs7xfZrAJGK97nwDs1SYIJR1gCQAAAAAiBgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACIGA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAA"
        );
        assert_eq!(
            base64::encode(encode::serialize(&revocation_txs.emergency_tx)).to_string(),
            "cHNidP8BAF4CAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////ARDEJAcAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBKwAOJwcAAAAAIgAgdfJpF3TIFneDGEawKCIA4oiyxZcQtY90MYPUklUH28UBBUdSIQJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQyEDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ5SriIGAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgYDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAAAA=="
        );
        assert_eq!(
            base64::encode(encode::serialize(&revocation_txs.emergency_unvault_tx)).to_string(),
            "cHNidP8BAF4CAAAAATdzv51EXeeNc1fv6E852OhRxc67KNaWd+BrA3qN1a/1AAAAAAD9////AfzgIwcAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBK7iGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwBBWEhAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmrFGHZHapFHKpXyKvmhuuuFL5qVJy+MIdmPJkiKxrdqkUtsmtuJyMk3Jsg+KhtdlHidd7lWGIrGyTUodnWLJoIgYCApM6sQN+rQqkp4WMbUnWcF4fs7xfZrAJGK97nwDs1SYIJR1gCQAAAAAiBgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACIGA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAAAA="
        );
    }

    #[test]
    fn derive_unvault_tx() {
        let cfg = Config::from_file(&PathBuf::from("examples/examples_cfg.toml")).unwrap();
        let signer = Signer::new(
            cfg.descriptors.map(|d| Descriptors {
                deposit_descriptor: d.deposit_descriptor,
                unvault_descriptor: d.unvault_descriptor,
                cpfp_descriptor: d.cpfp_descriptor,
            }),
            cfg.emergency_address,
        );

        let unvault_tx = signer
            .derive_unvault_tx(
                OutPoint::from_str(
                    "899aecbc9a3b06feaf096fc576b35da352a1ca1aa0c34db23ccfa944f30fae47:1",
                )
                .unwrap(),
                Amount::from_sat(120000000),
                ChildNumber::from_normal_idx(0).unwrap(),
            )
            .unwrap();

        assert_eq!(
            base64::encode(encode::serialize(&unvault_tx)).to_string(),
            "cHNidP8BAIkCAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////AriGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwwdQAAAAAAACIAIAjkMa8elv7dHUmYpDATWBtmMmpv9yyKFawMunvGQ1AMAAAAAAABASsADicHAAAAACIAIHXyaRd0yBZ3gxhGsCgiAOKIssWXELWPdDGD1JJVB9vFAQVHUiECWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MhA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOUq4iBgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACIGA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAACICAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmCCUdYAkAAAAAIgICWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiAgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgJQcvTgdleKXInUHXXx7VKav5LxzWq2Nrjot7sPMwJpQwj73ezNAAAAAAA="
        );
    }
}
