use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{util::bip32::Fingerprint, OutPoint};
use iced::{Command, Element, Subscription};

use revaultd::revault_tx::transactions::RevaultTransaction;

use crate::daemon::{
    model::{
        self, outpoint, VaultStatus, ALL_HISTORY_EVENTS, CURRENT_VAULT_STATUSES,
        MOVING_VAULT_STATUSES,
    },
    Daemon,
};

use crate::app::{
    context::Context,
    error::Error,
    menu::Menu,
    message::{Message, SignMessage, VaultFilterMessage},
    state::{
        cmd::list_vaults,
        history::{HistoryEventListItemState, HistoryEventState},
        sign::Device,
        vault::{Vault, VaultListItem},
        State,
    },
    view::{
        stakeholder::DelegateVaultsFilter,
        vault::{DelegateVaultListItemView, VaultListItemView},
        LoadingDashboard, LoadingModal, StakeholderCreateVaultsView, StakeholderDelegateVaultsView,
        StakeholderHomeView, StakeholderSelecteVaultsToDelegateView,
    },
};

#[derive(Debug)]
pub enum StakeholderHomeState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    /// ManagerHomeState is considered as loaded once the vaults are loaded,
    /// then after update_vaults, spend_tx and history events are loaded.
    Loaded {
        warning: Option<Error>,

        balance: HashMap<VaultStatus, (u64, u64)>,

        moving_vaults: Vec<VaultListItem<VaultListItemView>>,
        selected_vault: Option<Vault>,

        latest_events: Vec<HistoryEventListItemState>,
        selected_event: Option<HistoryEventState>,

        view: StakeholderHomeView,
    },
}

impl StakeholderHomeState {
    pub fn new() -> Self {
        StakeholderHomeState::Loading {
            view: LoadingDashboard::new(),
            fail: None,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        let mut total_balance = HashMap::new();
        for vault in &vaults {
            if vault.status == VaultStatus::Unconfirmed
                || MOVING_VAULT_STATUSES.contains(&vault.status)
            {
                continue;
            }
            if let Some((number, amount)) = total_balance.get_mut(&vault.status) {
                *number += 1;
                *amount += vault.amount.as_sat();
            } else {
                total_balance.insert(vault.status.clone(), (1, vault.amount.as_sat()));
            }
        }

        let moving_vlts = vaults
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

        match self {
            Self::Loading { .. } => {
                *self = Self::Loaded {
                    balance: total_balance,
                    warning: None,
                    moving_vaults: moving_vlts,
                    selected_vault: None,
                    selected_event: None,
                    latest_events: Vec::new(),
                    view: StakeholderHomeView::new(),
                };
            }
            Self::Loaded {
                balance,
                moving_vaults,
                ..
            } => {
                *balance = total_balance;
                *moving_vaults = moving_vlts;
            }
        }
    }
    fn load_history(ctx: &Context) -> Command<Message> {
        let now: u32 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        let revaultd = ctx.revaultd.clone();
        Command::perform(
            async move { revaultd.get_history(&ALL_HISTORY_EVENTS, 0, now, 5) },
            Message::HistoryEvents,
        )
    }

    fn load_vaults(ctx: &Context) -> Command<Message> {
        Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&CURRENT_VAULT_STATUSES), None),
            Message::Vaults,
        )
    }
}

impl State for StakeholderHomeState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::Vaults(res) = message {
                    match res {
                        Ok(vaults) => {
                            self.update_vaults(vaults);
                            return Self::load_history(ctx);
                        }
                        Err(e) => *fail = Some(e.into()),
                    }
                }
            }
            Self::Loaded {
                warning,
                latest_events,
                selected_vault,
                selected_event,
                moving_vaults,
                ..
            } => match message {
                Message::Reload => return self.load(ctx),
                Message::Vaults(res) => match res {
                    Ok(vaults) => {
                        self.update_vaults(vaults);
                        return Self::load_history(ctx);
                    }
                    Err(e) => *warning = Error::from(e).into(),
                },
                Message::SelectVault(selected_outpoint) => {
                    if let Some(selected) = selected_vault {
                        if outpoint(&selected.vault) == selected_outpoint {
                            *selected_vault = None;
                            return self.load(ctx);
                        }
                    }

                    if let Some(selected) = moving_vaults
                        .iter()
                        .find(|vlt| outpoint(&vlt.vault) == selected_outpoint)
                    {
                        let vault = Vault::new(selected.vault.clone());
                        let cmd = vault.load(ctx.revaultd.clone());
                        *selected_vault = Some(vault);
                        return cmd.map(Message::Vault);
                    };
                }
                Message::Vault(msg) => {
                    if let Some(selected) = selected_vault {
                        return selected.update(ctx, msg).map(Message::Vault);
                    }
                }
                Message::SelectHistoryEvent(i) => {
                    if let Some(item) = latest_events.get(i) {
                        let state = HistoryEventState::new(item.event.clone());
                        let cmd = state.load(ctx);
                        *selected_event = Some(state);
                        return cmd;
                    }
                }
                Message::HistoryEvent(msg) => {
                    if let Some(event) = selected_event {
                        event.update(msg)
                    }
                }
                Message::Close => {
                    if selected_event.is_some() {
                        *selected_event = None;
                    }
                }
                Message::HistoryEvents(res) => match res {
                    Ok(events) => {
                        *latest_events = events
                            .into_iter()
                            .map(HistoryEventListItemState::new)
                            .collect();
                    }
                    Err(e) => {
                        *warning = Some(Error::from(e));
                    }
                },
                _ => {}
            },
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { view, fail } => view.view(ctx, fail.as_ref()),
            Self::Loaded {
                selected_vault,
                selected_event,
                moving_vaults,
                latest_events,
                balance,
                view,
                warning,
            } => {
                if let Some(v) = selected_vault {
                    return v.view(ctx);
                }

                if let Some(v) = selected_event {
                    return v.view(ctx);
                }

                view.view(
                    ctx,
                    warning.as_ref(),
                    moving_vaults.iter_mut().map(|v| v.view(ctx)).collect(),
                    latest_events
                        .iter_mut()
                        .enumerate()
                        .map(|(i, evt)| evt.view(ctx, i))
                        .collect(),
                    &balance,
                )
            }
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Self::load_vaults(ctx)
    }
}

