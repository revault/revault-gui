use std::collections::{BTreeMap, HashMap};
use std::convert::From;
use std::convert::TryInto;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, OutPoint};
use iced::{Command, Element, Subscription};

use super::{
    cmd::{get_spend_tx, list_spend_txs, list_vaults, update_spend_tx},
    vault::{Vault, VaultListItem},
    State,
};

use crate::daemon::model::{self, VaultStatus, ALL_HISTORY_EVENTS, CURRENT_VAULT_STATUSES};

use revault_ui::component::form;
use revaultd::revault_tx::transactions::RevaultTransaction;

use crate::app::{
    context::Context,
    error::Error,
    message::{InputMessage, Message, RecipientMessage, SpendTxMessage},
    state::{
        history::{HistoryEventListItemState, HistoryEventState},
        sign::{Signer, SpendTransactionTarget},
        SpendTransactionListItem, SpendTransactionState,
    },
    view::{
        manager::{
            manager_send_input_view, ManagerImportTransactionView, ManagerSelectFeeView,
            ManagerSelectInputsView, ManagerSelectOutputsView, ManagerSendOutputView,
            ManagerSendWelcomeView, ManagerSpendTransactionCreatedView, ManagerStepSignView,
        },
        vault::VaultListItemView,
        LoadingDashboard, ManagerHomeView,
    },
};

#[derive(Debug)]
pub enum ManagerHomeState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    /// ManagerHomeState is considered as loaded once the vaults are loaded,
    /// then after update_vaults, spend_tx and history events are loaded.
    Loaded {
        view: ManagerHomeView,

        active_funds: u64,
        inactive_funds: u64,
        warning: Option<Error>,

        moving_vaults: Vec<VaultListItem<VaultListItemView>>,
        spendable_outpoints: HashMap<String, u64>,
        selected_vault: Option<Vault>,

        spend_txs: Vec<model::SpendTx>,
        spend_txs_item: Vec<SpendTransactionListItem>,
        selected_spend_tx: Option<SpendTransactionState>,

        latest_events: Vec<HistoryEventListItemState>,
        selected_event: Option<HistoryEventState>,
    },
}

impl ManagerHomeState {
    pub fn new() -> Self {
        ManagerHomeState::Loading {
            view: LoadingDashboard::new(),
            fail: None,
        }
    }

    pub fn update_spend_txs(&mut self, txs: Vec<model::SpendTx>) {
        if let Self::Loaded {
            spend_txs,
            spend_txs_item,
            spendable_outpoints,
            ..
        } = self
        {
            *spend_txs = txs
                .into_iter()
                .filter(|tx| {
                    tx.deposit_outpoints
                        .iter()
                        .all(|outpoint| spendable_outpoints.get(&outpoint.to_string()).is_some())
                })
                .collect();
            *spend_txs_item = spend_txs
                .clone()
                .into_iter()
                .map(|s| {
                    (
                        s.deposit_outpoints.iter().fold(0, |acc, x| {
                            acc + *spendable_outpoints
                                .get(&x.to_string())
                                .expect("Must be spendable")
                        }),
                        s,
                    )
                })
                .map(|(vaults_amount, s)| SpendTransactionListItem::new(s, vaults_amount))
                .collect();
        };
    }

