use std::convert::From;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use iced::{time, Command, Element, Subscription};

use super::{util::Watch, State};
use crate::revaultd::{
    model::{Vault, VaultStatus, VaultTransactions},
    RevaultD, RevaultDError,
};
use crate::ui::{
    error::Error,
    message::Message,
    view::manager::{ManagerHistoryView, ManagerHomeView},
};

#[derive(Debug)]
pub struct ManagerHomeState {
    revaultd: Arc<RevaultD>,
    view: ManagerHomeView,

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    vaults: Vec<Rc<Vault>>,
    selected_vault: Option<(Rc<Vault>, VaultTransactions)>,
}

impl ManagerHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerHomeState {
            revaultd,
            view: ManagerHomeView::new(),
            blockheight: Watch::None,
            vaults: Vec::new(),
            warning: Watch::None,
            selected_vault: None,
        }
    }

    pub fn reload_view(&mut self) {
        let balance = self.balance();
        self.view.load(
            self.vaults.clone(),
            self.selected_vault.clone(),
            balance,
            self.blockheight.clone().into(),
            self.warning.clone().into(),
        );
    }

    pub fn update_vaults(&mut self, vaults: Vec<Vault>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(Rc::new(vlt));
        }

        self.reload_view();
    }

    pub fn update_vault_selected(&mut self, txs: Vec<VaultTransactions>) {
        for vlt in &self.vaults {
            for tx in &txs {
                if vlt.outpoint() == tx.outpoint {
                    self.selected_vault = Some((vlt.clone(), tx.clone()));
                    break;
                }
            }
        }
        self.reload_view();
    }

    pub fn on_vault_selected(&mut self, outpoint: String) -> Command<Message> {
        if let Some((vlt, _)) = &self.selected_vault {
            if vlt.outpoint() == outpoint {
                self.selected_vault = None;
                self.reload_view();
                return Command::none();
            }
        }
        Command::perform(
            get_vaults_txs(self.revaultd.clone(), vec![outpoint]),
            Message::VaultTransactions,
        )
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

    pub fn balance(&self) -> u64 {
        let mut amt: u64 = 0;
        for vlt in &self.vaults {
            if vlt.status == VaultStatus::Active
                || vlt.status == VaultStatus::Secured
                || vlt.status == VaultStatus::Funded
                || vlt.status == VaultStatus::Unconfirmed
            {
                amt += vlt.amount
            }
        }
        amt
    }
}

impl State for ManagerHomeState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(_) => return self.on_tick(),
            Message::SelectVault(outpoint) => return self.on_vault_selected(outpoint),
            Message::VaultTransactions(res) => match res {
                Ok(txs) => self.update_vault_selected(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::BlockHeight(b) => match b {
                Ok(height) => {
                    self.blockheight = height.into();
                    self.reload_view();
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                    self.reload_view();
                }
            },
            _ => {}
        };
        Command::none()
    }

    fn view<'a>(&'a mut self) -> Element<Message> {
        self.view.view()
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

impl From<ManagerHomeState> for Box<dyn State> {
    fn from(s: ManagerHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct ManagerHistoryState {
    revaultd: Arc<RevaultD>,
    view: ManagerHistoryView,

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    vaults: Vec<Rc<Vault>>,
    selected_vault: Option<(Rc<Vault>, VaultTransactions)>,
}

impl ManagerHistoryState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerHistoryState {
            revaultd,
            view: ManagerHistoryView::new(),
            blockheight: Watch::None,
            vaults: Vec::new(),
            warning: Watch::None,
            selected_vault: None,
        }
    }

    pub fn reload_view(&mut self) {
        self.view.load(
            self.vaults.clone(),
            self.selected_vault.clone(),
            self.warning.clone().into(),
        );
    }

    pub fn update_vaults(&mut self, vaults: Vec<Vault>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(Rc::new(vlt));
        }

        self.reload_view();
    }

    pub fn update_vault_selected(&mut self, txs: Vec<VaultTransactions>) {
        for vlt in &self.vaults {
            for tx in &txs {
                if vlt.outpoint() == tx.outpoint {
                    self.selected_vault = Some((vlt.clone(), tx.clone()));
                    break;
                }
            }
        }
        self.reload_view();
    }

    pub fn on_vault_selected(&mut self, outpoint: String) -> Command<Message> {
        if let Some((vlt, _)) = &self.selected_vault {
            if vlt.outpoint() == outpoint {
                self.selected_vault = None;
                self.reload_view();
                return Command::none();
            }
        }
        Command::perform(
            get_vaults_txs(self.revaultd.clone(), vec![outpoint]),
            Message::VaultTransactions,
        )
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

impl State for ManagerHistoryState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(_) => return self.on_tick(),
            Message::SelectVault(outpoint) => return self.on_vault_selected(outpoint),
            Message::VaultTransactions(res) => match res {
                Ok(txs) => self.update_vault_selected(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
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

    fn view<'a>(&'a mut self) -> Element<Message> {
        self.view.view()
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

impl From<ManagerHistoryState> for Box<dyn State> {
    fn from(s: ManagerHistoryState) -> Box<dyn State> {
        Box::new(s)
    }
}

async fn get_vaults_txs(
    revaultd: Arc<RevaultD>,
    outpoints: Vec<String>,
) -> Result<Vec<VaultTransactions>, RevaultDError> {
    revaultd
        .list_transactions(Some(outpoints))
        .map(|res| res.transactions)
}

async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

async fn list_vaults(revaultd: Arc<RevaultD>) -> Result<Vec<Vault>, RevaultDError> {
    revaultd.list_vaults().map(|res| res.vaults)
}
