use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::collections::HashMap;
use std::convert::From;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::{Command, Element, Subscription};

use super::{
    cmd::{
        get_blockheight, get_history, get_spend_tx, list_spend_txs, list_vaults, update_spend_tx,
    },
    vault::{Vault, VaultListItem},
    State,
};

use crate::daemon::{
    client::Client,
    model::{self, HistoryEventKind, VaultStatus},
};

use revault_ui::component::form;

use crate::app::{
    context::Context,
    error::Error,
    message::{InputMessage, Message, RecipientMessage, SpendTxMessage},
    state::{
        history::HistoryEventState,
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
        ManagerHomeView,
    },
};

#[derive(Debug)]
pub struct ManagerHomeState {
    view: ManagerHomeView,

    active_funds: u64,
    inactive_funds: u64,
    blockheight: u64,
    warning: Option<Error>,

    moving_vaults: Vec<VaultListItem<VaultListItemView>>,
    spendable_outpoints: HashMap<String, u64>,
    selected_vault: Option<Vault>,

    spend_txs: Vec<model::SpendTx>,
    spend_txs_item: Vec<SpendTransactionListItem>,
    selected_spend_tx: Option<SpendTransactionState>,

    latest_events: Vec<HistoryEventState>,

    loading_vaults: bool,
}

impl ManagerHomeState {
    pub fn new() -> Self {
        ManagerHomeState {
            active_funds: 0,
            inactive_funds: 0,
            view: ManagerHomeView::new(),
            blockheight: 0,
            spendable_outpoints: HashMap::new(),
            moving_vaults: Vec::new(),
            warning: None,
            selected_vault: None,
            spend_txs: Vec::new(),
            spend_txs_item: Vec::new(),
            selected_spend_tx: None,
            latest_events: Vec::new(),
            loading_vaults: true,
        }
    }

    pub fn update_spend_txs(&mut self, txs: Vec<model::SpendTx>) {
        self.spend_txs = if self.loading_vaults {
            // Don't filter the txs if we still don't have the vaults!
            txs
        } else {
            // Displaying only the txs that spend non-spent vaults. This way
            // we're hiding to the user spend transactions that can't be spent
            // anymore, as one of the inputs got spent.
            // FIXME: this might be a bug? I absolutely don't remember why I introduced
            // this check in the first place
            txs.into_iter()
                .filter(|tx| {
                    tx.deposit_outpoints
                        .iter()
                        .all(|outpoint| self.spendable_outpoints.get(outpoint).is_some())
                })
                .collect()
        };

        self.spend_txs_item = if self.loading_vaults {
            // Let's avoid displaying txs if I don't have the vaults
            vec![]
        } else {
            self.spend_txs
                .clone()
                .into_iter()
                .map(|s| {
                    (
                        if self.loading_vaults {
                            0
                        } else {
                            // Amounts of the vaults being spent
                            s.deposit_outpoints.iter().fold(0, |acc, x| {
                                acc + *self.spendable_outpoints.get(x).expect("Must be spendable")
                            })
                        },
                        s,
                    )
                })
                .map(|(vaults_amount, s)| SpendTransactionListItem::new(s, vaults_amount))
                .collect()
        };
    }

    pub fn on_spend_tx_select<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
        psbt: Psbt,
    ) -> Command<Message> {
        if let Some(selected) = &self.selected_spend_tx {
            if selected.psbt.global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid() {
                self.selected_spend_tx = None;
                return Command::none();
            }
        }

        if self
            .spend_txs
            .iter()
            .any(|item| item.psbt.global.unsigned_tx.txid() == psbt.global.unsigned_tx.txid())
        {
            let selected_spend_tx = SpendTransactionState::new(ctx, psbt);
            let cmd = selected_spend_tx.load(ctx);
            self.selected_spend_tx = Some(selected_spend_tx);
            return cmd;
        };
        Command::none()
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        let (active_funds, inactive_funds) =
            vaults.iter().fold((0, 0), |acc, vault| match vault.status {
                VaultStatus::Active => (acc.0 + vault.amount, acc.1),
                VaultStatus::Funded | VaultStatus::Securing | VaultStatus::Secured => {
                    (acc.0, acc.1 + vault.amount)
                }
                _ => (acc.0, acc.1),
            });
        self.active_funds = active_funds;
        self.inactive_funds = inactive_funds;

        self.spendable_outpoints = vaults
            .iter()
            .filter_map(|vlt| {
                if vlt.status == VaultStatus::Active {
                    Some((vlt.outpoint(), vlt.amount))
                } else {
                    None
                }
            })
            .collect();

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

        self.loading_vaults = false;

        // The spendable outpoints changed, let's update the spend txs
        self.update_spend_txs(self.spend_txs.clone());
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
}

