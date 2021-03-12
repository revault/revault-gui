use std::sync::Arc;

use crate::ui::{
    error::Error,
    message::{Message, VaultMessage},
    state::cmd::get_onchain_txs,
    view::{
        vault::{VaultListItemView, VaultModal, VaultOnChainTransactionsPanel},
        Context,
    },
};

use iced::{Command, Element};

use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD,
};

#[derive(Debug)]
pub struct VaultListItem {
    pub vault: Vault,
    view: VaultListItemView,
}

impl VaultListItem {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault,
            view: VaultListItemView::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault)
    }
}

/// SelectedVault is a widget displaying information of a vault
/// and handling user action on it.
#[derive(Debug)]
pub struct SelectedVault {
    pub vault: Vault,
    warning: Option<Error>,
    panel: VaultPanel,
    view: VaultModal,
}

impl SelectedVault {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault,
            panel: VaultPanel::Unloaded,
            view: VaultModal::new(),
            warning: None,
        }
    }

    pub fn update(&mut self, message: VaultMessage) -> Command<Message> {
        match message {
            VaultMessage::OnChainTransactions(res) => match res {
                Ok(txs) => self.panel = VaultPanel::new_onchain_txs_panel(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            &self.vault,
            self.warning.as_ref(),
            self.panel.view(ctx),
        )
    }

    pub fn load(&self, revaultd: Arc<RevaultD>) -> Command<Message> {
        Command::perform(
            get_onchain_txs(revaultd.clone(), self.vault.outpoint()),
            |res| Message::Vault(VaultMessage::OnChainTransactions(res)),
        )
    }
}

#[derive(Debug)]
pub enum VaultPanel {
    Unloaded,
    OnchainTransactions {
        txs: VaultTransactions,
        view: VaultOnChainTransactionsPanel,
    },
}

impl VaultPanel {
    pub fn new_onchain_txs_panel(txs: VaultTransactions) -> Self {
        Self::OnchainTransactions {
            txs,
            view: VaultOnChainTransactionsPanel::new(),
        }
    }
    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Unloaded => iced::Container::new(iced::Column::new()).into(),
            Self::OnchainTransactions { txs, view } => view.view(ctx, &txs),
        }
    }
}