    pub fn on_spend_tx_select(&mut self, ctx: &Context, psbt: Psbt) -> Command<Message> {
        if let Self::Loaded {
            spend_txs,
            selected_spend_tx,
            ..
        } = self
        {
            if let Some(selected) = selected_spend_tx {
                if selected.psbt.global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid() {
                    *selected_spend_tx = None;
                    return Command::none();
                }
            }

            if spend_txs.iter().any(|item| {
                item.psbt.psbt().global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid()
            }) {
                let spend_tx = SpendTransactionState::new(ctx, psbt);
                let cmd = spend_tx.load(ctx);
                *selected_spend_tx = Some(spend_tx);
                return cmd;
            };
        }
        Command::none()
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        let (act_funds, inact_funds) =
            vaults.iter().fold((0, 0), |acc, vault| match vault.status {
                VaultStatus::Active => (acc.0 + vault.amount.as_sat(), acc.1),
                VaultStatus::Funded | VaultStatus::Securing | VaultStatus::Secured => {
                    (acc.0, acc.1 + vault.amount.as_sat())
                }
                _ => (acc.0, acc.1),
            });

        let spendable_vlts = vaults
            .iter()
            .filter_map(|vlt| {
                if vlt.status == VaultStatus::Active {
                    Some((vlt.outpoint().to_string(), vlt.amount.as_sat()))
                } else {
                    None
                }
            })
            .collect();

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
                    warning: None,
                    active_funds: act_funds,
                    inactive_funds: inact_funds,
                    spendable_outpoints: spendable_vlts,
                    moving_vaults: moving_vlts,
                    latest_events: Vec::new(),
                    spend_txs: Vec::new(),
                    spend_txs_item: Vec::new(),
                    selected_vault: None,
                    selected_event: None,
                    selected_spend_tx: None,
                    view: ManagerHomeView::new(),
                }
            }
            Self::Loaded {
                active_funds,
                inactive_funds,
                spendable_outpoints,
                moving_vaults,
                ..
            } => {
                *active_funds = act_funds;
                *inactive_funds = inact_funds;
                *spendable_outpoints = spendable_vlts;
                *moving_vaults = moving_vlts;
            }
        }
    }

    pub fn on_vault_select(&mut self, ctx: &Context, outpoint: OutPoint) -> Command<Message> {
        if let Self::Loaded {
            selected_vault,
            moving_vaults,
            ..
        } = self
        {
            if let Some(selected) = selected_vault {
                if selected.vault.outpoint() == outpoint {
                    *selected_vault = None;
                    return self.load(ctx);
                }
            }

            if let Some(selected) = moving_vaults
                .iter()
                .find(|vlt| vlt.vault.outpoint() == outpoint)
            {
                let vault = Vault::new(selected.vault.clone());
                let cmd = vault.load(ctx.revaultd.clone());
                *selected_vault = Some(vault);
                return cmd.map(Message::Vault);
            }
        };
        Command::none()
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

    fn load_spend_txs(ctx: &Context) -> Command<Message> {
        Command::perform(
            list_spend_txs(
                ctx.revaultd.clone(),
                Some(&[model::SpendTxStatus::NonFinal]),
            ),
            Message::SpendTransactions,
        )
    }

    fn load_vaults(ctx: &Context) -> Command<Message> {
        Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&CURRENT_VAULT_STATUSES), None),
            Message::Vaults,
        )
    }
}

impl State for ManagerHomeState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::Vaults(res) = message {
                    match res {
                        Ok(vaults) => {
                            self.update_vaults(vaults);
                            return Command::batch(vec![
                                Self::load_history(ctx),
                                Self::load_spend_txs(ctx),
                            ]);
                        }
                        Err(e) => *fail = Some(e.into()),
                    }
                }
            }
            Self::Loaded {
                warning,
                selected_vault,
                selected_event,
                selected_spend_tx,
                latest_events,
                ..
            } => match message {
                Message::Reload => {
                    return self.load(ctx);
                }
                Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                    return self.on_spend_tx_select(ctx, psbt);
                }
                Message::SpendTx(msg) => {
                    if let Some(tx) = selected_spend_tx {
                        return tx.update(ctx, Message::SpendTx(msg));
                    }
                }
                Message::SpendTransactions(res) => match res {
                    Ok(txs) => self.update_spend_txs(txs),
                    Err(e) => *warning = Error::from(e).into(),
                },
                Message::Vaults(res) => match res {
                    Ok(vaults) => {
                        self.update_vaults(vaults);
                        return Command::batch(vec![
                            Self::load_history(ctx),
                            Self::load_spend_txs(ctx),
                        ]);
                    }
                    Err(e) => *warning = Error::from(e).into(),
                },
                Message::SelectVault(outpoint) => return self.on_vault_select(ctx, outpoint),
                Message::Vault(msg) => {
                    if let Some(selected) = selected_vault {
                        return selected.update(ctx, msg).map(Message::Vault);
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
                        *warning = Error::from(e).into();
                    }
                },
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
                _ => {}
            },
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Self::Loaded {
            selected_spend_tx, ..
        } = self
        {
            if let Some(v) = selected_spend_tx {
                return v.sub();
            }
        }
        Subscription::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref()),
            Self::Loaded {
                warning,
                selected_vault,
                selected_spend_tx,
                selected_event,
                spend_txs_item,
                moving_vaults,
                latest_events,
                active_funds,
                inactive_funds,
                view,
                ..
            } => {
                if let Some(v) = selected_vault {
                    return v.view(ctx);
                }

                if let Some(tx) = selected_spend_tx {
                    return tx.view(ctx);
                }

                if let Some(v) = selected_event {
                    return v.view(ctx);
                }

                view.view(
                    ctx,
                    warning.as_ref(),
                    spend_txs_item
                        .iter_mut()
                        .map(|tx| tx.view(ctx).map(Message::SpendTx))
                        .collect(),
                    moving_vaults.iter_mut().map(|v| v.view(ctx)).collect(),
                    latest_events
                        .iter_mut()
                        .enumerate()
                        .map(|(i, evt)| evt.view(ctx, i))
                        .collect(),
                    *active_funds,
                    *inactive_funds,
                )
            }
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Self::load_vaults(ctx)
    }
}

