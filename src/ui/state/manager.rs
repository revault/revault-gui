use std::convert::From;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use iced::{time, Command, Element, Subscription};

use super::{
    cmd::{get_blockheight, list_vaults},
    util::Watch,
    State,
};

use crate::revaultd::{
    model::{Vault, VaultStatus, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::{InputMessage, Message, RecipientMessage},
    view::manager::{manager_send_input_view, ManagerSendOutputView, ManagerSendView},
    view::vault::VaultView,
    view::Context,
    view::{ManagerHomeView, ManagerNetworkView},
};

#[derive(Debug)]
pub struct ManagerHomeState {
    revaultd: Arc<RevaultD>,
    view: ManagerHomeView,

    balance: u64,
    blockheight: Watch<u64>,
    warning: Watch<Error>,

    vaults: Vec<ManagerVault>,
    selected_vault: Option<ManagerVault>,
}

impl ManagerHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerHomeState {
            revaultd,
            balance: 0,
            view: ManagerHomeView::new(),
            blockheight: Watch::None,
            vaults: Vec::new(),
            warning: Watch::None,
            selected_vault: None,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = vaults
            .into_iter()
            .map(|(vlt, txs)| ManagerVault::new(vlt, txs))
            .collect();
        self.calculate_balance();
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
            self.selected_vault = Some(ManagerVault::new_selected(
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

    pub fn calculate_balance(&mut self) {
        let mut amt: u64 = 0;
        for vlt in &self.vaults {
            if vlt.vault.status == VaultStatus::Active
                || vlt.vault.status == VaultStatus::Secured
                || vlt.vault.status == VaultStatus::Funded
                || vlt.vault.status == VaultStatus::Unconfirmed
            {
                amt += vlt.vault.amount
            }
        }
        self.balance = amt;
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
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                }
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
            &self.balance,
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

impl From<ManagerHomeState> for Box<dyn State> {
    fn from(s: ManagerHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct ManagerVault {
    vault: Vault,
    txs: VaultTransactions,
    view: VaultView,
}

impl ManagerVault {
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

#[derive(Debug)]
pub struct ManagerSendState {
    revaultd: Arc<RevaultD>,
    view: ManagerSendView,

    warning: Option<Error>,

    vaults: Vec<ManagerSendInput>,
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

    pub fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.vaults = vaults
            .into_iter()
            .map(|(vlt, txs)| ManagerSendInput::new(vlt, txs))
            .collect();
    }

    pub fn input_amount(&self) -> u64 {
        let mut input_amount = 0;
        for input in &self.vaults {
            if input.selected {
                input_amount += input.vault.amount;
            }
        }
        input_amount
    }

    pub fn output_amount(&self) -> u64 {
        let mut output_amount = 0;
        for output in &self.outputs {
            if let Ok(amount) = output.amount() {
                output_amount += amount;
            }
        }
        output_amount
    }
}

impl State for ManagerSendState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vlts) => self.update_vaults(vlts),
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            Message::Next => self.view = self.view.next(),
            Message::Previous => self.view = self.view.previous(),
            Message::AddRecipient => self.outputs.push(ManagerSendOutput::new()),
            Message::Recipient(i, RecipientMessage::Delete) => {
                self.outputs.remove(i);
            }
            Message::Input(i, msg) => {
                if let Some(input) = self.vaults.get_mut(i) {
                    input.update(msg);
                }
            }
            Message::Recipient(i, msg) => {
                if let Some(output) = self.outputs.get_mut(i) {
                    output.update(msg);
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, _ctx: &Context) -> Element<Message> {
        let input_amount = self.input_amount();
        let output_amount = self.output_amount();
        match &mut self.view {
            ManagerSendView::SelectOutputs(v) => {
                let valid = !self.outputs.iter().any(|o| !o.valid());
                v.view(
                    self.outputs
                        .iter_mut()
                        .enumerate()
                        .map(|(i, v)| v.view().map(move |msg| Message::Recipient(i, msg)))
                        .collect(),
                    valid,
                )
            }
            ManagerSendView::SelectInputs(v) => v.view(
                self.vaults
                    .iter_mut()
                    .enumerate()
                    .map(|(i, v)| v.view().map(move |msg| Message::Input(i, msg)))
                    .collect(),
                input_amount > output_amount,
            ),
            ManagerSendView::SelectFee(v) => v.view(false),
            ManagerSendView::Sign(v) => v.view(false),
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
    amount: String,

    warning_address: bool,
    warning_amount: bool,

    view: ManagerSendOutputView,
}

impl ManagerSendOutput {
    fn new() -> Self {
        Self {
            address: "".to_string(),
            amount: "".to_string(),
            warning_address: false,
            warning_amount: false,
            view: ManagerSendOutputView::new(),
        }
    }

    fn amount(&self) -> Result<u64, Error> {
        if self.amount.is_empty() {
            return Ok(0);
        }
        let a = f64::from_str(&self.amount)
            .map_err(|_| Error::UnexpectedError("cannot parse output amount".to_string()))?;
        Ok((a * 100000000_f64) as u64)
    }

    fn valid(&self) -> bool {
        !self.address.is_empty()
            && !self.warning_address
            && !self.amount.is_empty()
            && !self.warning_amount
    }

    fn update(&mut self, message: RecipientMessage) {
        match message {
            RecipientMessage::AddressEdited(address) => {
                self.address = address;
                if !self.address.is_empty() {
                    self.warning_address = bitcoin::Address::from_str(&self.address).is_err();
                }
            }
            RecipientMessage::AmountEdited(amount) => {
                self.amount = amount;
                if !self.amount.is_empty() {
                    self.warning_amount = f64::from_str(&self.amount).is_err();
                }
            }
            _ => {}
        };
    }

    fn view(&mut self) -> Element<RecipientMessage> {
        self.view.view(
            &self.address,
            &self.amount,
            &self.warning_address,
            &self.warning_amount,
        )
    }
}

#[derive(Debug)]
struct ManagerSendInput {
    vault: Vault,
    transactions: VaultTransactions,
    selected: bool,
}

impl ManagerSendInput {
    fn new(vault: Vault, transactions: VaultTransactions) -> Self {
        Self {
            vault,
            transactions,
            selected: false,
        }
    }

    pub fn view(&mut self) -> Element<InputMessage> {
        manager_send_input_view(&self.vault.outpoint(), &self.vault.amount, self.selected)
    }

    pub fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::Selected(selected) => self.selected = selected,
        }
    }
}

#[derive(Debug)]
pub struct ManagerNetworkState {
    revaultd: Arc<RevaultD>,

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    view: ManagerNetworkView,
}

impl ManagerNetworkState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerNetworkState {
            revaultd,
            blockheight: Watch::None,
            warning: Watch::None,
            view: ManagerNetworkView::new(),
        }
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

impl State for ManagerNetworkState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tick(_) => self.on_tick(),
            Message::BlockHeight(b) => {
                match b {
                    Ok(height) => {
                        self.blockheight = height.into();
                    }
                    Err(e) => {
                        self.warning = Error::from(e).into();
                    }
                };
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            self.warning.as_ref().into(),
            self.blockheight.as_ref().into(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_secs(1)).map(Message::Tick)
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_blockheight(self.revaultd.clone()),
            Message::BlockHeight,
        )])
    }
}

impl From<ManagerNetworkState> for Box<dyn State> {
    fn from(s: ManagerNetworkState) -> Box<dyn State> {
        Box::new(s)
    }
}
