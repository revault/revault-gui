use std::collections::HashMap;

use iced::{Command, Element, Subscription};

use crate::daemon::model::{self, VaultStatus};

use crate::app::{
    context::Context,
    error::Error,
    message::{Message, VaultMessage},
    state::{
        cmd::{get_blockheight, get_deposit_address, get_revocation_txs, list_vaults},
        vault::{Vault, VaultListItem},
        State,
    },
    view::{
        vault::{DelegateVaultListItemView, SecureVaultListItemView, VaultListItemView},
        StakeholderCreateVaultsView, StakeholderDelegateFundsView, StakeholderHomeView,
    },
};

#[derive(Debug)]
pub struct StakeholderHomeState {
    warning: Option<Error>,

    balance: HashMap<VaultStatus, (u64, u64)>,

    moving_vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,

    view: StakeholderHomeView,
}

impl StakeholderHomeState {
    pub fn new() -> Self {
        StakeholderHomeState {
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

    pub fn on_vault_select(&mut self, ctx: &Context, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return self.load(ctx);
            }
        }

        if let Some(selected) = self
            .moving_vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.load(ctx.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd.map(Message::Vault);
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
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SelectVault(outpoint) => return self.on_vault_select(ctx, outpoint),
            Message::Vault(msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    return selected.update(ctx, msg).map(Message::Vault);
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

    fn subscription(&self) -> Subscription<Message> {
        if let Some(v) = &self.selected_vault {
            return v.subscription().map(Message::Vault);
        }
        Subscription::none()
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(ctx.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                list_vaults(
                    ctx.revaultd.clone(),
                    Some(&VaultStatus::DEPOSIT_AND_CURRENT),
                    None,
                ),
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
pub struct StakeholderCreateVaultsState {
    warning: Option<Error>,
    balance: u64,
    address: Option<bitcoin::Address>,
    deposits: Vec<VaultListItem<SecureVaultListItemView>>,
    selected_vault: Option<Vault>,

    view: StakeholderCreateVaultsView,
}

impl StakeholderCreateVaultsState {
    pub fn new() -> Self {
        StakeholderCreateVaultsState {
            address: None,
            warning: None,
            deposits: Vec::new(),
            view: StakeholderCreateVaultsView::new(),
            balance: 0,
            selected_vault: None,
        }
    }

    pub fn on_vault_select(&mut self, ctx: &Context, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return self.load(ctx);
            }
        }

        if let Some(selected) = self
            .deposits
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            self.selected_vault = Some(Vault::new(selected.vault.clone()));
            return Command::perform(
                get_revocation_txs(ctx.revaultd.clone(), selected.vault.outpoint()),
                move |res| Message::Vault(VaultMessage::RevocationTransactions(res)),
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
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::DepositAddress(res) => match res {
                Ok(address) => {
                    // Address is loaded directly in the view in order to cache the created qrcode.
                    self.view.load(&address);
                    self.address = Some(address);
                    Command::none()
                }
                Err(e) => {
                    self.warning = Some(Error::RevaultDError(e));
                    Command::none()
                }
            },
            Message::SelectVault(outpoint) => self.on_vault_select(ctx, outpoint),
            Message::Vault(msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    return selected.update(ctx, msg).map(Message::Vault);
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

    fn subscription(&self) -> Subscription<Message> {
        if let Some(v) = &self.selected_vault {
            return v.subscription().map(Message::Vault);
        }
        Subscription::none()
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

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![
            Command::perform(
                get_deposit_address(ctx.revaultd.clone()),
                Message::DepositAddress,
            ),
            Command::perform(
                list_vaults(ctx.revaultd.clone(), Some(&[VaultStatus::Funded]), None),
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
    active_balance: u64,
    activating_balance: u64,
    vaults: Vec<VaultListItem<DelegateVaultListItemView>>,
    selected_vault: Option<Vault>,
    warning: Option<Error>,

    view: StakeholderDelegateFundsView,
}

impl StakeholderDelegateFundsState {
    pub fn new() -> Self {
        StakeholderDelegateFundsState {
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
        self.vaults = vaults.into_iter().map(VaultListItem::new).collect();
    }

    pub fn on_vault_select(&mut self, ctx: &Context, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return self.load(ctx);
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.load(ctx.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd.map(Message::Vault);
        };
        Command::none()
    }

    pub fn on_vault_delegate(&mut self, ctx: &Context, outpoint: String) -> Command<Message> {
        if let Some(selected) = &mut self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                return selected
                    .update(ctx, VaultMessage::SelectDelegate)
                    .map(Message::Vault);
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let mut selected_vault = Vault::new(selected.vault.clone());
            let cmd = selected_vault.update(ctx, VaultMessage::SelectDelegate);
            self.selected_vault = Some(selected_vault);
            return cmd.map(Message::Vault);
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
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SelectVault(outpoint) => return self.on_vault_select(ctx, outpoint),
            Message::DelegateVault(outpoint) => return self.on_vault_delegate(ctx, outpoint),
            Message::Vault(msg) => match msg {
                _ => {
                    if let Some(selected) = &mut self.selected_vault {
                        return selected.update(ctx, msg).map(Message::Vault);
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
            self.warning.as_ref(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(v) = &self.selected_vault {
            return v.subscription().map(Message::Vault);
        }
        Subscription::none()
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::perform(
            list_vaults(
                ctx.revaultd.clone(),
                Some(&[
                    VaultStatus::Secured,
                    VaultStatus::Activating,
                    VaultStatus::Active,
                ]),
                None,
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
