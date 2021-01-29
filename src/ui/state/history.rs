use std::convert::From;
use std::sync::Arc;
use std::time::Duration;

use iced::{time, Command, Element, Subscription};

use super::{
    cmd::{get_blockheight, list_vaults},
    util::Watch,
    State,
};

use crate::revaultd::{
    model::{Vault, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::{Context, Message},
    view::vault::VaultView,
    view::HistoryView,
};

#[derive(Debug)]
pub struct HistoryState {
    revaultd: Arc<RevaultD>,
    view: HistoryView,

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    vaults: Vec<HistoryVault>,
    selected_vault: Option<HistoryVault>,
}

impl HistoryState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        HistoryState {
            revaultd,
            view: HistoryView::new(),
            blockheight: Watch::None,
            vaults: Vec::new(),
            warning: Watch::None,
            selected_vault: None,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(HistoryVault::new(vlt.0, vlt.1));
        }
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

    pub fn on_tick(&mut self) -> Command<Message> {
        if !self.blockheight.is_recent(Duration::from_secs(5)) {
            return Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight);
        }

        if !self.warning.is_none() && !self.warning.is_recent(Duration::from_secs(30)) {
            self.warning.reset()
        }

        Command::none()
    }
}

impl State for HistoryState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(_) => return self.on_tick(),
            Message::SelectVault(outpoint) => return self.on_vault_selected(outpoint),
            Message::Vaults(res) => match res {
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

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_secs(1)).map(Message::Tick)
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