impl<C: Client + Send + Sync + 'static> State<C> for ManagerHomeState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Select(psbt)) => {
                return self.on_spend_tx_select(ctx, psbt);
            }
            Message::SpendTx(msg) => {
                if let Some(tx) = &mut self.selected_spend_tx {
                    return tx.update(ctx, Message::SpendTx(msg));
                }
            }
            Message::SpendTransactions(res) => match res {
                Ok(txs) => self.update_spend_txs(txs),
                Err(e) => self.warning = Error::from(e).into(),
            },
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
            Message::BlockHeight(b) => match b {
                Ok(height) => {
                    self.blockheight = height;
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                }
            },
            Message::HistoryEvents(res) => match res {
                Ok(events) => {
                    self.latest_events = events.into_iter().map(HistoryEventState::new).collect();
                }
                Err(e) => {
                    self.warning = Error::from(e).into();
                }
            },
            _ => {}
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Some(v) = &self.selected_spend_tx {
            return v.sub();
        }
        Subscription::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        if let Some(v) = &mut self.selected_vault {
            return v.view(ctx);
        }

        if let Some(tx) = &mut self.selected_spend_tx {
            return tx.view(ctx);
        }

        self.view.view(
            ctx,
            self.warning.as_ref(),
            self.spend_txs_item
                .iter_mut()
                .map(|tx| tx.view(ctx).map(Message::SpendTx))
                .collect(),
            self.moving_vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            self.latest_events.iter_mut().map(|e| e.view(ctx)).collect(),
            self.active_funds,
            self.inactive_funds,
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
                list_vaults(ctx.revaultd.clone(), Some(&VaultStatus::CURRENT), None),
                Message::Vaults,
            ),
            Command::perform(
                list_spend_txs(
                    ctx.revaultd.clone(),
                    Some(&[model::SpendTxStatus::NonFinal]),
                ),
                Message::SpendTransactions,
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

impl<C: Client + Send + Sync + 'static> From<ManagerHomeState> for Box<dyn State<C>> {
    fn from(s: ManagerHomeState) -> Box<dyn State<C>> {
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

impl<C: Client + Send + Sync + 'static> State<C> for ManagerSendState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
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

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::CreateSendTransaction(state) => state.view(ctx),
            Self::ImportSendTransaction(state) => state.view(ctx),
            Self::SendTransactionDetail(state) => state.view(ctx),
        }
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
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

impl<C: Client + Send + Sync + 'static> State<C> for ManagerImportSendTransactionState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
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

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        self.view.view(
            ctx,
            &self.psbt_input,
            self.psbt_imported.as_ref(),
            self.warning.as_ref(),
        )
    }

    fn load(&self, _ctx: &Context<C>) -> Command<Message> {
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
    feerate: Option<u32>,
    psbt: Option<(Psbt, u32)>,
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

impl<C: Client + Send + Sync + 'static> State<C> for ManagerCreateSendTransactionState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match message {
            Message::SpendTransaction(res) => {
                self.processing = false;
                match res {
                    Ok(tx) => {
                        self.psbt = Some((tx.spend_tx, tx.feerate));
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

                let outputs: HashMap<String, u64> = self
                    .outputs
                    .iter()
                    .map(|output| (output.address.value.clone(), output.amount().unwrap()))
                    .collect();

                return Command::perform(
                    get_spend_tx(ctx.revaultd.clone(), inputs, outputs, self.feerate.unwrap()),
                    Message::SpendTransaction,
                );
            }
            Message::SpendTx(SpendTxMessage::FeerateEdited(feerate)) => {
                if let Ok(f) = feerate.parse::<u32>() {
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
                        .update(msg)
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
                                &ctx.revaultd
                                    .config
                                    .managers_xpubs()
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

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
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
                    self.warning.as_ref(),
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

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(ctx.revaultd.clone(), Some(&[VaultStatus::Active]), None),
            Message::Vaults,
        )])
    }
}

impl<C: Client + Send + Sync + 'static> From<ManagerSendState> for Box<dyn State<C>> {
    fn from(s: ManagerSendState) -> Box<dyn State<C>> {
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
                if !self.address.value.is_empty() {
                    self.address.valid = bitcoin::Address::from_str(&self.address.value).is_ok();
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

    pub fn view<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
    ) -> Element<InputMessage> {
        manager_send_input_view(
            ctx,
            &self.vault.outpoint(),
            &self.vault.amount,
            self.selected,
        )
    }

    pub fn update(&mut self, msg: InputMessage) {
        match msg {
            InputMessage::Selected(selected) => self.selected = selected,
        }
    }
}
