use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use iced::{Command, Element};
use std::sync::Arc;

use crate::{
    revault::TransactionKind,
    revaultd::{
        model::{self, RevocationTransactions, VaultStatus, VaultTransactions},
        RevaultD,
    },
    ui::{
        error::Error,
        message::{Message, SignMessage, VaultMessage},
        state::{
            cmd::{
                get_onchain_txs, get_revocation_txs, get_unvault_tx, revault, set_revocation_txs,
                set_unvault_tx,
            },
            sign::SignState,
        },
        view::{
            vault::{
                AcknowledgeVaultView, DelegateVaultView, RevaultVaultView, VaultModal,
                VaultOnChainTransactionsPanel, VaultView,
            },
            Context,
        },
    },
};

#[derive(Debug)]
pub struct VaultListItem<T> {
    pub vault: model::Vault,
    view: T,
}

impl<T: VaultView> VaultListItem<T> {
    pub fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            view: T::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault)
    }
}

/// SelectedVault is a widget displaying information of a vault
/// and handling user action on it.
#[derive(Debug)]
pub struct Vault {
    pub vault: model::Vault,
    warning: Option<Error>,
    section: VaultSection,
    view: VaultModal,
}

impl Vault {
    pub fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            section: VaultSection::Unloaded,
            view: VaultModal::new(),
            warning: None,
        }
    }

    pub fn update(
        &mut self,
        revaultd: Arc<RevaultD>,
        message: VaultMessage,
    ) -> Command<VaultMessage> {
        match message {
            VaultMessage::ListOnchainTransaction => {
                return Command::perform(
                    get_onchain_txs(revaultd.clone(), self.vault.outpoint()),
                    VaultMessage::OnChainTransactions,
                );
            }
            VaultMessage::OnChainTransactions(res) => match res {
                Ok(txs) => self.section = VaultSection::new_onchain_txs_section(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::UnvaultTransaction(res) => match res {
                Ok(tx) => self.section = VaultSection::new_delegate_section(tx.unvault_tx),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::RevocationTransactions(res) => match res {
                Ok(tx) => self.section = VaultSection::new_ack_section(tx),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::SelectRevault => {
                self.section = VaultSection::new_revault_section();
            }
            VaultMessage::Delegate => {
                return Command::perform(
                    get_unvault_tx(revaultd.clone(), self.vault.outpoint()),
                    VaultMessage::UnvaultTransaction,
                );
            }
            VaultMessage::Acknowledge => {
                return Command::perform(
                    get_revocation_txs(revaultd.clone(), self.vault.outpoint()),
                    VaultMessage::RevocationTransactions,
                );
            }
            _ => {
                return self.section.update(revaultd, &mut self.vault, message);
            }
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            &self.vault,
            self.warning.as_ref(),
            self.section.view(ctx, &self.vault),
        )
    }

    pub fn load(&self, revaultd: Arc<RevaultD>) -> Command<VaultMessage> {
        Command::perform(
            get_onchain_txs(revaultd.clone(), self.vault.outpoint()),
            VaultMessage::OnChainTransactions,
        )
    }
}

#[derive(Debug)]
pub enum VaultSection {
    Unloaded,
    OnchainTransactions {
        txs: VaultTransactions,
        view: VaultOnChainTransactionsPanel,
    },
    Delegate {
        signer: SignState,
        view: DelegateVaultView,
        warning: Option<Error>,
    },
    Acknowledge {
        emergency_tx: (Psbt, bool),
        emergency_unvault_tx: (Psbt, bool),
        cancel_tx: (Psbt, bool),
        warning: Option<Error>,
        view: AcknowledgeVaultView,
        signer: SignState,
    },
    /// Revault action ask the user if the vault that is unvaulting
    /// should be revaulted and executes the revault command after
    /// confirmation from the user.
    Revault {
        processing: bool,
        success: bool,
        warning: Option<Error>,
        view: RevaultVaultView,
    },
}

impl VaultSection {
    pub fn new_onchain_txs_section(txs: VaultTransactions) -> Self {
        Self::OnchainTransactions {
            txs,
            view: VaultOnChainTransactionsPanel::new(),
        }
    }

    pub fn new_delegate_section(unvault_tx: Psbt) -> Self {
        Self::Delegate {
            signer: SignState::new(unvault_tx, TransactionKind::Unvault),
            view: DelegateVaultView::new(),
            warning: None,
        }
    }

    pub fn new_revault_section() -> Self {
        Self::Revault {
            processing: false,
            success: false,
            view: RevaultVaultView::new(),
            warning: None,
        }
    }

    pub fn new_ack_section(txs: RevocationTransactions) -> Self {
        Self::Acknowledge {
            emergency_tx: (txs.emergency_tx.clone(), false),
            emergency_unvault_tx: (txs.emergency_unvault_tx.clone(), false),
            cancel_tx: (txs.cancel_tx.clone(), false),
            signer: SignState::new(txs.emergency_tx, TransactionKind::Emergency),
            view: AcknowledgeVaultView::new(),
            warning: None,
        }
    }

    fn update(
        &mut self,
        revaultd: Arc<RevaultD>,
        vault: &mut model::Vault,
        message: VaultMessage,
    ) -> Command<VaultMessage> {
        match message {
            VaultMessage::Revault => {
                if let Self::Revault {
                    processing,
                    warning,
                    ..
                } = self
                {
                    *processing = true;
                    *warning = None;
                    return Command::perform(
                        revault(revaultd.clone(), vault.outpoint()),
                        VaultMessage::Revaulted,
                    );
                }
            }
            VaultMessage::Revaulted(res) => {
                if let Self::Revault {
                    processing,
                    success,
                    warning,
                    ..
                } = self
                {
                    *processing = false;
                    match res {
                        Ok(()) => {
                            *success = true;
                            *warning = None;
                            vault.status = VaultStatus::Canceling;
                        }
                        Err(e) => *warning = Error::from(e).into(),
                    }
                }
            }
            VaultMessage::Signed(res) => match self {
                VaultSection::Delegate {
                    warning, signer, ..
                } => match res {
                    Ok(()) => {
                        *warning = None;
                        signer.update(SignMessage::Success);
                    }
                    Err(e) => {
                        *warning = Some(Error::RevaultDError(e));
                    }
                },
                VaultSection::Acknowledge {
                    warning, signer, ..
                } => match res {
                    Ok(()) => {
                        *warning = None;
                        signer.update(SignMessage::Success);
                    }
                    Err(e) => {
                        *warning = Some(Error::RevaultDError(e));
                    }
                },
                _ => {}
            },
            VaultMessage::Sign(msg) => match self {
                VaultSection::Delegate {
                    signer, warning, ..
                } => {
                    *warning = None;
                    signer.update(msg);
                    if let Some(psbt) = &signer.signed_psbt {
                        return Command::perform(
                            set_unvault_tx(revaultd.clone(), vault.outpoint(), psbt.clone()),
                            VaultMessage::Signed,
                        );
                    }
                }
                VaultSection::Acknowledge {
                    signer,
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                    warning,
                    ..
                } => {
                    *warning = None;
                    signer.update(msg);
                    if let Some(psbt) = &signer.signed_psbt {
                        match signer.transaction_kind {
                            TransactionKind::Emergency => {
                                *emergency_tx = (psbt.clone(), true);
                                *signer = SignState::new(
                                    emergency_unvault_tx.0.clone(),
                                    TransactionKind::EmergencyUnvault,
                                );
                            }
                            TransactionKind::EmergencyUnvault => {
                                *emergency_unvault_tx = (psbt.clone(), true);
                                *signer =
                                    SignState::new(cancel_tx.0.clone(), TransactionKind::Cancel);
                            }
                            TransactionKind::Cancel => {
                                *cancel_tx = (psbt.clone(), true);
                                return Command::perform(
                                    set_revocation_txs(
                                        revaultd,
                                        vault.outpoint(),
                                        emergency_tx.0.clone(),
                                        emergency_unvault_tx.0.clone(),
                                        cancel_tx.0.clone(),
                                    ),
                                    VaultMessage::Signed,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context, vault: &model::Vault) -> Element<Message> {
        let outpoint = vault.outpoint();
        match self {
            Self::Unloaded => iced::Container::new(iced::Column::new()).into(),
            Self::OnchainTransactions { txs, view } => view.view(ctx, &vault, &txs),
            Self::Delegate {
                signer,
                view,
                warning,
                ..
            } => view.view(ctx, &vault, warning.as_ref(), signer.view(ctx)),
            Self::Acknowledge {
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                warning,
                view,
                signer,
            } => view
                .view(
                    ctx,
                    warning.as_ref(),
                    vault,
                    &emergency_tx,
                    &emergency_unvault_tx,
                    &cancel_tx,
                    signer.view(ctx).map(VaultMessage::Sign),
                )
                .map(move |msg| Message::Vault(outpoint.clone(), msg)),
            Self::Revault {
                processing,
                success,
                warning,
                view,
            } => view.view(ctx, vault, &processing, &success, warning.as_ref()),
        }
    }
}
