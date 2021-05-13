use std::collections::HashMap;
use std::sync::Arc;

use iced::{Command, Element};

use crate::revaultd::{
    model::{self, VaultStatus},
    RevaultD,
};

use crate::app::{
    error::Error,
    message::{Message, VaultMessage},
    state::{
        cmd::{get_blockheight, get_deposit_address, get_revocation_txs, list_vaults},
        vault::{Vault, VaultListItem},
        State,
    },
    view::{
        vault::{DelegateVaultListItemView, SecureVaultListItemView, VaultListItemView},
        Context, StakeholderCreateVaultsView, StakeholderDelegateFundsView, StakeholderHomeView,
        StakeholderNetworkView,
    },
};

#[derive(Debug)]
pub struct StakeholderHomeState {
    revaultd: Arc<RevaultD>,
    warning: Option<Error>,

    balance: HashMap<VaultStatus, (u64, u64)>,

    moving_vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,

    view: StakeholderHomeView,
}

impl StakeholderHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderHomeState {
            revaultd,
            warning: None,
            view: StakeholderHomeView::new(),
            balance: HashMap::new(),
            moving_vaults: Vec::new(),
            selected_vault: None,
        }
    }

    fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.calculate_balance(&vaults);
        self.moving_vaults = vaults
            .into_iter()
            .filter_map(|vlt| {
                if vlt.status == VaultStatus::Canceling
                    || vlt.status == VaultStatus::Spending
                    || vlt.status == VaultStatus::Unvaulting
                    || vlt.status == VaultStatus::Unvaulted
                {
                    Some(VaultListItem::new(vlt))
                } else {
                    None
                }
            })
            .collect();
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return Command::none();
            }
        }

        if let Some(selected) = self
            .moving_vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.load(self.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd.map(move |msg| Message::Vault(outpoint.clone(), msg));
        };
        Command::none()
    }

    fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        let mut balance = HashMap::new();
        for vault in vaults {
            if vault.status == VaultStatus::Unconfirmed
                || VaultStatus::MOVING.contains(&vault.status)
            {
                continue;
            }
            if let Some((number, amount)) = balance.get_mut(&vault.status) {
                *number += 1;
                *amount += vault.amount;
            } else {
                balance.insert(vault.status.clone(), (1, vault.amount));
            }
        }

        self.balance = balance;
    }
}

impl State for StakeholderHomeState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vault(outpoint, VaultMessage::Select) => {
                return self.on_vault_select(outpoint)
            }
            Message::Vault(outpoint, msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    if selected.vault.outpoint() == outpoint {
                        return selected
                            .update(self.revaultd.clone(), msg)
                            .map(move |msg| Message::Vault(outpoint.clone(), msg));
                    }
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }

        self.view.view(
            ctx,
            None,
            self.moving_vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            &self.balance,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                list_vaults(self.revaultd.clone(), Some(&VaultStatus::CURRENT)),
                Message::Vaults,
            ),
        ])
    }
}

impl From<StakeholderHomeState> for Box<dyn State> {
    fn from(s: StakeholderHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct StakeholderNetworkState {
    revaultd: Arc<RevaultD>,

    blockheight: Option<u64>,
    warning: Option<Error>,

    view: StakeholderNetworkView,
}

impl StakeholderNetworkState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderNetworkState {
            revaultd,
            blockheight: None,
            warning: None,
            view: StakeholderNetworkView::new(),
        }
    }
}

impl State for StakeholderNetworkState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
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

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            get_blockheight(self.revaultd.clone()),
            Message::BlockHeight,
        )])
    }
}

impl From<StakeholderNetworkState> for Box<dyn State> {
    fn from(s: StakeholderNetworkState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct StakeholderCreateVaultsState {
    revaultd: Arc<RevaultD>,

    warning: Option<Error>,
    balance: u64,
    address: Option<bitcoin::Address>,
    deposits: Vec<VaultListItem<SecureVaultListItemView>>,
    selected_vault: Option<Vault>,

    view: StakeholderCreateVaultsView,
}

impl StakeholderCreateVaultsState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderCreateVaultsState {
            revaultd,
            address: None,
            warning: None,
            deposits: Vec::new(),
            view: StakeholderCreateVaultsView::new(),
            balance: 0,
            selected_vault: None,
        }
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return self.load();
            }
        }

        if let Some(selected) = self
            .deposits
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            self.selected_vault = Some(Vault::new(selected.vault.clone()));
            return Command::perform(
                get_revocation_txs(self.revaultd.clone(), selected.vault.outpoint()),
                move |res| {
                    Message::Vault(outpoint.clone(), VaultMessage::RevocationTransactions(res))
                },
            );
        };
        Command::none()
    }

    fn update_deposits(&mut self, vaults: Vec<model::Vault>) {
        self.calculate_balance(&vaults);
        self.deposits = vaults.into_iter().map(VaultListItem::new).collect();
    }

    fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        let mut balance: u64 = 0;
        for vault in vaults {
            if vault.status == VaultStatus::Funded {
                balance += vault.amount;
            }
        }

        self.balance = balance;
    }
}

