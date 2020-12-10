use std::sync::Arc;
use std::time::Duration;

use iced::{time, Command, Element, Subscription};

use super::{util::Watch, State};
use crate::revaultd::{model::Vault, RevaultD, RevaultDError};
use crate::ui::{
    error::Error,
    message::{Message, MessageMenu},
    view::manager::{ManagerHistoryView, ManagerHomeView, ManagerView},
};

#[derive(Debug, Clone)]
pub struct ManagerState {
    revaultd: Arc<RevaultD>,
    view: ManagerView,

    blockheight: Watch<u64>,
    vaults: Watch<Vec<Vault>>,
    warning: Watch<Error>,
}

impl ManagerState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        ManagerState {
            revaultd,
            view: ManagerView::Home(ManagerHomeView::new()),
            blockheight: Watch::None,
            vaults: Watch::None,
            warning: Watch::None,
        }
    }

    pub fn on_tick(&mut self) -> Command<Message> {
        if !self.blockheight.is_recent(Duration::from_secs(5)) {
            return Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight);
        }

        if !self.warning.is_none() && !self.warning.is_recent(Duration::from_secs(30)) {
            self.warning.reset()
        }

        Command::none()
    }

    pub fn balance(&self) -> u64 {
        let mut amt: u64 = 0;
        if let Watch::Some { ref value, .. } = self.vaults {
            for vlt in value {
                amt += vlt.amount
            }
        }
        amt
    }
}

impl State for ManagerState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Menu(m) => match m {
                MessageMenu::Home => self.view = ManagerView::Home(ManagerHomeView::new()),
                MessageMenu::History => self.view = ManagerView::History(ManagerHistoryView::new()),
            },
            Message::Tick(_) => return self.on_tick(),
            Message::Vaults(res) => match res {
                Ok(vaults) => self.vaults = vaults.into(),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::BlockHeight(b) => match b {
                Ok(height) => self.blockheight = height.into(),
                Err(e) => self.warning = Error::from(e).into(),
            },
            _ => {}
        };
        Command::none()
    }

    fn view<'a>(&'a mut self) -> Element<Message> {
        let b = self.balance();
        match &mut self.view {
            ManagerView::History(v) => v.view(),
            ManagerView::Home(v) => v.view(
                b,
                self.warning.as_ref().into(),
                self.blockheight.as_ref().into(),
                self.vaults.as_ref().into(),
            ),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_secs(1)).map(Message::Tick)
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(list_vaults(self.revaultd.clone()), Message::Vaults),
        ])
    }
}

async fn get_blockheight(revaultd: Arc<RevaultD>) -> Result<u64, RevaultDError> {
    revaultd.get_info().map(|res| res.blockheight)
}

async fn list_vaults(revaultd: Arc<RevaultD>) -> Result<Vec<Vault>, RevaultDError> {
    revaultd.list_vaults().map(|res| res.vaults)
}

impl From<ManagerState> for Box<dyn State> {
    fn from(s: ManagerState) -> Box<dyn State> {
        Box::new(s)
    }
}
