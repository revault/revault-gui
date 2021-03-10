use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, get_onchain_txs, list_vaults},
    vault::{SelectedVault, VaultListItem},
    State,
};

use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::Message,
    view::{Context, HistoryView},
};

#[derive(Debug)]
pub struct HistoryState {
    revaultd: Arc<RevaultD>,
    view: HistoryView,

    blockheight: u64,
    warning: Option<Error>,

    vaults: Vec<VaultListItem>,
    selected_vault: Option<SelectedVault>,
}

impl HistoryState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        HistoryState {
            revaultd,
            view: HistoryView::new(),
            blockheight: 0,
            vaults: Vec::new(),
            warning: None,
            selected_vault: None,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<Vault>) {
        self.vaults = vaults
            .into_iter()
            .map(|vlt| VaultListItem::new(vlt))
            .collect();
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return Command::none();
            }
        }

        return Command::perform(
            get_onchain_txs(self.revaultd.clone(), outpoint),
            Message::VaultOnChainTransactions,
        );
    }

    pub fn update_selected_vault(&mut self, vault_txs: VaultTransactions) {
        if let Some(i) = self
            .vaults
            .iter()
            .position(|vlt| vlt.vault.outpoint() == vault_txs.vault_outpoint)
        {
            self.selected_vault = Some(SelectedVault::new(self.vaults[i].vault.clone(), vault_txs));
        };
    }
}

impl State for HistoryState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectVault(outpoint) => return self.on_vault_select(outpoint),
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::VaultOnChainTransactions(res) => match res {
                Ok(txs) => self.update_selected_vault(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::BlockHeight(b) => match b {
                Ok(height) => self.blockheight = height.into(),
                Err(e) => self.warning = Error::from(e).into(),
            },
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }
        self.view.view(
            ctx,
            self.warning.as_ref().into(),
            self.vaults.iter_mut().map(|v| v.view(ctx)).collect(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(list_vaults(self.revaultd.clone()), Message::Vaults),
        ])
    }
}

impl From<HistoryState> for Box<dyn State> {
    fn from(s: HistoryState) -> Box<dyn State> {
        Box::new(s)
    }
}
