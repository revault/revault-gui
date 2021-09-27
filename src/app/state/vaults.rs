use std::convert::From;

use iced::{Command, Element, Subscription};

use super::{
    cmd::{get_blockheight, list_vaults},
    vault::{Vault, VaultListItem},
    State,
};

use crate::daemon::{client::Client, model, model::VaultStatus};

use crate::app::{
    context::Context,
    error::Error,
    message::{Message, VaultFilterMessage},
    view::{vault::VaultListItemView, VaultsView},
};

#[derive(Debug)]
pub struct VaultsState {
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
    pub fn new() -> Self {
        VaultsState {
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

    pub fn on_vault_select<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
        outpoint: String,
    ) -> Command<Message> {
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
            let cmd = selected_vault.load(ctx.revaultd.clone());
            self.selected_vault = Some(selected_vault);
            return cmd.map(Message::Vault);
        };
        Command::none()
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for VaultsState {
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
            Message::FilterVaults(VaultFilterMessage::Status(statuses)) => {
                self.loading = true;
                self.vault_status_filter = statuses;
                return Command::perform(
                    list_vaults(ctx.revaultd.clone(), Some(self.vault_status_filter), None),
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

    fn subscription(&self) -> Subscription<Message> {
        if let Some(v) = &self.selected_vault {
            return v.subscription().map(Message::Vault);
        }
        Subscription::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
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

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        Command::batch(vec![
            Command::perform(get_blockheight(ctx.revaultd.clone()), Message::BlockHeight),
            Command::perform(
                list_vaults(ctx.revaultd.clone(), Some(self.vault_status_filter), None),
                Message::Vaults,
            ),
        ])
    }
}

impl<C: Client + Send + Sync + 'static> From<VaultsState> for Box<dyn State<C>> {
    fn from(s: VaultsState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
