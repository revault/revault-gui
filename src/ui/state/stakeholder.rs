use std::sync::Arc;
use std::time::Duration;

use bitcoin::util::psbt::PartiallySignedTransaction;

use iced::{time, Command, Element, Subscription};

use crate::revault::TransactionKind;

use crate::revaultd::{
    model::{RevocationTransactions, Vault, VaultStatus, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::{DepositMessage, Message},
    state::{
        cmd::{get_blockheight, get_revocation_txs, list_vaults, list_vaults_with_transactions},
        sign::SignState,
        util::Watch,
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
    warning: Watch<Error>,

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
            warning: Watch::None,
            view: StakeholderHomeView::new(),
            unsecured_fund_balance: 0,
            balance: (0, 0),
        }
    }

    fn update_vaults(&mut self, vaults: Vec<(Vault, VaultTransactions)>) {
        self.calculate_balance(&vaults);
    }

    fn calculate_balance(&mut self, vaults: &Vec<(Vault, VaultTransactions)>) {
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
        match message {
            Message::VaultsWithTransactions(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            _ => {}
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

    blockheight: Watch<u64>,
    warning: Watch<Error>,

    view: StakeholderNetworkView,
}

impl StakeholderNetworkState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderNetworkState {
            revaultd,
            blockheight: Watch::None,
            warning: Watch::None,
            view: StakeholderNetworkView::new(),
        }
    }

    pub fn on_tick(&mut self) -> Command<Message> {
        if !self.blockheight.is_recent(Duration::from_secs(5)) {
            return Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight);
        }

        if self.warning.is_some() && !self.warning.is_recent(Duration::from_secs(30)) {
            self.warning.reset()
        }

        Command::none()
    }
}

impl State for StakeholderNetworkState {
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

impl From<StakeholderNetworkState> for Box<dyn State> {
    fn from(s: StakeholderNetworkState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct StakeholderACKFundsState {
    revaultd: Arc<RevaultD>,
    warning: Watch<Error>,

    balance: u64,
    deposits: Vec<Deposit>,
    view: StakeholderACKFundsView,
}

impl StakeholderACKFundsState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderACKFundsState {
            revaultd,
            warning: Watch::None,
            deposits: Vec::new(),
            view: StakeholderACKFundsView::new(),
            balance: 0,
        }
    }

    fn update_deposits(&mut self, vaults: Vec<Vault>) -> Command<Message> {
        self.calculate_balance(&vaults);
        self.deposits = vaults.into_iter().map(|vlt| Deposit::new(vlt)).collect();
        if let Some(Deposit::Pending { vault }) = self.deposits.first() {
            return Command::perform(
                get_revocation_txs(self.revaultd.clone(), vault.outpoint()),
                |res| Message::Deposit(0, DepositMessage::RevocationTransactions(res)),
            );
        }
        Command::none()
    }

    fn calculate_balance(&mut self, vaults: &Vec<Vault>) {
        let mut balance: u64 = 0;
        for vault in vaults {
            match vault.status {
                VaultStatus::Funded => {
                    balance += vault.amount;
                }
                _ => {}
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
                    deposit.update(msg);
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
                .map(|(i, v)| v.view(ctx).map(move |msg| Message::Deposit(i, msg)))
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
        vault: Vault,
        emergency_tx: (PartiallySignedTransaction, bool),
        emergency_unvault_tx: (PartiallySignedTransaction, bool),
        cancel_tx: (PartiallySignedTransaction, bool),
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

    fn update(&mut self, message: DepositMessage) {
        match message {
            DepositMessage::RevocationTransactions(res) => {
                if let Ok(txs) = res {
                    self.signing(txs)
                }
            }
            DepositMessage::Sign(msg) => {
                if let Deposit::Signing { signer, .. } = self {
                    signer.update(msg);
                }
            }
        }
    }

    fn signing(&mut self, txs: RevocationTransactions) {
        if let Deposit::Pending { vault } = self {
            let signer = SignState::new(txs.emergency_tx.clone(), TransactionKind::Emergency);
            *self = Deposit::Signing {
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
                vault,
                view,
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                signer,
            } => view.view(
                ctx,
                vault,
                emergency_tx,
                emergency_unvault_tx,
                cancel_tx,
                signer.view(ctx).map(move |msg| DepositMessage::Sign(msg)),
            ),
        }
    }
}

impl From<StakeholderACKFundsState> for Box<dyn State> {
    fn from(s: StakeholderACKFundsState) -> Box<dyn State> {
        Box::new(s)
    }
}
