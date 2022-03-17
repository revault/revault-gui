use std::convert::From;

use bitcoin::OutPoint;
use iced::{Command, Element};

use super::{
    cmd::list_vaults,
    vault::{Vault, VaultListItem},
    State,
};

use crate::daemon::{
    model,
    model::{
        outpoint, VaultStatus, CURRENT_VAULT_STATUSES, MOVED_VAULT_STATUSES, MOVING_VAULT_STATUSES,
    },
};

use crate::app::{
    context::Context,
    error::Error,
    menu::VaultsMenu,
    message::{Message, VaultFilterMessage},
    view::{vault::VaultListItemView, LoadingDashboard, VaultsView},
};

#[derive(Debug)]
pub enum VaultsState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
        vault_status_filter: &'static [VaultStatus],
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
    pub fn new(menu: &VaultsMenu) -> Self {
        Self::Loading {
            view: LoadingDashboard::new(),
            fail: None,
            vault_status_filter: match menu {
                VaultsMenu::Current => &CURRENT_VAULT_STATUSES,
                VaultsMenu::Moving => &MOVING_VAULT_STATUSES,
                VaultsMenu::Moved => &MOVED_VAULT_STATUSES,
            },
        }
    }

    pub fn update_vaults(&mut self, vlts: Vec<model::Vault>) {
        match self {
            Self::Loading {
                vault_status_filter,
                ..
            } => {
                let vaults = vlts.into_iter().map(VaultListItem::new).collect();
                *self = Self::Loaded {
                    view: VaultsView::new(),
                    vault_status_filter,
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

    pub fn on_vault_select(
        &mut self,
        ctx: &Context,
        selected_outpoint: OutPoint,
    ) -> Command<Message> {
        if let Self::Loaded {
            selected_vault,
            vaults,
            ..
        } = self
        {
            if let Some(selected) = selected_vault {
                if outpoint(&selected.vault) == selected_outpoint {
                    *selected_vault = None;
                    return Command::none();
                }
            }

            if let Some(selected) = vaults
                .iter()
                .find(|vlt| outpoint(&vlt.vault) == selected_outpoint)
            {
                let vault = Vault::new(selected.vault.clone());
                let cmd = vault.load(ctx.revaultd.clone());
                *selected_vault = Some(vault);
                return cmd.map(Message::Vault);
            };
        };
        Command::none()
    }
}

impl State for VaultsState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
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

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { fail, view, .. } => view.view(ctx, fail.as_ref()),
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

    fn load(&self, ctx: &Context) -> Command<Message> {
        match self {
            Self::Loading {
                vault_status_filter,
                ..
            } => Command::batch(vec![Command::perform(
                list_vaults(ctx.revaultd.clone(), Some(vault_status_filter), None),
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

impl From<VaultsState> for Box<dyn State> {
    fn from(s: VaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}
