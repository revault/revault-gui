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
    message::{ManagerSendOutputMessage, Message},
    view::manager::{ManagerHistoryView, ManagerHomeView, ManagerSendOutputView, ManagerSendView},
};

#[derive(Debug)]
pub struct ManagerHomeState {
    revaultd: Arc<RevaultD>,
    view: ManagerHomeView,

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    vaults: Vec<Rc<(Vault, VaultTransactions)>>,
    selected_vault: Option<Rc<(Vault, VaultTransactions)>>,
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

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(Rc::new(vlt));
        }

        self.reload_view();
    }

    pub fn on_vault_selected(&mut self, outpoint: String) -> Command<Message> {
        if let Some(vlt) = &self.selected_vault {
            if vlt.0.outpoint() == outpoint {
                self.selected_vault = None;
                self.reload_view();
                return Command::none();
            }
        }

        if let Some(i) = self
            .vaults
            .iter()
            .position(|vlt| vlt.0.outpoint() == outpoint)
        {
            self.selected_vault = Some(self.vaults[i].clone());
            self.reload_view();
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

    pub fn balance(&self) -> u64 {
        let mut amt: u64 = 0;
        for vlt in &self.vaults {
            if vlt.0.status == VaultStatus::Active
                || vlt.0.status == VaultStatus::Secured
                || vlt.0.status == VaultStatus::Funded
                || vlt.0.status == VaultStatus::Unconfirmed
            {
                amt += vlt.0.amount
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

    fn view(&mut self) -> Element<Message> {
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

    vaults: Vec<Rc<(Vault, VaultTransactions)>>,
    selected_vault: Option<Rc<(Vault, VaultTransactions)>>,
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

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(Rc::new(vlt));
        }

        self.reload_view();
    }

    pub fn on_vault_selected(&mut self, outpoint: String) -> Command<Message> {
        if let Some(vlt) = &self.selected_vault {
            if vlt.0.outpoint() == outpoint {
                self.selected_vault = None;
                self.reload_view();
                return Command::none();
            }
        }

        if let Some(i) = self
            .vaults
            .iter()
            .position(|vlt| vlt.0.outpoint() == outpoint)
        {
            self.selected_vault = Some(self.vaults[i].clone());
            self.reload_view();
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

impl State for ManagerHistoryState {
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

    fn view(&mut self) -> Element<Message> {
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

#[derive(Debug)]
pub struct ManagerSendState {
    revaultd: Arc<RevaultD>,
    view: ManagerSendView,

    warning: Option<Error>,

    vaults: Vec<Rc<(Vault, VaultTransactions)>>,
    outputs: Vec<ManagerSendOutput>,
}

impl ManagerSendState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerSendState {
            revaultd,
            view: ManagerSendView::new(),
            warning: None,
            vaults: Vec::new(),
            outputs: vec![ManagerSendOutput::new()],
        }
    }
    pub fn reload_view(&mut self) {
        match &mut self.view {
            ManagerSendView::SelectInputs(v) => {
                v.load(self.vaults.clone(), self.warning.clone().into())
            }
            _ => {}
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = Vec::new();
        for vlt in vaults {
            self.vaults.push(Rc::new(vlt));
        }

        self.reload_view();
    }
}

impl State for ManagerSendState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ManagerSendOutput(i, msg) => {
                if let Some(output) = self.outputs.get_mut(i) {
                    output.update(msg);
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        match &mut self.view {
            ManagerSendView::SelectOutputs(v) => v.view(
                self.outputs
                    .iter_mut()
                    .enumerate()
                    .map(|(i, v)| v.view().map(move |msg| Message::ManagerSendOutput(i, msg)))
                    .collect(),
            ),
            ManagerSendView::SelectInputs(v) => v.view(),
        }
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(self.revaultd.clone()),
            Message::Vaults,
        )])
    }
}

impl From<ManagerSendState> for Box<dyn State> {
    fn from(s: ManagerSendState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
struct ManagerSendOutput {
    address: String,
    amount: u64,

    view: ManagerSendOutputView,
}

impl ManagerSendOutput {
    fn new() -> Self {
        Self {
            address: "".to_string(),
            amount: 0,
            view: ManagerSendOutputView::new_edit(),
        }
    }

    fn update(&mut self, message: ManagerSendOutputMessage) {
        match message {
            ManagerSendOutputMessage::AddressEdited(address) => self.address = address,
        };
    }

    fn view(&mut self) -> Element<ManagerSendOutputMessage> {
        self.view.view(&self.address)
    }
}

async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

async fn list_vaults(
    revaultd: Arc<RevaultD>,
) -> Result<Vec<(Vault, VaultTransactions)>, RevaultDError> {
    let vaults = revaultd.list_vaults().map(|res| res.vaults)?;
    let outpoints = vaults.iter().map(|vlt| vlt.outpoint()).collect();
    let txs = revaultd.list_transactions(Some(outpoints))?;

    let mut vec = Vec::new();
    for vlt in vaults {
        if let Some(i) = txs
            .transactions
            .iter()
            .position(|tx| tx.outpoint == vlt.outpoint())
        {
            vec.push((vlt, txs.transactions[i].to_owned()));
        }
    }
    Ok(vec)
}
