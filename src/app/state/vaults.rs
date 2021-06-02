use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, list_vaults},
    vault::{Vault, VaultListItem},
    State,
};

use crate::revaultd::{model, model::VaultStatus, RevaultD};

use crate::app::{
    error::Error,
    message::{Message, VaultFilterMessage, VaultMessage},
    view::{vault::VaultListItemView, Context, VaultsView},
};

#[derive(Debug)]
pub struct VaultsState {
    revaultd: Arc<RevaultD>,
    view: VaultsView,

    blockheight: u64,

    vault_status_filter: &'static [VaultStatus],
    vaults: Vec<VaultListItem<VaultListItemView>>,
    selected_vault: Option<Vault>,

    warning: Option<Error>,

    /// loading is true until Message::Vaults is handled
    loading: bool,
}

impl VaultsState {
    pub fn new(revaultd: Arc<RevaultD>) -> Self {
        VaultsState {
            revaultd,
            view: VaultsView::new(),
            blockheight: 0,
            vault_status_filter: &VaultStatus::CURRENT,
            vaults: Vec::new(),
            selected_vault: None,
            warning: None,
            loading: true,
        }
    }

    pub fn update_vaults(&mut self, vaults: Vec<model::Vault>) {
        self.vaults = vaults.into_iter().map(VaultListItem::new).collect();
        self.loading = false;
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

impl State for VaultsState {
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
            Message::FilterVaults(VaultFilterMessage::Status(statuses)) => {
                self.loading = true;
                self.vault_status_filter = statuses;
                return Command::perform(
                    list_vaults(self.revaultd.clone(), Some(self.vault_status_filter), None),
                    Message::Vaults,
                );
            }
            Message::BlockHeight(b) => match b {
                Ok(height) => self.blockheight = height,
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
            self.warning.as_ref(),
            self.vaults.iter_mut().map(|v| v.view(ctx)).collect(),
            self.vault_status_filter,
            self.loading,
        )
    }

    fn load(&self) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(self.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                list_vaults(self.revaultd.clone(), Some(self.vault_status_filter), None),
                Message::Vaults,
            ),
        ])
    }
}

impl From<VaultsState> for Box<dyn State> {
    fn from(s: VaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}
