use std::sync::Arc;
use std::time::Duration;

use iced::{time, Command, Element, Subscription};

use crate::revaultd::{
    model::{Vault, VaultStatus, VaultTransactions},
    RevaultD,
};

use crate::ui::{
    error::Error,
    message::Message,
    state::{cmd::get_blockheight, util::Watch, State},
    view::{Context, StakeholderHomeView, StakeholderNetworkView},
};

#[derive(Debug)]
pub struct StakeholderHomeState {
    revaultd: Arc<RevaultD>,

    balance: (u64, u64),
    view: StakeholderHomeView,
}

impl StakeholderHomeState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        StakeholderHomeState {
            revaultd,
            view: StakeholderHomeView::new(),
            balance: (0, 0),
        }
    }

    pub fn calculate_balance(&mut self, vaults: &Vec<(Vault, VaultTransactions)>) {
        let mut active_amount: u64 = 0;
        let mut inactive_amount: u64 = 0;
        for (vault, _) in vaults {
            match vault.status {
                VaultStatus::Active => active_amount += vault.amount,
                VaultStatus::Secured | VaultStatus::Funded | VaultStatus::Unconfirmed => {
                    inactive_amount += vault.amount
                }
                _ => {}
            }
        }

        self.balance = (active_amount, inactive_amount);
    }
}

impl State for StakeholderHomeState {
    fn update(&mut self, _message: Message) -> Command<Message> {
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, None, Vec::new(), &self.balance)
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![])
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