impl State for StakeholderCreateVaultsState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DepositAddress(res) => match res {
                Ok(address) => {
                    // Address is loaded directly in the view in order to cache the created qrcode.
                    self.view.load(&address);
                    self.address = Some(address);
                    return Command::none();
                }
                Err(e) => {
                    self.warning = Some(Error::RevaultDError(e));
                    return Command::none();
                }
            },
            Message::Vault(outpoint, VaultMessage::Select) => self.on_vault_select(outpoint),
            Message::Vault(outpoint, msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    if selected.vault.outpoint() == outpoint {
                        return selected
                            .update(self.revaultd.clone(), msg)
                            .map(move |msg| Message::Vault(outpoint.clone(), msg));
                    }
                }
                Command::none()
            }
            Message::Vaults(res) => match res {
                Ok(vaults) => {
                    self.update_deposits(vaults);
                    Command::none()
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                    Command::none()
                }
            },
            _ => Command::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        if let Some(selected) = &mut self.selected_vault {
            return selected.view(ctx);
        }
        self.view.view(
            ctx,
            self.deposits.iter_mut().map(|v| v.view(ctx)).collect(),
            self.address.as_ref(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(
                get_deposit_address(self.revaultd.clone()),
                Message::DepositAddress,
            ),
            Command::perform(
                list_vaults(self.revaultd.clone(), Some(&[VaultStatus::Funded])),
                Message::Vaults,
            ),
        ])
    }
}

impl From<StakeholderCreateVaultsState> for Box<dyn State> {
    fn from(s: StakeholderCreateVaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct StakeholderDelegateFundsState {
    revaultd: Arc<RevaultD>,

    active_balance: u64,
    activating_balance: u64,
    vaults: Vec<VaultListItem<DelegateVaultListItemView>>,
    selected_vault: Option<Vault>,
    warning: Option<Error>,

    view: StakeholderDelegateFundsView,
}

impl StakeholderDelegateFundsState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderDelegateFundsState {
            revaultd,
            active_balance: 0,
            activating_balance: 0,
            vaults: Vec::new(),
            selected_vault: None,
            warning: None,
            view: StakeholderDelegateFundsView::new(),
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.calculate_balance(&vaults);
        self.vaults = vaults
            .into_iter()
            .map(|vlt| VaultListItem::new(vlt))
            .collect();
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return self.load();
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.load(self.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd.map(move |msg| Message::Vault(outpoint.clone(), msg));
        };
        Command::none()
    }

    pub fn on_vault_delegate(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &mut self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                return selected
                    .update(self.revaultd.clone(), VaultMessage::Delegate)
                    .map(move |msg| Message::Vault(outpoint.clone(), msg));
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let mut selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.update(self.revaultd.clone(), VaultMessage::Delegate);
            self.selected_vault = Some(selected_vault);
            return cmd.map(move |msg| Message::Vault(outpoint.clone(), msg));
        };
        Command::none()
    }

    pub fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        let (active_balance, activating_balance) = vaults.iter().fold((0, 0), |acc, vault| {
            if vault.status == VaultStatus::Active {
                (acc.0 + vault.amount, acc.1)
            } else if vault.status == VaultStatus::Activating {
                (acc.0, acc.1 + vault.amount)
            } else {
                acc
            }
        });

        self.active_balance = active_balance;
        self.activating_balance = activating_balance;
    }
}

impl State for StakeholderDelegateFundsState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vault(outpoint, msg) => match msg {
                VaultMessage::Select => return self.on_vault_select(outpoint),
                VaultMessage::Delegate => return self.on_vault_delegate(outpoint),
                _ => {
                    if let Some(selected) = &mut self.selected_vault {
                        if selected.vault.outpoint() == outpoint {
                            return selected
                                .update(self.revaultd.clone(), msg)
                                .map(move |msg| Message::Vault(outpoint.clone(), msg));
                        }
                    }
                    return Command::none();
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
            &self.active_balance,
            &self.activating_balance,
            self.vaults
                .iter_mut()
                .filter(|v| v.vault.status == VaultStatus::Secured)
                .map(|v| v.view(ctx))
                .collect(),
            self.warning.as_ref().into(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::perform(
            list_vaults(
                self.revaultd.clone(),
                Some(&[
                    VaultStatus::Secured,
                    VaultStatus::Activating,
                    VaultStatus::Active,
                ]),
            ),
            Message::Vaults,
        )
    }
}

impl From<StakeholderDelegateFundsState> for Box<dyn State> {
    fn from(s: StakeholderDelegateFundsState) -> Box<dyn State> {
        Box::new(s)
    }
}
