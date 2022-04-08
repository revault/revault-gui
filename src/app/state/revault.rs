use iced::{Command, Element};

use bitcoin::OutPoint;

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::Message,
        state::State,
        view::{
            LoadingModal, RevaultSelectVaultsView, RevaultSuccessView, RevaultVaultListItemView,
        },
    },
    daemon::model::{outpoint, Vault, VaultStatus},
};

#[derive(Debug)]
pub enum RevaultVaultsState {
    Loading {
        fail: Option<Error>,
        view: LoadingModal,
    },
    SelectVaults {
        total: u64,
        vaults: Vec<RevaultVaultListItem>,
        view: RevaultSelectVaultsView,
        processing: bool,
        warning: Option<Error>,
    },
    Success {
        vaults: Vec<Vault>,
        view: RevaultSuccessView,
    },
}

impl Default for RevaultVaultsState {
    fn default() -> Self {
        RevaultVaultsState::Loading {
            fail: None,
            view: LoadingModal::default(),
        }
    }
}

impl State for RevaultVaultsState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::Vaults(res) = message {
                    match res {
                        Ok(vaults) => {
                            *self = Self::SelectVaults {
                                total: vaults.iter().map(|v| v.amount.as_sat()).sum::<u64>(),
                                vaults: vaults.into_iter().map(RevaultVaultListItem::new).collect(),
                                view: RevaultSelectVaultsView::default(),
                                warning: None,
                                processing: false,
                            };
                        }
                        Err(e) => *fail = Some(e.into()),
                    };
                }
            }
            Self::SelectVaults {
                vaults,
                processing,
                warning,
                ..
            } => match message {
                Message::SelectVault(selected_outpoint) => {
                    if !*processing {
                        for vlt in vaults.iter_mut() {
                            if outpoint(&vlt.vault) == selected_outpoint {
                                vlt.selected = !vlt.selected
                            }
                        }
                    }
                }
                Message::Revault => {
                    *processing = true;
                    let revaultd = ctx.revaultd.clone();
                    let outpoints: Vec<OutPoint> = vaults
                        .iter()
                        .filter_map(|v| {
                            if v.selected {
                                Some(outpoint(&v.vault))
                            } else {
                                None
                            }
                        })
                        .collect();
                    return Command::perform(
                        async move {
                            for outpoint in outpoints {
                                revaultd.revault(&outpoint)?;
                            }
                            Ok(())
                        },
                        Message::Revaulted,
                    );
                }
                Message::Revaulted(res) => match res {
                    Ok(()) => {
                        *self = Self::Success {
                            vaults: vaults
                                .iter()
                                .filter_map(|v| {
                                    if v.selected {
                                        Some(v.vault.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                            view: RevaultSuccessView::default(),
                        }
                    }
                    Err(e) => *warning = Some(e.into()),
                },
                _ => {}
            },
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref(), Menu::Home),
            Self::Success { vaults, view } => view.view(ctx, vaults.len()),
            Self::SelectVaults {
                view,
                vaults,
                total,
                warning,
                processing,
            } => view.view(
                ctx,
                vaults
                    .iter()
                    .filter(|v| v.selected)
                    .fold((0, 0), |(count, total), v| {
                        (count + 1, total + v.vault.amount.as_sat())
                    }),
                vaults.iter_mut().map(|vault| vault.view(ctx)).collect(),
                *total,
                warning.as_ref(),
                *processing,
            ),
        }
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        let revaultd = ctx.revaultd.clone();
        Command::perform(
            async move {
                revaultd.list_vaults(
                    Some(&[VaultStatus::Unvaulting, VaultStatus::Unvaulted]),
                    None,
                )
            },
            Message::Vaults,
        )
    }
}

impl From<RevaultVaultsState> for Box<dyn State> {
    fn from(s: RevaultVaultsState) -> Box<dyn State> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct RevaultVaultListItem {
    vault: Vault,
    selected: bool,
    view: RevaultVaultListItemView,
}

impl RevaultVaultListItem {
    pub fn new(vault: Vault) -> Self {
        Self {
            vault,
            selected: false,
            view: RevaultVaultListItemView::default(),
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(ctx, &self.vault, self.selected)
    }
}