impl From<ManagerHomeState> for Box<dyn State> {
    fn from(s: ManagerHomeState) -> Box<dyn State> {
        Box::new(s)
    }
}

pub enum ManagerSendState {
    SendTransactionDetail(SpendTransactionState),
    ImportSendTransaction(ManagerImportSendTransactionState),
    CreateSendTransaction(ManagerCreateSendTransactionState),
}

impl ManagerSendState {
    pub fn new() -> Self {
        Self::CreateSendTransaction(ManagerCreateSendTransactionState::new())
    }
}

impl State for ManagerSendState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::CreateSendTransaction(state) => match message {
                Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                    *self = ManagerSendState::SendTransactionDetail(SpendTransactionState::new(
                        ctx, psbt,
                    ));
                    self.load(ctx)
                }
                Message::SpendTx(SpendTxMessage::Import) => {
                    *self = ManagerSendState::ImportSendTransaction(
                        ManagerImportSendTransactionState::new(),
                    );
                    self.load(ctx)
                }
                _ => state.update(ctx, message),
            },
            Self::ImportSendTransaction(state) => match message {
                Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                    *self = ManagerSendState::SendTransactionDetail(SpendTransactionState::new(
                        ctx, psbt,
                    ));
                    self.load(ctx)
                }
                _ => state.update(ctx, message),
            },
            Self::SendTransactionDetail(state) => state.update(ctx, message),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            ManagerSendState::SendTransactionDetail(s) => s.sub(),
            ManagerSendState::CreateSendTransaction(s) => s.sub(),
            _ => Subscription::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::CreateSendTransaction(state) => state.view(ctx),
            Self::ImportSendTransaction(state) => state.view(ctx),
            Self::SendTransactionDetail(state) => state.view(ctx),
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        match self {
            Self::CreateSendTransaction(state) => state.load(ctx),
            Self::ImportSendTransaction(state) => state.load(ctx),
            Self::SendTransactionDetail(state) => state.load(ctx),
        }
    }
}

#[derive(Debug)]
pub struct ManagerImportSendTransactionState {
    psbt_imported: Option<Psbt>,
    psbt_input: form::Value<String>,
    warning: Option<Error>,

    view: ManagerImportTransactionView,
}

impl ManagerImportSendTransactionState {
    pub fn new() -> Self {
        Self {
            psbt_imported: None,
            psbt_input: form::Value::default(),
            warning: None,
            view: ManagerImportTransactionView::new(),
        }
    }

    pub fn parse_pbst(&self) -> Option<Psbt> {
        bitcoin::base64::decode(&self.psbt_input.value)
            .ok()
            .and_then(|bytes| bitcoin::consensus::encode::deserialize(&bytes).ok())
    }
}

impl State for ManagerImportSendTransactionState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Updated(res)) => match res {
                Ok(()) => self.psbt_imported = self.parse_pbst(),
                Err(e) => self.warning = Some(Error::from(e)),
            },
            Message::SpendTx(SpendTxMessage::PsbtEdited(psbt)) => {
                self.warning = None;
                self.psbt_input.value = psbt;
            }
            Message::SpendTx(SpendTxMessage::Import) => {
                if !self.psbt_input.value.is_empty() {
                    if let Some(psbt) = self.parse_pbst() {
                        return Command::perform(
                            update_spend_tx(ctx.revaultd.clone(), psbt),
                            |res| Message::SpendTx(SpendTxMessage::Updated(res)),
                        );
                    } else {
                        self.psbt_input.valid = false;
                    }
                } else {
                    self.psbt_input.valid = false;
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            &self.psbt_input,
            self.psbt_imported.as_ref(),
            self.warning.as_ref(),
        )
    }

    fn load(&self, _ctx: &Context) -> Command<Message> {
        Command::none()
    }
}

