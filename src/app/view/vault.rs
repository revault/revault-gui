use chrono::NaiveDateTime;
use iced::{Align, Column, Container, Element, Length, Row};

use revault_ui::component::{badge, button, card, separation, text::Text};

use crate::app::{
    context::Context,
    error::Error,
    message::{Message, VaultMessage},
    view::{layout, warning::warn},
};

use crate::{
    daemon::{
        client::Client,
        model::{BroadcastedTransaction, Vault, VaultStatus, VaultTransactions},
    },
    revault::Role,
};

#[derive(Debug)]
pub struct VaultModal {
    copy_button: iced::button::State,
    modal: layout::Modal,
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            copy_button: iced::button::State::default(),
            modal: layout::Modal::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        vlt: &Vault,
        warning: Option<&Error>,
        panel_title: &str,
        panel: Element<'a, Message>,
    ) -> Element<'a, Message> {
        self.modal.view(
            ctx,
            warning,
            Container::new(
                Column::new()
                    .push(
                        Container::new(Text::new(&panel_title))
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .push(Container::new(vault(ctx, &mut self.copy_button, vlt)))
                    .push(Container::new(panel))
                    .max_width(1000)
                    .spacing(20),
            )
            .width(Length::Fill)
            .align_x(Align::Center),
            None,
            Message::SelectVault(vlt.outpoint()),
        )
    }
}

fn vault<'a, C: Client>(
    ctx: &Context<C>,
    copy_button: &'a mut iced::button::State,
    vlt: &Vault,
) -> Container<'a, Message> {
    card::simple(Container::new(
        Column::new()
            .push(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(vault_badge(&vlt))
                                .push(
                                    Column::new()
                                        .push(
                                            Row::new()
                                                .push(Text::new(&vlt.txid.to_string()).bold())
                                                .push(button::clipboard(
                                                    copy_button,
                                                    Message::Clipboard(vlt.txid.to_string()),
                                                ))
                                                .align_items(Align::Center),
                                        )
                                        .push(Text::new(&format!("{}", &vlt.status,))),
                                )
                                .align_items(Align::Center)
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(
                                    Text::new(&format!("{}", ctx.converter.converts(vlt.amount),))
                                        .bold(),
                                )
                                .push(Text::new(&ctx.converter.unit.to_string())),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            )
            .spacing(20),
    ))
}

/// This panel is the default view of a vault.
/// It lists the onchain transactions and suggest a call to action to
/// the user according to the vault status:
/// - If the status is FOUNDED, the panel asks the user to acknowledge the vault.
/// - If the status is SECURED, the panel asks the user to activate the vault.
/// - If the status is UNVAULTING, the panel asks the user to revault the vault.
#[derive(Debug)]
pub struct VaultOnChainTransactionsPanel {
    /// button used for ack fund panel or delegate vault panel or cancel spending panel
    /// depending of vault status.
    action_button: iced::button::State,
}

impl VaultOnChainTransactionsPanel {
    pub fn new() -> Self {
        VaultOnChainTransactionsPanel {
            action_button: iced::button::State::new(),
        }
    }
    pub fn view<C: Client>(
        &mut self,
        ctx: &Context<C>,
        vault: &Vault,
        txs: &VaultTransactions,
    ) -> Element<Message> {
        let mut col = Column::new().spacing(20);
        if ctx.role == Role::Stakeholder {
            match vault.status {
                VaultStatus::Unvaulted | VaultStatus::Unvaulting => {
                    col = col.push(card::white(Container::new(
                        Row::new()
                            .push(
                                Container::new(Text::new(
                                    "Funds are moving, do you want to revault them?",
                                ))
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    button::primary(
                                        &mut self.action_button,
                                        button::button_content(None, "Revault"),
                                    )
                                    .on_press(Message::Vault(VaultMessage::SelectRevault)),
                                )
                                .width(Length::Shrink),
                            )
                            .align_items(Align::Center),
                    )))
                }
                _ => {}
            };
        } else if vault.status == VaultStatus::Unvaulted || vault.status == VaultStatus::Unvaulting
        {
            col = col.push(card::white(Container::new(
                Row::new()
                    .push(
                        Container::new(Text::new("Funds are moving, do you want to revault them?"))
                            .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            button::primary(
                                &mut self.action_button,
                                button::button_content(None, "Revault"),
                            )
                            .on_press(Message::Vault(VaultMessage::SelectRevault)),
                        )
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            )))
        }

        col = col.push(Container::new(Text::new("Onchain transactions:").bold()));
        if let Some(tx) = &txs.spend {
            col = col.push(transaction(ctx, "Spend transaction", &tx));
        }
        if let Some(tx) = &txs.cancel {
            col = col.push(transaction(ctx, "Cancel transaction", &tx));
        }
        if let Some(tx) = &txs.unvault_emergency {
            col = col.push(transaction(ctx, "Unvault Emergency transaction", &tx));
        }
        if let Some(tx) = &txs.emergency {
            col = col.push(transaction(ctx, "Emergency transaction", &tx));
        }
        if let Some(tx) = &txs.unvault {
            col = col.push(transaction(ctx, "Unvault transaction", &tx));
        }
        col = col.push(transaction(ctx, "Deposit transaction", &txs.deposit));
        Container::new(Column::new().push(col)).into()
    }
}

