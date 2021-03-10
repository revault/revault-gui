use crate::ui::{
    message::Message,
    view::{
        vault::{VaultListItemView, VaultModal},
        Context,
    },
};
use iced::Element;

use crate::revaultd::model::{Vault, VaultTransactions};

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

#[derive(Debug)]
pub struct SelectedVault {
    pub vault: Vault,
    pub txs: VaultTransactions,
    view: VaultModal,
}

impl SelectedVault {
    pub fn new(vault: Vault, txs: VaultTransactions) -> Self {
        Self {
            vault,
            txs,
            view: VaultModal::new(),
        }
    }
    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault, &self.txs)
    }
}
