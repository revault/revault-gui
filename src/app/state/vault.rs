use iced::{Command, Element};
use std::sync::Arc;

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, VaultMessage},
        state::cmd::get_onchain_txs,
        view::{
            vault::{VaultModal, VaultView},
            LoadingModal,
        },
    },
    daemon::{
        model::{self, outpoint, VaultTransactions},
        Daemon,
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
pub enum Vault {
    Loading {
        vault: model::Vault,
        fail: Option<Error>,
        view: LoadingModal,
    },
    Loaded {
        txs: VaultTransactions,
        vault: model::Vault,
        view: VaultModal,
    },
}

impl Vault {
    pub fn new(vault: model::Vault) -> Self {
        Self::Loading {
            vault,
            view: LoadingModal::new(),
            fail: None,
        }
    }

    pub fn inner(&self) -> &model::Vault {
        match self {
            Self::Loading { vault, .. } => vault,
            Self::Loaded { vault, .. } => vault,
        }
    }

    pub fn update(&mut self, _ctx: &Context, message: VaultMessage) -> Command<VaultMessage> {
        if let Self::Loading { fail, vault, .. } = self {
            if let VaultMessage::OnChainTransactions(res) = message {
                match res {
                    Ok(txs) => {
                        *self = Self::Loaded {
                            vault: vault.clone(),
                            txs,
                            view: VaultModal::new(),
                        }
                    }
                    Err(e) => *fail = Some(e.into()),
                }
            }
        }
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { view, fail, .. } => view.view(ctx, fail.as_ref(), Message::Close),
            Self::Loaded { view, vault, txs } => view.view(ctx, vault, &txs),
        }
    }

    pub fn load(&self, revaultd: Arc<dyn Daemon + Send + Sync>) -> Command<VaultMessage> {
        if let Self::Loading { vault, .. } = self {
            Command::perform(
                get_onchain_txs(revaultd, outpoint(&vault)),
                VaultMessage::OnChainTransactions,
            )
        } else {
            Command::none()
        }
    }
}