fn transaction<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    title: &str,
    transaction: &BroadcastedTransaction,
) -> Container<'a, T> {
    Container::new(
        Column::new()
            .push(separation().width(Length::Fill))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(Container::new(Text::new(title).bold()).width(Length::Fill))
                            .push(
                                Container::new(
                                    Text::new(&transaction.tx.txid().to_string()).bold().small(),
                                )
                                .width(Length::Shrink),
                            ),
                    )
                    .push(
                        Text::new(&format!(
                            "Received at {}",
                            NaiveDateTime::from_timestamp(transaction.received_at, 0)
                        ))
                        .small(),
                    )
                    .push(
                        Text::new(&if let Some(blockheight) = &transaction.blockheight {
                            format!("Blockheight: {}", blockheight)
                        } else {
                            "Not in a block".to_string()
                        })
                        .small(),
                    ),
            )
            .push(
                Container::new(input_and_outputs(ctx, &transaction))
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(20),
    )
}

fn input_and_outputs<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    broadcasted: &BroadcastedTransaction,
) -> Container<'a, T> {
    let mut col_input = Column::new().push(Text::new("Inputs").bold()).spacing(10);
    for input in &broadcasted.tx.input {
        col_input = col_input
            .push(
                card::simple(Container::new(
                    Text::new(&format!("{}", input.previous_output)).small(),
                ))
                .width(Length::Fill),
            )
            .width(Length::FillPortion(1));
    }
    let mut col_output = Column::new().push(Text::new("Outputs").bold()).spacing(10);
    for output in &broadcasted.tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network);
        let mut col = Column::new();
        if let Some(a) = addr {
            col = col.push(Text::new(&a.to_string()).small())
        } else {
            col = col.push(Text::new(&output.script_pubkey.to_string()).small())
        }
        col_output = col_output
            .push(
                card::simple(Container::new(
                    col.push(
                        Text::new(&ctx.converter.converts(output.value).to_string())
                            .bold()
                            .small(),
                    ),
                ))
                .width(Length::Fill),
            )
            .width(Length::FillPortion(1));
    }
    Container::new(Row::new().push(col_input).push(col_output).spacing(20))
}

/// vault_badge returns a badge headlining the vault status.
fn vault_badge<'a, T: 'a>(vault: &Vault) -> Container<'a, T> {
    match &vault.status {
        VaultStatus::Unconfirmed => badge::vault_unconfirmed(),
        VaultStatus::Funded
        | VaultStatus::Securing
        | VaultStatus::Secured
        | VaultStatus::Activating
        | VaultStatus::Active => badge::tx_deposit(),
        VaultStatus::Unvaulting | VaultStatus::Unvaulted => badge::vault_unvaulting(),
        VaultStatus::Canceling
        | VaultStatus::EmergencyVaulting
        | VaultStatus::UnvaultEmergencyVaulting => badge::vault_canceling(),
        VaultStatus::Canceled
        | VaultStatus::EmergencyVaulted
        | VaultStatus::UnvaultEmergencyVaulted => badge::vault_canceled(),
        VaultStatus::Spending => badge::vault_spending(),
        VaultStatus::Spent => badge::vault_spent(),
    }
}

pub trait VaultView {
    fn new() -> Self;
    fn view<C: Client>(&mut self, ctx: &Context<C>, vault: &Vault) -> Element<Message>;
}

#[derive(Debug, Clone)]
pub struct VaultListItemView {
    state: iced::button::State,
}

impl VaultView for VaultListItemView {
    fn new() -> Self {
        VaultListItemView {
            state: iced::button::State::new(),
        }
    }

    fn view<C: Client>(&mut self, ctx: &Context<C>, vault: &Vault) -> iced::Element<Message> {
        button::white_card_button(
            &mut self.state,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(vault_badge(&vault))
                                .push(
                                    Column::new()
                                        .push(Text::new(&vault.outpoint()).bold().small())
                                        .push(Text::new(&format!("{}", &vault.status,)).small()),
                                )
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(
                                    Text::new(
                                        &format!("{}", ctx.converter.converts(vault.amount),),
                                    )
                                    .bold(),
                                )
                                .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(Message::SelectVault(vault.outpoint()))
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct SecureVaultListItemView {
    select_button: iced::button::State,
}

impl VaultView for SecureVaultListItemView {
    fn new() -> Self {
        Self {
            select_button: iced::button::State::new(),
        }
    }

    fn view<C: Client>(&mut self, ctx: &Context<C>, vault: &Vault) -> iced::Element<Message> {
        if vault.status == VaultStatus::Funded || vault.status == VaultStatus::Unconfirmed {
            vault_ack_pending(&mut self.select_button, ctx, vault)
        } else {
            vault_ack_signed(ctx, vault).map(Message::Vault)
        }
    }
}

fn vault_ack_signed<'a, T: 'a, C: Client>(ctx: &Context<C>, deposit: &Vault) -> Element<'a, T> {
    card::white(Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(badge::shield_success())
                        .push(
                            Container::new(Text::new(&deposit.address).small().bold().success())
                                .align_y(Align::Center),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(
                            Text::new(&format!("{}", ctx.converter.converts(deposit.amount)))
                                .success()
                                .bold(),
                        )
                        .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                        .align_items(Align::Center),
                )
                .width(Length::Shrink),
            )
            .spacing(20)
            .align_items(Align::Center),
    ))
    .into()
}

