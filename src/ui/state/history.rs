use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, list_vaults},
    vault::{Vault, VaultListItem},
    State,
};

use crate::revaultd::{model, RevaultD};

use crate::ui::{
    error::Error,
    message::{Message, VaultMessage},
    view::{vault::VaultListItemView, Context, HistoryView},
};

#[derive(Debug)]
pub struct HistoryState {
    revaultd: Arc<RevaultD>,
    view: HistoryView,

    blockheight: u64,
    warning: Option<Error>,

    vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,
}

impl HistoryState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        HistoryState {
            revaultd,
            view: HistoryView::new(),
            blockheight: 0,
            vaults: Vec::new(),
            warning: None,
            selected_vault: None,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.vaults = vaults
            .into_iter()
            .map(|vlt| VaultListItem::new(vlt))
            .collect();
    }

    pub fn on_vault_select(&mut self, outpoint: String) -> Command<Message> {
        if let Some(selected) = &self.selected_vault {
            if selected.vault.outpoint() == outpoint {
                self.selected_vault = None;
                return Command::none();
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
            return cmd.map(move |msg| Message::Vault(outpoint.clone(), msg));
        };
        Command::none()
    }
}

impl State for HistoryState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::Vault(outpoint, VaultMessage::Select) => {
                return self.on_vault_select(outpoint)
            }
            Message::Vault(outpoint, msg) => {
                if let Some(selected) = &mut self.selected_vault {
                    if selected.vault.outpoint() == outpoint {
                        return selected
                            .update(self.revaultd.clone(), msg)
                            .map(move |msg| Message::Vault(outpoint.clone(), msg));
                    }
                }
            }
            Message::BlockHeight(b) => match b {
                Ok(height) => self.blockheight = height.into(),
                Err(e) => self.warning = Error::from(e).into(),
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
            self.warning.as_ref().into(),
            self.vaults.iter_mut().map(|v| v.view(ctx)).collect(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(list_vaults(self.revaultd.clone(), None), Message::Vaults),
        ])
    }
}

impl From<HistoryState> for Box<dyn State> {
    fn from(s: HistoryState) -> Box<dyn State> {
        Box::new(s)
    }
}
