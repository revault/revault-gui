use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, list_vaults_with_transactions},
    State,
};

use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::Message,
    view::{vault::VaultView, Context, HistoryView},
};

#[derive(Debug)]
pub struct HistoryState {
    revaultd: Arc<RevaultD>,
    view: HistoryView,

    blockheight: u64,
    warning: Option<Error>,

    vaults: Vec<HistoryVault>,
    selected_vault: Option<HistoryVault>,
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

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = vaults
            .into_iter()
            .map(|(vlt, txs)| HistoryVault::new(vlt, txs))
            .collect();
    }

    pub fn on_vault_selected(&mut self, outpoint: String) -> Command<Message> {
        if let Some(vlt) = &self.selected_vault {
            if vlt.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return Command::none();
            }
        }

        if let Some(i) = self
            .vaults
            .iter()
            .position(|vlt| vlt.vault.outpoint() == outpoint)
        {
            self.selected_vault = Some(HistoryVault::new_selected(
                self.vaults[i].vault.clone(),
                self.vaults[i].txs.clone(),
            ));
        }
        return Command::none();
    }
}

impl State for HistoryState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectVault(outpoint) => return self.on_vault_selected(outpoint),
            Message::VaultsWithTransactions(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
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
            Command::perform(
                list_vaults_with_transactions(self.revaultd.clone()),
                Message::VaultsWithTransactions,
            ),
        ])
    }
}

impl From<HistoryState> for Box<dyn State> {
    fn from(s: HistoryState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct HistoryVault {
    vault: Vault,
    txs: VaultTransactions,
    view: VaultView,
}

impl HistoryVault {
    pub fn new(vault: Vault, txs: VaultTransactions) -> Self {
        Self {
            vault,
            txs,
            view: VaultView::new(),
        }
    }
    pub fn new_selected(vault: Vault, txs: VaultTransactions) -> Self {
        Self {
            vault,
            txs,
            view: VaultView::new_modal(),
        }
    }
    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault, &self.txs)
    }
}
