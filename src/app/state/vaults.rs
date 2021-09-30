use std::convert::From;

use iced::{Command, Element, Subscription};

use super::{
    cmd::list_vaults,
    vault::{Vault, VaultListItem},
    State,
};

use crate::daemon::{client::Client, model, model::VaultStatus};

use crate::app::{
    context::Context,
    error::Error,
    message::{Message, VaultFilterMessage},
    view::{vault::VaultListItemView, LoadingDashboard, VaultsView},
};

#[derive(Debug)]
pub enum VaultsState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    Loaded {
        selected_vault: Option<Vault>,
        vault_status_filter: &'static [VaultStatus],
        vaults: Vec<VaultListItem<VaultListItemView>>,
        warning: Option<Error>,

        view: VaultsView,
    },
}

impl VaultsState {
    pub fn new() -> Self {
        Self::Loading {
            view: LoadingDashboard::new(),
            fail: None,
        }
    }

    pub fn update_vaults(&mut self, vlts: Vec<model::Vault>) {
        match self {
            Self::Loading { .. } => {
                let vaults = vlts.into_iter().map(VaultListItem::new).collect();
                *self = Self::Loaded {
                    view: VaultsView::new(),
                    vault_status_filter: &VaultStatus::CURRENT,
                    vaults,
                    selected_vault: None,
                    warning: None,
                };
            }
            Self::Loaded {
                vaults, warning, ..
            } => {
                *vaults = vlts.into_iter().map(VaultListItem::new).collect();
                *warning = None;
            }
        }
    }

    pub fn on_error(&mut self, error: Error) {
        match self {
            Self::Loading { fail, .. } => {
                *fail = Some(error);
            }
            Self::Loaded { warning, .. } => {
                *warning = Some(error);
            }
        }
    }

    pub fn on_vault_select<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
        outpoint: String,
    ) -> Command<Message> {
        if let Self::Loaded {
            selected_vault,
            vaults,
            ..
        } = self
        {
            if let Some(selected) = selected_vault {
                if selected.vault.outpoint() == outpoint {
                    *selected_vault = None;
                    return Command::none();
                }
            }

            if let Some(selected) = vaults.iter().find(|vlt| vlt.vault.outpoint() == outpoint) {
                let vault = Vault::new(selected.vault.clone());
                let cmd = vault.load(ctx.revaultd.clone());
                *selected_vault = Some(vault);
                return cmd.map(Message::Vault);
            };
        };
        Command::none()
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for VaultsState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match message {
            Message::Reload => return self.load(ctx),
            Message::Vaults(res) => match res {
                Ok(vaults) => self.update_vaults(vaults),
                Err(e) => self.on_error(Error::from(e)),
            },
            Message::SelectVault(outpoint) => return self.on_vault_select(ctx, outpoint),
            Message::Vault(msg) => {
                if let Self::Loaded { selected_vault, .. } = self {
                    if let Some(selected) = selected_vault {
                        return selected.update(ctx, msg).map(Message::Vault);
                    }
                }
            }
            Message::FilterVaults(VaultFilterMessage::Status(statuses)) => {
                if let Self::Loaded {
                    vault_status_filter,
                    ..
                } = self
                {
                    *vault_status_filter = statuses;
                    return Command::perform(
                        list_vaults(ctx.revaultd.clone(), Some(vault_status_filter), None),
                        Message::Vaults,
                    );
                }
            }
            _ => {}
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if let Self::Loaded { selected_vault, .. } = self {
            if let Some(v) = selected_vault {
                return v.subscription().map(Message::Vault);
            }
        }
        Subscription::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref()),
            Self::Loaded {
                selected_vault,
                vaults,
                vault_status_filter,
                view,
                warning,
                ..
            } => {
                if let Some(v) = selected_vault {
                    return v.view(ctx);
                }
                view.view(
                    ctx,
                    warning.as_ref(),
                    vaults.iter_mut().map(|v| v.view(ctx)).collect(),
                    vault_status_filter,
                )
            }
        }
    }

    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        match self {
            Self::Loading { .. } => Command::batch(vec![Command::perform(
                list_vaults(ctx.revaultd.clone(), Some(&VaultStatus::CURRENT), None),
                Message::Vaults,
            )]),
            Self::Loaded {
                vault_status_filter,
                ..
            } => Command::batch(vec![Command::perform(
                list_vaults(ctx.revaultd.clone(), Some(vault_status_filter), None),
                Message::Vaults,
            )]),
        }
    }
}

impl<C: Client + Send + Sync + 'static> From<VaultsState> for Box<dyn State<C>> {
    fn from(s: VaultsState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}
