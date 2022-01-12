use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::{Command, Element, Subscription};

use crate::daemon::{
    client::{Client, RevaultD},
    model::{self, HistoryEventKind, VaultStatus},
};

use crate::app::{
    context::Context,
    error::Error,
    menu::Menu,
    message::{Message, SignMessage},
    state::{
        cmd::{get_blockheight, get_history, list_vaults},
        history::HistoryEventState,
        sign::Device,
        vault::{Vault, VaultListItem},
        State,
    },
    view::{
        vault::{DelegateVaultListItemView, VaultListItemView},
        LoadingModal, StakeholderCreateVaultsView, StakeholderDelegateVaultsView,
        StakeholderHomeView, StakeholderSelecteVaultsToDelegateView,
    },
};

#[derive(Debug)]
pub struct StakeholderHomeState {
    warning: Option<Error>,

    balance: HashMap<VaultStatus, (u64, u64)>,

    moving_vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,

    latest_events: Vec<HistoryEventState>,

    view: StakeholderHomeView,
}

impl StakeholderHomeState {
    pub fn new() -> Self {
        StakeholderHomeState {
            warning: None,
            view: StakeholderHomeView::new(),
            balance: HashMap::new(),
            moving_vaults: Vec::new(),
            latest_events: Vec::new(),
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

    pub fn on_vault_select<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
        outpoint: String,
    ) -> Command<Message> {
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

impl<C: Client + Sync + Send + 'static> State<C> for StakeholderHomeState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
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
            Message::HistoryEvents(res) => match res {
                Ok(events) => {
                    self.latest_events = events.into_iter().map(HistoryEventState::new).collect();
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                }
            },
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }

        self.view.view(
            ctx,
            None,
            self.moving_vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            self.latest_events.iter_mut().map(|e| e.view(ctx)).collect(),
            &self.balance,
        )
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
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
            Command::perform(
                get_history(
                    ctx.revaultd.clone(),
                    vec![
                        HistoryEventKind::Cancel,
                        HistoryEventKind::Deposit,
                        HistoryEventKind::Spend,
                    ],
                    0,
                    now,
                    5,
                ),
                Message::HistoryEvents,
            ),
        ])
    }
}

impl<C: Client + Sync + Send + 'static> From<StakeholderHomeState> for Box<dyn State<C>> {
    fn from(s: StakeholderHomeState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub enum StakeholderCreateVaultsState {
    Loading {
        fail: Option<Error>,
        view: LoadingModal,
    },
    Loaded {
        device: Device,
        processing: bool,
        deposits: Vec<model::Vault>,
        warning: Option<Error>,
        view: StakeholderCreateVaultsView,
    },
}

impl StakeholderCreateVaultsState {
    pub fn new() -> Self {
        StakeholderCreateVaultsState::Loading {
            fail: None,
            view: LoadingModal::new(),
        }
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for StakeholderCreateVaultsState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::Vaults(res) = message {
                    match res {
                        Ok(deposits) => {
                            *self = Self::Loaded {
                                device: Device::new(),
                                processing: false,
                                deposits,
                                warning: None,
                                view: StakeholderCreateVaultsView::new(),
                            };
                        }
                        Err(e) => *fail = Some(Error::RevaultDError(e)),
                    };
                }
                Command::none()
            }
            Self::Loaded {
                device,
                processing,
                deposits,
                warning,
                ..
            } => match message {
                Message::DepositsSecured(res) => match res {
                    Ok(secured_deposits_outpoints) => {
                        let mut deposits_to_secure = Vec::new();
                        for deposit in deposits.iter_mut() {
                            if secured_deposits_outpoints.contains(&deposit.outpoint()) {
                                deposit.status = VaultStatus::Securing;
                            } else if deposit.status != VaultStatus::Securing
                                && deposit.status != VaultStatus::Secured
                            {
                                deposits_to_secure.push(deposit.clone());
                            }
                        }
                        if !deposits_to_secure.is_empty() {
                            Command::perform(
                                secure_deposits(
                                    ctx.revaultd.clone(),
                                    device.clone(),
                                    deposits_to_secure.clone(),
                                ),
                                Message::DepositsSecured,
                            )
                        } else {
                            Command::none()
                        }
                    }
                    Err(e) => {
                        *warning = Some(e);
                        Command::none()
                    }
                },
                Message::Sign(SignMessage::SelectSign) => {
                    *processing = true;
                    if !deposits.is_empty() {
                        Command::perform(
                            secure_deposits(ctx.revaultd.clone(), device.clone(), deposits.clone()),
                            Message::DepositsSecured,
                        )
                    } else {
                        Command::none()
                    }
                }
                Message::Sign(msg) => device.update(ctx, msg).map(Message::Sign),
                _ => Command::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            Self::Loaded { device, .. } => device.subscription().map(Message::Sign),
            _ => Subscription::none(),
        }
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref(), Menu::Home),
            Self::Loaded {
                view,
                warning,
                deposits,
                processing,
                device,
                ..
            } => view.view(
                ctx,
                deposits,
                *processing,
                device.is_connected(),
                warning.as_ref(),
            ),
        }
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&[VaultStatus::Funded]), None),
            Message::Vaults,
        )])
    }
}