#[derive(Debug)]
enum ManagerSendStep {
    WelcomeUser(ManagerSendWelcomeView),
    SelectOutputs(ManagerSelectOutputsView),
    SelectFee(ManagerSelectFeeView),
    SelectInputs(ManagerSelectInputsView),
    Sign {
        signer: Signer<SpendTransactionTarget>,
        view: ManagerStepSignView,
    },
    Success(ManagerSpendTransactionCreatedView),
}

#[derive(Debug)]
pub struct ManagerCreateSendTransactionState {
    warning: Option<Error>,

    vaults: Vec<ManagerSendInput>,
    outputs: Vec<ManagerSendOutput>,
    feerate: Option<u64>,
    psbt: Option<(Psbt, u64)>,
    cpfp_index: usize,
    change_index: Option<usize>,
    processing: bool,
    valid_feerate: bool,

    step: ManagerSendStep,
}

impl ManagerCreateSendTransactionState {
    pub fn new() -> Self {
        Self {
            step: ManagerSendStep::WelcomeUser(ManagerSendWelcomeView::new()),
            warning: None,
            vaults: Vec::new(),
            outputs: vec![ManagerSendOutput::new()],
            feerate: None,
            psbt: None,
            cpfp_index: 0,
            change_index: None,
            processing: false,
            valid_feerate: false,
        }
    }

