use std::sync::Arc;

use iced::{Command, Element};

use crate::revaultd::{
    model::{self, VaultStatus},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::{Message, VaultFilterMessage, VaultMessage},
    state::{
        cmd::{get_blockheight, get_revocation_txs, list_vaults},
        vault::{Vault, VaultListItem},
        State,
    },
    view::{
        vault::{AcknowledgeVaultListItemView, DelegateVaultListItemView, VaultListItemView},
        Context, StakeholderACKFundsView, StakeholderDelegateFundsView, StakeholderHomeView,
        StakeholderNetworkView,
    },
};

#[derive(Debug)]
pub struct StakeholderHomeState {
    revaultd: Arc<RevaultD>,
    warning: Option<Error>,

    /// funds without presigned revocation transactions.
    unsecured_fund_balance: u64,
    /// balance as active and inactive tuple.
    balance: (u64, u64),
    view: StakeholderHomeView,

    vaults: Vec<VaultListItem<VaultListItemView>>,
}

impl StakeholderHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderHomeState {
            revaultd,
            warning: None,
            view: StakeholderHomeView::new(),
            unsecured_fund_balance: 0,
            balance: (0, 0),
            vaults: Vec::new(),
        }
    }

    fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.calculate_balance(&vaults);
        self.vaults = vaults
            .into_iter()
            .map(|vlt| VaultListItem::new(vlt))
            .collect();
    }

    fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        let mut active_amount: u64 = 0;
        let mut inactive_amount: u64 = 0;
        let mut unsecured_amount: u64 = 0;
        for vault in vaults {
            match vault.status {
                VaultStatus::Active | VaultStatus::Unvaulting | VaultStatus::Unvaulted => {
                    active_amount += vault.amount
                }
                VaultStatus::Unconfirmed | VaultStatus::Funded => {
                    inactive_amount += vault.amount;
                    unsecured_amount += vault.amount;
                }
                VaultStatus::Secured => {
                    inactive_amount += vault.amount;
                }
                _ => {}
            }
        }

        self.balance = (active_amount, inactive_amount);
        self.unsecured_fund_balance = unsecured_amount;
    }
}

impl State for StakeholderHomeState {
    fn update(&mut self, message: Message) -> Command<Message> {
        if let Message::Vaults(res) = message {
            match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            }
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            None,
            self.vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            &self.balance,
            &self.unsecured_fund_balance,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(list_vaults(self.revaultd.clone(), None), Message::Vaults),
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
pub struct StakeholderACKFundsState {
    revaultd: Arc<RevaultD>,

    warning: Option<Error>,
    balance: u64,
    deposits: Vec<VaultListItem<AcknowledgeVaultListItemView>>,
    selected_vault: Option<Vault>,

    view: StakeholderACKFundsView,
}

impl StakeholderACKFundsState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderACKFundsState {
            revaultd,
            warning: None,
            deposits: Vec::new(),
            view: StakeholderACKFundsView::new(),
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
                |res| Message::Vault(VaultMessage::RevocationTransactions(res)),
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

impl State for StakeholderACKFundsState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vault(VaultMessage::Select(outpoint)) => self.on_vault_select(outpoint),
            Message::Vault(msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    return selected.update(self.revaultd.clone(), msg);
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
        self.view
            .view(ctx, self.deposits.iter_mut().map(|v| v.view(ctx)).collect())
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(
                self.revaultd.clone(),
                Some(&[VaultStatus::Securing, VaultStatus::Funded]),
            ),
            Message::Vaults,
        )])
    }
}

impl From<StakeholderACKFundsState> for Box<dyn State> {
    fn from(s: StakeholderACKFundsState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct StakeholderDelegateFundsState {
    revaultd: Arc<RevaultD>,

    active_balance: u64,
    vault_status_filter: Vec<VaultStatus>,
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
            vaults: Vec::new(),
            vault_status_filter: vec![
                VaultStatus::Funded,
                VaultStatus::Securing,
                VaultStatus::Activating,
                VaultStatus::Secured,
            ],
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
            return cmd;
        };
        Command::none()
    }

    pub fn on_vault_delegate(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &mut self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                return selected.update(self.revaultd.clone(), VaultMessage::Delegate(outpoint));
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let mut selected_vault = Vault::new(selected.vault.clone());
            let cmd =
                selected_vault.update(self.revaultd.clone(), VaultMessage::Delegate(outpoint));
            self.selected_vault = Some(selected_vault);
            return cmd;
        };
        Command::none()
    }

    pub fn on_vault_acknowledge(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &mut self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                return selected.update(self.revaultd.clone(), VaultMessage::Acknowledge(outpoint));
            }
        }

        if let Some(selected) = self
            .vaults
            .iter()
            .find(|vlt| vlt.vault.outpoint() == outpoint)
        {
            let mut selected_vault = Vault::new(selected.vault.clone());
            let cmd =
                selected_vault.update(self.revaultd.clone(), VaultMessage::Acknowledge(outpoint));
            self.selected_vault = Some(selected_vault);
            return cmd;
        };
        Command::none()
    }

    pub fn calculate_balance(&mut self, vaults: &[model::Vault]) {
        self.active_balance = 0;
        for vault in vaults {
            match vault.status {
                VaultStatus::Active | VaultStatus::Unvaulting | VaultStatus::Unvaulted => {
                    self.active_balance += vault.amount
                }
                _ => {}
            }
        }
    }
}

impl State for StakeholderDelegateFundsState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vault(msg) => match msg {
                VaultMessage::Select(outpoint) => return self.on_vault_select(outpoint),
                VaultMessage::Acknowledge(outpoint) => return self.on_vault_acknowledge(outpoint),
                VaultMessage::Delegate(outpoint) => return self.on_vault_delegate(outpoint),
                _ => {
                    if let Some(vault) = &mut self.selected_vault {
                        return vault.update(self.revaultd.clone(), msg);
                    }
                    return Command::none();
                }
            },
            Message::FilterVaults(VaultFilterMessage::Status(statuses)) => {
                self.vault_status_filter = statuses;
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }
        let status_filters = &self.vault_status_filter;
        self.view.view(
            ctx,
            &self.active_balance,
            self.vaults
                .iter_mut()
                .filter(|v| status_filters.contains(&v.vault.status))
                .map(|v| v.view(ctx))
                .collect(),
            self.warning.as_ref().into(),
            &self.vault_status_filter.contains(&VaultStatus::Active),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::perform(
            list_vaults(
                self.revaultd.clone(),
                Some(&[
                    VaultStatus::Funded,
                    VaultStatus::Securing,
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