pub async fn secure_deposits<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    device: Device,
    deposits: Vec<model::Vault>,
) -> Result<Vec<String>, Error> {
    match device.clone().secure_batch(&deposits).await {
        Ok(revocation_txs) => {
            for (i, (emergency_tx, emergency_unvault_tx, cancel_tx)) in
                revocation_txs.into_iter().enumerate()
            {
                revaultd.set_revocation_txs(
                    &deposits[i].outpoint(),
                    &emergency_tx,
                    &emergency_unvault_tx,
                    &cancel_tx,
                )?;
            }

            return Ok(deposits
                .into_iter()
                .map(|deposit| deposit.outpoint())
                .collect());
        }
        Err(revault_hwi::HWIError::UnimplementedMethod) => {
            log::info!("device does not support batching");
        }
        Err(e) => return Err(e.into()),
    };

    // Batching is not supported, so we secure only the first one.
    if let Some(deposit) = deposits.into_iter().nth(0) {
        let outpoint = deposit.outpoint();
        let revocation_txs = revaultd.get_revocation_txs(&outpoint)?;

        let (emergency_tx, emergency_unvault_tx, cancel_tx) = device
            .sign_revocation_txs(
                revocation_txs.emergency_tx.clone(),
                revocation_txs.emergency_unvault_tx.clone(),
                revocation_txs.cancel_tx.clone(),
            )
            .await?;

        revaultd.set_revocation_txs(&outpoint, &emergency_tx, &emergency_unvault_tx, &cancel_tx)?;

        Ok(vec![outpoint])
    } else {
        Ok(Vec::new())
    }
}

impl<C: Client + Send + Sync + 'static> From<StakeholderCreateVaultsState> for Box<dyn State<C>> {
    fn from(s: StakeholderCreateVaultsState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
#[derive(Debug)]
pub struct DelegateVaultListItem {
    vault: model::Vault,
    selected: bool,
    view: DelegateVaultListItemView,
}

impl DelegateVaultListItem {
    pub fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            selected: false,
            view: DelegateVaultListItemView::new(),
        }
    }

    pub fn view<C: Client>(&mut self, ctx: &Context<C>) -> Element<Message> {
        self.view.view(ctx, &self.vault, self.selected)
    }
}

#[derive(Debug)]
pub enum StakeholderDelegateVaultsState {
    Loading {
        fail: Option<Error>,
        view: LoadingModal,
    },
    SelectVaults {
        active_balance: u64,
        activating_balance: u64,
        vaults: Vec<DelegateVaultListItem>,
        view: StakeholderSelecteVaultsToDelegateView,
    },
    Signing {
        device: Device,
        processing: bool,
        vaults: Vec<model::Vault>,
        warning: Option<Error>,
        view: StakeholderDelegateVaultsView,
    },
}