impl From<StakeholderHomeState> for Box<dyn State> {
    fn from(s: StakeholderHomeState) -> Box<dyn State> {
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

impl State for StakeholderCreateVaultsState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
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
                            if secured_deposits_outpoints.contains(&outpoint(deposit)) {
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

    fn view(&mut self, ctx: &Context) -> Element<Message> {
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

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&[VaultStatus::Funded]), None),
            Message::Vaults,
        )])
    }
}

pub async fn secure_deposits(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    device: Device,
    deposits: Vec<model::Vault>,
) -> Result<Vec<OutPoint>, Error> {
    match device.clone().secure_batch(&deposits).await {
        Ok(revocation_txs) => {
            for (i, (emergency_tx, emergency_unvault_tx, cancel_tx)) in
                revocation_txs.into_iter().enumerate()
            {
                revaultd.set_revocation_txs(
                    &outpoint(&deposits[i]),
                    &emergency_tx,
                    &emergency_unvault_tx,
                    &cancel_tx,
                )?;
            }

            return Ok(deposits.iter().map(outpoint).collect());
        }
        Err(revault_hwi::HWIError::UnimplementedMethod) => {
            log::info!("device does not support batching");
        }
        Err(e) => return Err(e.into()),
    };

    // Batching is not supported, so we secure only the first one.
    if let Some(deposit) = deposits.into_iter().nth(0) {
        let outpoint = outpoint(&deposit);
        let revocation_txs = revaultd.get_revocation_txs(&outpoint)?;

        let (emergency_tx, emergency_unvault_tx, cancel_tx) = device
            .sign_revocation_txs(
                revocation_txs.emergency_tx.into_psbt(),
                revocation_txs.emergency_unvault_tx.into_psbt(),
                revocation_txs.cancel_tx.into_psbt(),
            )
            .await?;

        revaultd.set_revocation_txs(&outpoint, &emergency_tx, &emergency_unvault_tx, &cancel_tx)?;

        Ok(vec![outpoint])
    } else {
        Ok(Vec::new())
    }
}

