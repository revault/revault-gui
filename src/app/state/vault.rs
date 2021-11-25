use iced::{Command, Element};
use std::sync::Arc;

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, VaultMessage},
        state::cmd::{get_onchain_txs, revault},
        view::vault::{RevaultVaultView, VaultModal, VaultOnChainTransactionsPanel, VaultView},
    },
    daemon::{
        client::{Client, RevaultD},
        model::{self, VaultStatus, VaultTransactions},
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

    pub fn view<C: Client>(&mut self, ctx: &Context<C>) -> Element<Message> {
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

    pub fn update<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
        message: VaultMessage,
    ) -> Command<VaultMessage> {
        match message {
            VaultMessage::ListOnchainTransaction => {
                return Command::perform(
                    get_onchain_txs(ctx.revaultd.clone(), self.vault.outpoint()),
                    VaultMessage::OnChainTransactions,
                );
            }
            VaultMessage::OnChainTransactions(res) => match res {
                Ok(txs) => self.section = VaultSection::new_onchain_txs_section(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            VaultMessage::SelectRevault => {
                self.section = VaultSection::new_revault_section();
            }
            _ => {
                return self
                    .section
                    .update(ctx.revaultd.clone(), &mut self.vault, message);
            }
        };
        Command::none()
    }

    pub fn view<C: Client>(&mut self, ctx: &Context<C>) -> Element<Message> {
        self.view.view(
            ctx,
            &self.vault,
            self.warning.as_ref(),
            self.section.title(&self.vault),
            self.section.view(ctx, &self.vault),
        )
    }

    pub fn load<C: Client + Sync + Send + 'static>(
        &self,
        revaultd: Arc<RevaultD<C>>,
    ) -> Command<VaultMessage> {
        Command::perform(
            get_onchain_txs(revaultd, self.vault.outpoint()),
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
    pub fn title(&self, vault: &model::Vault) -> &'static str {
        match self {
            Self::Unloaded => "",
            Self::OnchainTransactions { .. } => match vault.status {
                VaultStatus::Funded | VaultStatus::Unconfirmed => "Deposit details",
                _ => "Vault details",
            },
            Self::Revault { .. } => "Revault funds",
        }
    }

    pub fn new_onchain_txs_section(txs: VaultTransactions) -> Self {
        Self::OnchainTransactions {
            txs,
            view: VaultOnChainTransactionsPanel::new(),
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

    fn update<C: Client + Send + Sync + 'static>(
        &mut self,
        revaultd: Arc<RevaultD<C>>,
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
            _ => {}
        };
        Command::none()
    }

    pub fn view<C: Client>(&mut self, ctx: &Context<C>, vault: &model::Vault) -> Element<Message> {
        match self {
            Self::Unloaded => iced::Container::new(iced::Column::new()).into(),
            Self::OnchainTransactions { txs, view } => view.view(ctx, &vault, &txs),
            Self::Revault {
                processing,
                success,
                warning,
                view,
            } => view.view(ctx, vault, &processing, &success, warning.as_ref()),
        }
    }
}