impl StakeholderDelegateVaultsState {
    pub fn new() -> Self {
        StakeholderDelegateVaultsState::Loading {
            fail: None,
            view: LoadingModal::new(),
        }
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for StakeholderDelegateVaultsState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::Vaults(res) = message {
                    match res {
                        Ok(vaults) => {
                            let (active_balance, activating_balance) =
                                vaults.iter().fold((0, 0), |acc, vault| {
                                    if vault.status == VaultStatus::Active {
                                        (acc.0 + vault.amount, acc.1)
                                    } else if vault.status == VaultStatus::Activating {
                                        (acc.0, acc.1 + vault.amount)
                                    } else {
                                        acc
                                    }
                                });

                            *self = Self::SelectVaults {
                                active_balance,
                                activating_balance,
                                vaults: vaults
                                    .into_iter()
                                    .filter(|vlt| vlt.status == VaultStatus::Secured)
                                    .map(DelegateVaultListItem::new)
                                    .collect(),
                                view: StakeholderSelecteVaultsToDelegateView::new(),
                            };
                        }
                        Err(e) => *fail = Some(Error::RevaultDError(e)),
                    };
                }
                Command::none()
            }
            Self::SelectVaults { vaults, .. } => match message {
                Message::SelectVault(outpoint) => {
                    for vlt in vaults.iter_mut() {
                        if vlt.vault.outpoint() == outpoint {
                            vlt.selected = !vlt.selected
                        }
                    }
                    Command::none()
                }
                Message::Next => {
                    *self = Self::Signing {
                        vaults: vaults
                            .iter()
                            .filter(|vlt| vlt.selected)
                            .map(|vlt| vlt.vault.clone())
                            .collect(),
                        device: Device::new(),
                        processing: false,
                        warning: None,
                        view: StakeholderDelegateVaultsView::new(),
                    };
                    Command::none()
                }
                _ => Command::none(),
            },
            Self::Signing {
                device,
                processing,
                vaults,
                warning,
                ..
            } => match message {
                Message::VaultsDelegated(res) => match res {
                    Ok(activated_vaults_outpoints) => {
                        let mut vaults_to_delegate = Vec::new();
                        for vault in vaults.iter_mut() {
                            if activated_vaults_outpoints.contains(&vault.outpoint()) {
                                vault.status = VaultStatus::Activating;
                            } else if vault.status != VaultStatus::Activating
                                && vault.status != VaultStatus::Active
                            {
                                vaults_to_delegate.push(vault.clone());
                            }
                        }
                        if !vaults_to_delegate.is_empty() {
                            Command::perform(
                                delegate_vaults(
                                    ctx.revaultd.clone(),
                                    device.clone(),
                                    vaults_to_delegate.clone(),
                                ),
                                Message::VaultsDelegated,
                            )
                        } else {
                            Command::none()
                        }
                    }
                    Err(e) => {
                        *warning = Some(e);
                        Command::none()
                    }
                },
                Message::Sign(SignMessage::SelectSign) => {
                    *processing = true;
                    if !vaults.is_empty() {
                        Command::perform(
                            delegate_vaults(ctx.revaultd.clone(), device.clone(), vaults.clone()),
                            Message::VaultsDelegated,
                        )
                    } else {
                        Command::none()
                    }
                }
                Message::Sign(msg) => device.update(ctx, msg).map(Message::Sign),
                _ => Command::none(),
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            Self::Signing { device, .. } => device.subscription().map(Message::Sign),
            _ => Subscription::none(),
        }
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref(), Menu::Home),
            Self::SelectVaults {
                view,
                active_balance,
                activating_balance,
                vaults,
            } => view.view(
                ctx,
                active_balance,
                activating_balance,
                vaults
                    .iter()
                    .filter(|v| v.selected)
                    .fold((0, 0), |(count, total), v| {
                        (count + 1, total + v.vault.amount)
                    }),
                vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            ),
            Self::Signing {
                view,
                warning,
                vaults,
                processing,
                device,
                ..
            } => view.view(
                ctx,
                vaults,
                *processing,
                device.is_connected(),
                warning.as_ref(),
            ),
        }
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![Command::perform(
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
        )])
    }
}

pub async fn delegate_vaults<C: Client>(
    revaultd: Arc<RevaultD<C>>,
    device: Device,
    vaults: Vec<model::Vault>,
) -> Result<Vec<String>, Error> {
    match device.clone().delegate_batch(&vaults).await {
        Ok(revocation_txs) => {
            for (i, unvault_tx) in revocation_txs.into_iter().enumerate() {
                revaultd.set_unvault_tx(&vaults[i].outpoint(), &unvault_tx)?;
            }

            return Ok(vaults.into_iter().map(|vault| vault.outpoint()).collect());
        }
        Err(revault_hwi::HWIError::UnimplementedMethod) => {
            log::info!("device does not support batching");
        }
        Err(e) => return Err(e.into()),
    };

    // Batching is not supported, so we secure only the first one.
    if let Some(vault) = vaults.into_iter().nth(0) {
        let outpoint = vault.outpoint();
        let res = revaultd.get_unvault_tx(&outpoint)?;
        let unvault_tx = device.sign_unvault_tx(res.unvault_tx).await?;
        revaultd.set_unvault_tx(&outpoint, &unvault_tx)?;

        Ok(vec![outpoint])
    } else {
        Ok(Vec::new())
    }
}

impl<C: Client + Send + Sync + 'static> From<StakeholderDelegateVaultsState> for Box<dyn State<C>> {
    fn from(s: StakeholderDelegateVaultsState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