impl From<StakeholderCreateVaultsState> for Box<dyn State> {
    fn from(s: StakeholderCreateVaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}
#[derive(Debug)]
pub struct DelegateVaultListItem {
    vault: model::Vault,
    sigs: Vec<Fingerprint>,
    selected: bool,
    view: DelegateVaultListItemView,
}

impl DelegateVaultListItem {
    pub fn new(vault: model::Vault, sigs: Vec<Fingerprint>) -> Self {
        Self {
            vault,
            sigs,
            selected: false,
            view: DelegateVaultListItemView::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault, &self.sigs, self.selected)
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
        vault_status_filter: &'static [VaultStatus],
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

impl State for StakeholderDelegateVaultsState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::VaultsWithPresignedTxs(res) = message {
                    match res {
                        Ok(vaults) => {
                            let (active_balance, activating_balance) =
                                vaults.iter().fold((0, 0), |acc, (vault, _)| {
                                    if vault.status == VaultStatus::Active {
                                        (acc.0 + vault.amount.as_sat(), acc.1)
                                    } else if vault.status == VaultStatus::Activating {
                                        (acc.0, acc.1 + vault.amount.as_sat())
                                    } else {
                                        acc
                                    }
                                });

                            let mut vaults: Vec<DelegateVaultListItem> = vaults
                                .into_iter()
                                .filter_map(|(vault, txs)| {
                                    if vault.status == VaultStatus::Secured
                                        || vault.status == VaultStatus::Activating
                                    {
                                        let unvault = txs.unvault.psbt.into_psbt();
                                        Some(DelegateVaultListItem::new(
                                            vault,
                                            unvault.inputs[0]
                                                .partial_sigs
                                                .keys()
                                                .filter_map(|key| {
                                                    unvault.inputs[0]
                                                        .bip32_derivation
                                                        .get(&key)
                                                        .map(|(fingerprint, _)| *fingerprint)
                                                })
                                                .collect(),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            vaults.sort_by(|a, b| {
                                if a.sigs.len() == b.sigs.len() {
                                    b.vault.amount.cmp(&a.vault.amount)
                                } else {
                                    b.sigs.cmp(&a.sigs)
                                }
                            });

                            *self = Self::SelectVaults {
                                active_balance,
                                activating_balance,
                                vault_status_filter: &DelegateVaultsFilter::ALL,
                                vaults,
                                view: StakeholderSelecteVaultsToDelegateView::new(),
                            };
                        }
                        Err(e) => *fail = Some(Error::RevaultDError(e)),
                    };
                }
                Command::none()
            }
            Self::SelectVaults {
                vaults,
                vault_status_filter,
                ..
            } => match message {
                Message::FilterVaults(VaultFilterMessage::Status(filter)) => {
                    *vault_status_filter = filter;
                    Command::none()
                }
                Message::SelectVault(selected_outpoint) => {
                    for vlt in vaults.iter_mut() {
                        if outpoint(&vlt.vault) == selected_outpoint {
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
                            if activated_vaults_outpoints.contains(&outpoint(vault)) {
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

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref(), Menu::Home),
            Self::SelectVaults {
                view,
                active_balance,
                activating_balance,
                vault_status_filter,
                vaults,
            } => view.view(
                ctx,
                active_balance,
                activating_balance,
                vault_status_filter,
                vaults
                    .iter()
                    .filter(|v| v.selected)
                    .fold((0, 0), |(count, total), v| {
                        (count + 1, total + v.vault.amount.as_sat())
                    }),
                vaults
                    .iter_mut()
                    .filter_map(|v| {
                        if vault_status_filter.contains(&v.vault.status) {
                            Some(v.view(ctx))
                        } else {
                            None
                        }
                    })
                    .collect(),
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

    fn load(&self, ctx: &Context) -> Command<Message> {
        let revaultd = ctx.revaultd.clone();
        Command::perform(
            async move {
                let vaults = revaultd.list_vaults(
                    Some(&[
                        VaultStatus::Secured,
                        VaultStatus::Activating,
                        VaultStatus::Active,
                    ]),
                    None,
                )?;
                let outpoints: Vec<OutPoint> =
                    vaults.iter().map(|vault| model::outpoint(vault)).collect();
                let vaults_txs = revaultd.list_presigned_transactions(outpoints.as_slice())?;

                let res: Vec<(model::Vault, model::VaultPresignedTransactions)> = vaults
                    .into_iter()
                    .map(|vault| {
                        let outpoint = model::outpoint(&vault);
                        let txs = vaults_txs
                            .iter()
                            .find(|vault_txs| vault_txs.vault_outpoint == outpoint)
                            .unwrap();
                        (vault, txs.clone())
                    })
                    .collect();
                Ok(res)
            },
            Message::VaultsWithPresignedTxs,
        )
    }
}

pub async fn delegate_vaults(
    revaultd: Arc<dyn Daemon + Send + Sync>,
    device: Device,
    vaults: Vec<model::Vault>,
) -> Result<Vec<OutPoint>, Error> {
    match device.clone().delegate_batch(&vaults).await {
        Ok(revocation_txs) => {
            for (i, unvault_tx) in revocation_txs.into_iter().enumerate() {
                revaultd.set_unvault_tx(&outpoint(&vaults[i]), &unvault_tx)?;
            }

            return Ok(vaults.iter().map(outpoint).collect());
        }
        Err(revault_hwi::HWIError::UnimplementedMethod) => {
            log::info!("device does not support batching");
        }
        Err(e) => return Err(e.into()),
    };

    // Batching is not supported, so we secure only the first one.
    if let Some(vault) = vaults.into_iter().nth(0) {
        let outpoint = outpoint(&vault);
        let res = revaultd.get_unvault_tx(&outpoint)?;
        let unvault_tx = device.sign_unvault_tx(res).await?;
        revaultd.set_unvault_tx(&outpoint, &unvault_tx)?;

        Ok(vec![outpoint])
    } else {
        Ok(Vec::new())
    }
}

impl From<StakeholderDelegateVaultsState> for Box<dyn State> {
    fn from(s: StakeholderDelegateVaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}