fn vault_ack_pending<'a, C: Client>(
    state: &'a mut iced::button::State,
    ctx: &Context<C>,
    deposit: &Vault,
) -> Element<'a, Message> {
    Container::new(
        button::white_card_button(
            state,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::shield_notif())
                                .push(
                                    Container::new(Text::new(&deposit.address).small().bold())
                                        .align_y(Align::Center),
                                )
                                .spacing(20)
                                .align_items(Align::Center),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(
                                    Text::new(&format!(
                                        "{}",
                                        ctx.converter.converts(deposit.amount),
                                    ))
                                    .bold(),
                                )
                                .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(Message::SelectVault(deposit.outpoint())),
    )
    .into()
}

#[derive(Debug, Clone)]
pub struct DelegateVaultListItemView {
    select_button: iced::button::State,
    delegate_button: iced::button::State,
}

impl DelegateVaultListItemView {
    pub fn new() -> Self {
        DelegateVaultListItemView {
            select_button: iced::button::State::new(),
            delegate_button: iced::button::State::new(),
        }
    }

    pub fn view<C: Client>(
        &mut self,
        ctx: &Context<C>,
        vault: &Vault,
        selected: bool,
    ) -> iced::Element<Message> {
        Container::new(
            button::white_card_button(
                &mut self.select_button,
                Container::new(
                    Row::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(if selected {
                                        badge::person_check_success()
                                    } else {
                                        badge::person_check()
                                    })
                                    .push(
                                        Container::new(if selected {
                                            Text::new(&vault.outpoint()).small().bold().success()
                                        } else {
                                            Text::new(&vault.outpoint()).small().bold()
                                        })
                                        .align_y(Align::Center),
                                    )
                                    .spacing(20)
                                    .align_items(Align::Center),
                            )
                            .width(Length::Fill),
                        )
                        .push(
                            Container::new(if selected {
                                Row::new()
                                    .push(
                                        Text::new(&format!(
                                            "{}",
                                            ctx.converter.converts(vault.amount),
                                        ))
                                        .bold()
                                        .success(),
                                    )
                                    .push(
                                        Text::new(&format!(" {}", ctx.converter.unit))
                                            .small()
                                            .success(),
                                    )
                                    .align_items(Align::Center)
                            } else {
                                Row::new()
                                    .push(
                                        Text::new(&format!(
                                            "{}",
                                            ctx.converter.converts(vault.amount),
                                        ))
                                        .bold(),
                                    )
                                    .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                                    .align_items(Align::Center)
                            })
                            .width(Length::Shrink),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                ),
            )
            .on_press(Message::SelectVault(vault.outpoint())),
        )
        .into()
    }
}

/// RevaultVaultView displays a section with a button asking if the user wants to revault the
/// unvaulting vault. The view displays the sucess message or the failure after the processing
/// state.
#[derive(Debug, Clone)]
pub struct RevaultVaultView {
    back_button: iced::button::State,
    broadcast_button: iced::button::State,
}

impl RevaultVaultView {
    pub fn new() -> Self {
        Self {
            back_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        _ctx: &Context<C>,
        _vault: &Vault,
        processing: &bool,
        success: &bool,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().push(warn(warning));
        if *processing {
            col = col.push(button::primary(
                &mut self.broadcast_button,
                button::button_content(None, "Broadcasting"),
            ));
        } else if *success {
            col = col.push(
                card::success(Text::new("The cancel transaction is broadcasted"))
                    .padding(20)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            );
        } else {
            col = col
                .push(Text::new("The cancel transaction will be broadcast"))
                .push(Text::new("Are you sure to revault?"))
                .push(
                    button::primary(
                        &mut self.broadcast_button,
                        button::button_content(None, "Yes, Revault"),
                    )
                    .on_press(Message::Vault(VaultMessage::Revault)),
                );
        }

        Column::new()
            .push(
                button::transparent(
                    &mut self.back_button,
                    Container::new(Text::new("< vault transactions").small()),
                )
                .on_press(Message::Vault(VaultMessage::ListOnchainTransaction)),
            )
            .push(
                card::white(Container::new(col.spacing(20).align_items(Align::Center)))
                    .width(Length::Fill)
                    .align_x(Align::Center)
                    .padding(20),
            )
            .into()
    }
}