    pub fn update_vaults(&mut self, mut vaults: Vec<model::Vault>) {
        // Ordering the vaults, the biggest amounts first
        vaults.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap());
        self.vaults = vaults.into_iter().map(ManagerSendInput::new).collect();
    }

    pub fn input_amount(&self) -> u64 {
        let mut input_amount = 0;
        for input in &self.vaults {
            if input.selected {
                input_amount += input.vault.amount.as_sat();
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

    pub fn selected_inputs(&self) -> Vec<model::Vault> {
        self.vaults
            .iter()
            .cloned()
            .filter_map(|input| {
                if input.selected {
                    Some(input.vault)
                } else {
                    None
                }
            })
            .collect()
    }

    // TODO: remove it for subscription
    // It was introduced because of difficulties with the trait type inference.
    pub fn sub(&self) -> Subscription<Message> {
        if let ManagerSendStep::Sign { signer, .. } = &self.step {
            return signer
                .subscription()
                .map(|msg| Message::SpendTx(SpendTxMessage::Sign(msg)));
        }
        Subscription::none()
    }
}

impl State for ManagerCreateSendTransactionState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::SpendTransaction(res) => {
                self.processing = false;
                match res {
                    Ok(spend) => {
                        self.psbt = Some(spend);
                    }
                    Err(e) => self.warning = Some(Error::RevaultDError(e)),
                }
                return self.update(ctx, Message::Next);
            }
            Message::SpendTx(SpendTxMessage::Generate) => {
                self.processing = true;
                self.warning = None;
                let inputs = self
                    .selected_inputs()
                    .into_iter()
                    .map(|input| input.outpoint())
                    .collect();

                let outputs: BTreeMap<bitcoin::Address, u64> = self
                    .outputs
                    .iter()
                    .map(|output| {
                        (
                            bitcoin::Address::from_str(&output.address.value).unwrap(),
                            output.amount().unwrap(),
                        )
                    })
                    .collect();

                return Command::perform(
                    get_spend_tx(ctx.revaultd.clone(), inputs, outputs, self.feerate.unwrap()),
                    Message::SpendTransaction,
                );
            }
            Message::SpendTx(SpendTxMessage::FeerateEdited(feerate)) => {
                if let Ok(f) = feerate.parse::<u64>() {
                    self.feerate = Some(f);
                    self.valid_feerate = true;
                } else if feerate.is_empty() {
                    self.feerate = None;
                    self.valid_feerate = false;
                }
            }
            Message::Vaults(res) => match res {
                Ok(vlts) => self.update_vaults(vlts),
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            Message::SpendTx(SpendTxMessage::Signed(res)) => match res {
                Ok(_) => {
                    if let ManagerSendStep::Sign { signer, .. } = &mut self.step {
                        // During this step state has a generated psbt
                        // and signer has a signed psbt.
                        self.psbt = Some((
                            signer.target.spend_tx.clone(),
                            self.psbt.clone().expect("As the received message is a sign success, the psbt should not be None").1,
                        ));
                        self.step =
                            ManagerSendStep::Success(ManagerSpendTransactionCreatedView::new());
                    };
                }
                Err(e) => self.warning = Some(Error::RevaultDError(e)),
            },
            Message::SpendTx(SpendTxMessage::Sign(msg)) => {
                if let ManagerSendStep::Sign { signer, .. } = &mut self.step {
                    let cmd = signer
                        .update(ctx, msg)
                        .map(|m| Message::SpendTx(SpendTxMessage::Sign(m)));
                    if signer.signed() {
                        return Command::perform(
                            update_spend_tx(ctx.revaultd.clone(), signer.target.spend_tx.clone()),
                            |res| Message::SpendTx(SpendTxMessage::Signed(res)),
                        );
                    }
                    return cmd;
                }
            }
            Message::Next => match self.step {
                ManagerSendStep::WelcomeUser(_) => {
                    self.step = ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new());
                }
                ManagerSendStep::SelectOutputs(_) => {
                    self.step = ManagerSendStep::SelectFee(ManagerSelectFeeView::new());
                }
                ManagerSendStep::SelectInputs(_) => {
                    if let Some((psbt, _)) = &self.psbt {
                        self.step = ManagerSendStep::Sign {
                            signer: Signer::new(SpendTransactionTarget::new(
                                &ctx.managers_xpubs()
                                    .iter()
                                    .map(|xpub| xpub.master_fingerprint())
                                    .collect(),
                                psbt.clone(),
                            )),
                            view: ManagerStepSignView::new(),
                        };
                    }
                }
                ManagerSendStep::SelectFee(_) => {
                    self.step = ManagerSendStep::SelectInputs(ManagerSelectInputsView::new());
                }
                _ => (),
            },
            Message::Previous => {
                // Because the process is going backward, the warning can be ignored.
                // Once the process goes upward again, the checks will set again
                // the warning in case of error.
                self.warning = None;
                self.step = match self.step {
                    ManagerSendStep::SelectInputs(_) => {
                        ManagerSendStep::SelectFee(ManagerSelectFeeView::new())
                    }
                    ManagerSendStep::SelectFee(_) => {
                        ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new())
                    }
                    ManagerSendStep::Sign { .. } => {
                        ManagerSendStep::SelectInputs(ManagerSelectInputsView::new())
                    }
                    _ => ManagerSendStep::SelectOutputs(ManagerSelectOutputsView::new()),
                }
            }
            Message::AddRecipient => self.outputs.push(ManagerSendOutput::new()),
            Message::Recipient(i, RecipientMessage::Delete) => {
                self.outputs.remove(i);
            }
            Message::Input(i, msg) => {
                self.psbt = None;
                if let Some(input) = self.vaults.get_mut(i) {
                    input.update(msg);
                }
            }
            Message::Recipient(i, msg) => {
                self.psbt = None;
                if let Some(output) = self.outputs.get_mut(i) {
                    output.update(msg);
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.sub()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        let selected_inputs = self.selected_inputs();
        let input_amount = self.input_amount();
        let output_amount = self.output_amount();
        match &mut self.step {
            ManagerSendStep::WelcomeUser(v) => v.view(ctx),
            ManagerSendStep::SelectOutputs(v) => {
                let mut valid =
                    !self.outputs.is_empty() && !self.outputs.iter().any(|o| !o.valid());
                let mut no_duplicate = true;
                for (i, output) in self.outputs.iter().enumerate() {
                    if self.outputs[i + 1..].iter().any(|o| {
                        o.address.value == output.address.value && !output.address.value.is_empty()
                    }) {
                        valid = false;
                        no_duplicate = false;
                    }
                }
                v.view(
                    self.outputs
                        .iter_mut()
                        .enumerate()
                        .map(|(i, v)| v.view().map(move |msg| Message::Recipient(i, msg)))
                        .collect(),
                    valid,
                    no_duplicate,
                )
            }
            ManagerSendStep::SelectInputs(v) => v.view(
                ctx,
                self.vaults
                    .iter_mut()
                    .enumerate()
                    .map(|(i, v)| v.view(ctx).map(move |msg| Message::Input(i, msg)))
                    .collect(),
                input_amount,
                output_amount,
                self.warning.as_ref(),
            ),
            ManagerSendStep::SelectFee(v) => {
                v.view(self.feerate, self.valid_feerate, self.warning.as_ref())
            }
            ManagerSendStep::Sign { signer, view } => {
                let (psbt, feerate) = self.psbt.as_ref().unwrap();
                view.view(
                    ctx,
                    &selected_inputs,
                    &psbt,
                    self.cpfp_index,
                    self.change_index,
                    &feerate,
                    signer.error.clone().as_ref(),
                    signer
                        .view(ctx)
                        .map(|m| Message::SpendTx(SpendTxMessage::Sign(m))),
                )
            }
            ManagerSendStep::Success(v) => {
                let (psbt, _) = self.psbt.as_ref().unwrap();
                v.view(
                    ctx,
                    &selected_inputs,
                    &psbt,
                    self.cpfp_index,
                    self.change_index,
                    &self.feerate.unwrap(),
                )
            }
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&[VaultStatus::Active]), None),
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
    address: form::Value<String>,
    amount: form::Value<String>,

    view: ManagerSendOutputView,
}

impl ManagerSendOutput {
    fn new() -> Self {
        Self {
            address: form::Value::default(),
            amount: form::Value::default(),
            view: ManagerSendOutputView::new(),
        }
    }

    fn amount(&self) -> Result<u64, Error> {
        if self.amount.value.is_empty() {
            return Err(Error::UnexpectedError(
                "Amount should be non-zero".to_string(),
            ));
        }

        let amount =
            bitcoin::Amount::from_str_in(&self.amount.value, bitcoin::Denomination::Bitcoin)
                .map_err(|_| Error::UnexpectedError("cannot parse output amount".to_string()))?;

        if amount.as_sat() == 0 {
            return Err(Error::UnexpectedError(
                "Amount should be non-zero".to_string(),
            ));
        }

        if let Ok(address) = bitcoin::Address::from_str(&self.address.value) {
            if amount <= address.script_pubkey().dust_value() {
                return Err(Error::UnexpectedError(
                    "Amount must be superior to script dust value".to_string(),
                ));
            }
        }

        Ok(amount.as_sat())
    }

    fn valid(&self) -> bool {
        !self.address.value.is_empty()
            && self.address.valid
            && !self.amount.value.is_empty()
            && self.amount.valid
    }

    fn update(&mut self, message: RecipientMessage) {
        match message {
            RecipientMessage::AddressEdited(address) => {
                self.address.value = address;
                if !self.address.value.is_empty()
                    && bitcoin::Address::from_str(&self.address.value).is_ok()
                {
                    self.address.valid = true;
                    if !self.amount.value.is_empty() {
                        self.amount.valid = self.amount().is_ok();
                    }
                } else {
                    // Make the error disappear if we deleted the invalid address
                    self.address.valid = true;
                }
            }
            RecipientMessage::AmountEdited(amount) => {
                self.amount.value = amount;
                if !self.amount.value.is_empty() {
                    self.amount.valid = self.amount().is_ok();
                } else {
                    // Make the error disappear if we deleted the invalid amount
                    self.amount.valid = true;
                }
            }
            _ => {}
        };
    }

    fn view(&mut self) -> Element<RecipientMessage> {
        self.view.view(&self.address, &self.amount)
    }
}

#[derive(Debug, Clone)]
struct ManagerSendInput {
    vault: model::Vault,
    selected: bool,
}

impl ManagerSendInput {
    fn new(vault: model::Vault) -> Self {
        Self {
            vault,
            selected: false,
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<InputMessage> {
        manager_send_input_view(
            ctx,
            &self.vault.outpoint().to_string(),
            &self.vault.amount.as_sat(),
            self.selected,
        )
    }

    pub fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::Selected(selected) => self.selected = selected,
        }
    }
}
