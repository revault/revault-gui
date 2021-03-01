use std::sync::Arc;

use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{Command, Element};

use crate::revault::TransactionKind;

use crate::revaultd::{
    model::{RevocationTransactions, Vault, VaultStatus, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::{DepositMessage, Message, SignMessage},
    state::{
        cmd::{
            get_blockheight, get_revocation_txs, list_vaults, list_vaults_with_transactions,
            set_revocation_txs,
        },
        sign::SignState,
        State,
    },
    view::{
        stakeholder::{stakeholder_deposit_pending, stakeholder_deposit_signed},
        Context, StakeholderACKDepositView, StakeholderACKFundsView, StakeholderHomeView,
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
}

impl StakeholderHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderHomeState {
            revaultd,
            warning: None,
            view: StakeholderHomeView::new(),
            unsecured_fund_balance: 0,
            balance: (0, 0),
        }
    }

    fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.calculate_balance(&vaults);
    }

    fn calculate_balance(&mut self, vaults: &[(Vault, VaultTransactions)]) {
        let mut active_amount: u64 = 0;
        let mut inactive_amount: u64 = 0;
        let mut unsecured_amount: u64 = 0;
        for (vault, _) in vaults {
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
        if let Message::VaultsWithTransactions(res) = message {
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
            Vec::new(),
            &self.balance,
            &self.unsecured_fund_balance,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                list_vaults_with_transactions(self.revaultd.clone()),
                Message::VaultsWithTransactions,
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
pub struct StakeholderACKFundsState {
    revaultd: Arc<RevaultD>,
    warning: Option<Error>,

    balance: u64,
    deposits: Vec<Deposit>,
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
        }
    }

    fn start_signing_deposit(&mut self, index: usize) -> Command<Message> {
        if let Some(Deposit::Pending { vault }) = self.deposits.get_mut(index) {
            return Command::perform(
                get_revocation_txs(self.revaultd.clone(), vault.outpoint()),
                move |res| Message::Deposit(index, DepositMessage::RevocationTransactions(res)),
            );
        }
        Command::none()
    }

    fn update_deposits(&mut self, vaults: Vec<Vault>) -> Command<Message> {
        self.calculate_balance(&vaults);
        self.deposits = vaults.into_iter().map(Deposit::new).collect();
        self.start_signing_deposit(0)
    }

    fn calculate_balance(&mut self, vaults: &[Vault]) {
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
            Message::Deposit(i, msg) => {
                if let Some(deposit) = self.deposits.get_mut(i) {
                    let cmd = deposit
                        .update(self.revaultd.clone(), msg)
                        .map(move |msg| Message::Deposit(i, msg));
                    if deposit.signed() {
                        return Command::batch(vec![cmd, self.start_signing_deposit(i + 1)]);
                    }
                    return cmd;
                }
                Command::none()
            }
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_deposits(vaults),
                Err(e) => {
                    self.warning = Error::from(e).into();
                    Command::none()
                }
            },
            _ => Command::none(),
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            self.deposits
                .iter_mut()
                .enumerate()
                .map(|(i, v)| {
                    v.view(ctx).map(move |msg| {
                        if let DepositMessage::Sign(SignMessage::Clipboard(psbt)) = msg {
                            return Message::Clipboard(psbt);
                        }
                        Message::Deposit(i, msg)
                    })
                })
                .collect(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![Command::perform(
            list_vaults(self.revaultd.clone()),
            Message::Vaults,
        )])
    }
}

#[derive(Debug)]
pub enum Deposit {
    Signed {
        vault: Vault,
    },
    Signing {
        warning: Option<String>,
        vault: Vault,
        emergency_tx: (Psbt, bool),
        emergency_unvault_tx: (Psbt, bool),
        cancel_tx: (Psbt, bool),
        view: StakeholderACKDepositView,
        signer: SignState,
    },
    Pending {
        vault: Vault,
    },
}

impl Deposit {
    fn new(vault: Vault) -> Self {
        Deposit::Pending { vault }
    }

    fn signed(&self) -> bool {
        matches!(self, Self::Signed { .. })
    }

    fn update(
        &mut self,
        revaultd: Arc<RevaultD>,
        message: DepositMessage,
    ) -> Command<DepositMessage> {
        match message {
            DepositMessage::Retry => {
                if let Deposit::Signing {
                    warning,
                    vault,
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                    ..
                } = self
                {
                    *warning = None;
                    return Command::perform(
                        set_revocation_txs(
                            revaultd,
                            vault.outpoint(),
                            emergency_tx.0.clone(),
                            emergency_unvault_tx.0.clone(),
                            cancel_tx.0.clone(),
                        ),
                        DepositMessage::Signed,
                    );
                }
            }
            DepositMessage::Signed(res) => {
                if let Deposit::Signing { vault, warning, .. } = self {
                    if let Err(e) = res {
                        *warning = Some(format!("Error: {}", e));
                    } else {
                        *self = Deposit::Signed {
                            vault: vault.clone(),
                        };
                    }
                }
            }
            DepositMessage::RevocationTransactions(res) => {
                if let Ok(txs) = res {
                    self.signing(txs)
                }
            }
            DepositMessage::Sign(msg) => {
                if let Deposit::Signing {
                    signer,
                    emergency_tx,
                    emergency_unvault_tx,
                    cancel_tx,
                    vault,
                    ..
                } = self
                {
                    signer.update(msg);
                    if let Some(psbt) = &signer.signed_psbt {
                        match signer.transaction_kind {
                            TransactionKind::Emergency => {
                                *emergency_tx = (psbt.clone(), true);
                                *signer = SignState::new(
                                    emergency_unvault_tx.0.clone(),
                                    TransactionKind::EmergencyUnvault,
                                );
                            }
                            TransactionKind::EmergencyUnvault => {
                                *emergency_unvault_tx = (psbt.clone(), true);
                                *signer =
                                    SignState::new(cancel_tx.0.clone(), TransactionKind::Cancel);
                            }
                            TransactionKind::Cancel => {
                                *cancel_tx = (psbt.clone(), true);
                                return Command::perform(
                                    set_revocation_txs(
                                        revaultd,
                                        vault.outpoint(),
                                        emergency_tx.0.clone(),
                                        emergency_unvault_tx.0.clone(),
                                        cancel_tx.0.clone(),
                                    ),
                                    DepositMessage::Signed,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Command::none()
    }

    fn signing(&mut self, txs: RevocationTransactions) {
        if let Deposit::Pending { vault } = self {
            let signer = SignState::new(txs.emergency_tx.clone(), TransactionKind::Emergency);
            *self = Deposit::Signing {
                warning: None,
                vault: vault.to_owned(),
                view: StakeholderACKDepositView::new(),
                emergency_tx: (txs.emergency_tx, false),
                emergency_unvault_tx: (txs.emergency_unvault_tx, false),
                cancel_tx: (txs.cancel_tx, false),
                signer,
            };
        }
    }

    fn view(&mut self, ctx: &Context) -> Element<DepositMessage> {
        match self {
            Self::Signed { vault } => stakeholder_deposit_signed(ctx, vault),
            Self::Pending { vault } => stakeholder_deposit_pending(ctx, vault),
            Self::Signing {
                warning,
                vault,
                view,
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                signer,
            } => view.view(
                ctx,
                warning.as_ref(),
                vault,
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                signer.view(ctx).map(DepositMessage::Sign),
            ),
        }
    }
}

impl From<StakeholderACKFundsState> for Box<dyn State> {
    fn from(s: StakeholderACKFundsState) -> Box<dyn State> {
        Box::new(s)
    }
}
