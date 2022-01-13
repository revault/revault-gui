use async_trait::async_trait;

use bitcoin::{
    blockdata::transaction::OutPoint, util::psbt::PartiallySignedTransaction as Psbt, Amount,
};

use crate::{HWIError, HWI};

/// RevaultHWI is the common Revault Hardware Wallet Interface.
#[async_trait]
pub trait RevaultHWI: HWI {
    /// Returns true if the device is able to secure and delegate vaults
    /// by creating and signing itself the revocation transactions and the
    /// unvault transaction from internal descriptors.
    async fn has_revault_app(&mut self) -> bool;

    /// Sign the revocation transactions.
    async fn sign_revocation_txs(
        &mut self,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(Psbt, Psbt, Psbt), HWIError>;

    /// Sign the unvault transaction required for delegation.
    async fn sign_unvault_tx(&mut self, unvault_tx: &Psbt) -> Result<Psbt, HWIError>;

    /// Create vaults from deposits by giving the utxos to the hardware wallet storing the
    /// descriptors and deriving itself the revocation transactions.
    async fn create_vaults(
        &mut self,
        deposits: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<(Psbt, Psbt, Psbt)>, HWIError>;

    /// Delegate a list of vaults by giving the utxos to an hardware wallet storing the
    /// descriptors and deriving itself the unvault transactions.
    async fn delegate_vaults(
        &mut self,
        vaults: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<Psbt>, HWIError>;
}

pub trait NoRevaultApp {}

#[async_trait]
impl<T: HWI + NoRevaultApp + Send> RevaultHWI for T {
    async fn has_revault_app(&mut self) -> bool {
        false
    }

    async fn sign_revocation_txs(
        &mut self,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(Psbt, Psbt, Psbt), HWIError> {
        let emergency_tx = self.sign_tx(emergency_tx).await?;
        let emergency_unvault_tx = self.sign_tx(emergency_unvault_tx).await?;
        let cancel_tx = self.sign_tx(cancel_tx).await?;
        Ok((emergency_tx, emergency_unvault_tx, cancel_tx))
    }

    async fn sign_unvault_tx(&mut self, unvault_tx: &Psbt) -> Result<Psbt, HWIError> {
        self.sign_tx(unvault_tx).await.map_err(|e| e.into())
    }

    async fn create_vaults(
        &mut self,
        _deposits: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<(Psbt, Psbt, Psbt)>, HWIError> {
        Err(HWIError::UnimplementedMethod)
    }

    async fn delegate_vaults(
        &mut self,
        _vaults: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<Psbt>, HWIError> {
        Err(HWIError::UnimplementedMethod)
    }
}
