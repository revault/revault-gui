use chrono::NaiveDateTime;
use iced::{pure::Pure, tooltip, Alignment, Column, Container, Element, Length, Row, Tooltip};

use bitcoin::{util::bip32::Fingerprint, Amount};
use revault_ui::{
    color,
    component::{badge, button, card, collapse::collapse, separation, text::Text, TooltipStyle},
    icon,
};

use crate::app::{context::Context, message::Message, view::layout};

use crate::daemon::model::{
    outpoint, transaction_from_hex, Vault, VaultStatus, VaultTransactions, WalletTransaction,
};

pub struct VaultModal {
    copy_button: iced::button::State,
    modal: layout::Modal,
    state: iced::pure::State,
}

impl std::fmt::Debug for VaultModal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultModal").finish()
    }
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            copy_button: iced::button::State::default(),
            modal: layout::Modal::default(),
            state: iced::pure::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vlt: &Vault,
        txs: &VaultTransactions,
    ) -> Element<'a, Message> {
        let mut col = Column::new().spacing(20);
        col = col.push(Container::new(Text::new("Onchain transactions:").bold()));
        if let Some(tx) = &txs.spend {
            col = col.push(transaction(ctx, "Spend transaction", tx));
        }
        if let Some(tx) = &txs.cancel {
            col = col.push(transaction(ctx, "Cancel transaction", tx));
        }
        if let Some(tx) = &txs.unvault_emergency {
            col = col.push(transaction(ctx, "Unvault Emergency transaction", tx));
        }
        if let Some(tx) = &txs.emergency {
            col = col.push(transaction(ctx, "Emergency transaction", tx));
        }
        if let Some(tx) = &txs.unvault {
            col = col.push(transaction(ctx, "Unvault transaction", tx));
        }
        col = col.push(transaction_collapse(
            ctx,
            "Deposit transaction",
            &txs.deposit,
            &mut self.state,
        ));

        self.modal.view(
            ctx,
            None,
            Container::new(
                Column::new()
                    .push(Container::new(vault(ctx, &mut self.copy_button, vlt)))
                    .push(col)
                    .max_width(1000)
                    .spacing(20),
            )
            .width(Length::Fill)
            .center_x(),
            None,
            Message::Close,
        )
    }
}

fn vault<'a>(
    ctx: &Context,
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
                                                .align_items(Alignment::Center),
                                        )
                                        .push(Text::new(&format!("{}", &vlt.status,))),
                                )
                                .align_items(Alignment::Center)
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
                    .align_items(Alignment::Center),
            )
            .spacing(20),
    ))
}

fn transaction_collapse<'a, T: Clone + 'a>(
    ctx: &Context,
    title: &str,
    transaction: &WalletTransaction,
    state: &'a mut iced::pure::State,
) -> Container<'a, T> {
    let tx = transaction_from_hex(&transaction.hex);
    Container::new(Pure::new(
        state,
        collapse::<_, T, _, _, _>(
            move || {
                iced::pure::row()
                    .push(iced::pure::text("hello"))
                    .push(iced::pure::text("hello-again"))
                    .width(Length::Fill)
                    .into()
            },
            move || {
                iced::pure::column()
                    .push(iced::pure::text(format!("{}", tx.txid())))
                    .into()
            },
        ),
    ))
}

fn transaction<'a, T: 'a>(
    ctx: &Context,
    title: &str,
    transaction: &WalletTransaction,
) -> Container<'a, T> {
    let tx = transaction_from_hex(&transaction.hex);
    Container::new(
        Column::new()
            .push(separation().width(Length::Fill))
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(Container::new(Text::new(title).bold()).width(Length::Fill))
                            .push(
                                Container::new(Text::new(&tx.txid().to_string()).bold().small())
                                    .width(Length::Shrink),
                            ),
                    )
                    .push(
                        Text::new(&format!(
                            "Received at {}",
                            NaiveDateTime::from_timestamp(transaction.received_time.into(), 0)
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
                Container::new(input_and_outputs(ctx, transaction))
                    .width(Length::Fill)
                    .center_x(),
            )
            .spacing(20),
    )
}

fn input_and_outputs<'a, T: 'a>(
    ctx: &Context,
    broadcasted: &WalletTransaction,
) -> Container<'a, T> {
    let mut col_input = Column::new().push(Text::new("Inputs").bold()).spacing(10);
    let tx = transaction_from_hex(&broadcasted.hex);
    for input in &tx.input {
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
    for output in &tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network());
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
                        Text::new(
                            &ctx.converter
                                .converts(Amount::from_sat(output.value))
                                .to_string(),
                        )
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
fn vault_badge<'a, T: 'a>(vault: &Vault) -> Element<'a, T> {
    match &vault.status {
        VaultStatus::Unconfirmed => badge::vault_unconfirmed().into(),
        VaultStatus::Funded => badge::tx_deposit().into(),
        VaultStatus::Securing => badge::vault_securing().into(),
        VaultStatus::Secured | VaultStatus::Activating | VaultStatus::Active => {
            badge::vault().into()
        }
        VaultStatus::Unvaulting | VaultStatus::Unvaulted => badge::Badge::new(icon::unlock_icon())
            .style(badge::Style::Warning)
            .into(),
        VaultStatus::Canceling
        | VaultStatus::EmergencyVaulting
        | VaultStatus::UnvaultEmergencyVaulting => badge::vault_canceling().into(),
        VaultStatus::Canceled
        | VaultStatus::EmergencyVaulted
        | VaultStatus::UnvaultEmergencyVaulted => badge::vault_canceled().into(),
        VaultStatus::Spending => badge::vault_spending().into(),
        VaultStatus::Spent => badge::vault_spent().into(),
    }
}

pub trait VaultView {
    fn new() -> Self;
    fn view(&mut self, ctx: &Context, vault: &Vault) -> Element<Message>;
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

    fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        button::white_card_button(
            &mut self.state,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(vault_badge(&vault))
                                .push(if vault.status == VaultStatus::Activating {
                                    Text::new("Delegation approved").small()
                                } else if vault.status == VaultStatus::Active {
                                    Text::new("Delegated").bold().small()
                                } else if vault.status == VaultStatus::Secured {
                                    Text::new("").small()
                                } else if vault.status == VaultStatus::Securing {
                                    Text::new("Waiting other stakeholders signatures").small()
                                } else {
                                    Text::new(&format!("{}", &vault.status)).bold().small()
                                })
                                .align_items(Alignment::Center)
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(Text::new(&ctx.converter.converts(vault.amount)).bold())
                                .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                                .align_items(Alignment::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Alignment::Center),
            ),
        )
        .on_press(Message::SelectVault(outpoint(vault)))
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

    fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        if vault.status == VaultStatus::Funded || vault.status == VaultStatus::Unconfirmed {
            vault_ack_pending(&mut self.select_button, ctx, vault)
        } else {
            vault_ack_signed(ctx, vault).map(Message::Vault)
        }
    }
}

fn vault_ack_signed<'a, T: 'a>(ctx: &Context, deposit: &Vault) -> Element<'a, T> {
    card::white(Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(badge::shield_success())
                        .push(
                            Container::new(
                                Text::new(&deposit.address.to_string())
                                    .small()
                                    .bold()
                                    .success(),
                            )
                            .center_y(),
                        )
                        .spacing(20)
                        .align_items(Alignment::Center),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(
                            Text::new(&ctx.converter.converts(deposit.amount))
                                .success()
                                .bold(),
                        )
                        .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                        .align_items(Alignment::Center),
                )
                .width(Length::Shrink),
            )
            .spacing(20)
            .align_items(Alignment::Center),
    ))
    .into()
}

fn vault_ack_pending<'a>(
    state: &'a mut iced::button::State,
    ctx: &Context,
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
                                    Container::new(
                                        Text::new(&deposit.address.to_string()).small().bold(),
                                    )
                                    .center_y(),
                                )
                                .spacing(20)
                                .align_items(Alignment::Center),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(Text::new(&ctx.converter.converts(deposit.amount)).bold())
                                .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                                .align_items(Alignment::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Alignment::Center),
            ),
        )
        .on_press(Message::SelectVault(outpoint(deposit))),
    )
    .into()
}

#[derive(Debug, Clone)]
pub struct DelegateVaultListItemView {
    select_button: iced::button::State,
}

impl DelegateVaultListItemView {
    pub fn new() -> Self {
        DelegateVaultListItemView {
            select_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        ctx: &Context,
        vault: &Vault,
        sigs: &Vec<Fingerprint>,
        selected: bool,
    ) -> iced::Element<Message> {
        let mut sigs_row = Row::new().align_items(Alignment::Center);
        for fingerprint in sigs {
            sigs_row = sigs_row.push(
                Tooltip::new(
                    icon::person_icon().color(color::SUCCESS),
                    fingerprint.to_string(),
                    tooltip::Position::Top,
                )
                .gap(5)
                .size(20)
                .padding(10)
                .style(TooltipStyle),
            );
        }
        for _i in 0..(ctx.stakeholders_xpubs().len() - sigs.len()) {
            if vault.status == VaultStatus::Activating {
                sigs_row = sigs_row.push(icon::person_icon().color(color::DARK_GREY));
            } else {
                sigs_row = sigs_row.push(icon::person_icon());
            }
        }
        let content = Container::new(
            Row::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(if vault.status == VaultStatus::Activating {
                                Container::new(
                                    Tooltip::new(
                                        badge::circle_check_success(),
                                        "You signed",
                                        tooltip::Position::Right,
                                    )
                                    .gap(5)
                                    .size(20)
                                    .style(TooltipStyle),
                                )
                            } else if selected {
                                badge::square_check()
                            } else {
                                badge::square()
                            })
                            .push(sigs_row)
                            .spacing(20)
                            .align_items(Alignment::Center),
                    )
                    .width(Length::Fill),
                )
                .push(
                    Container::new(if selected {
                        Row::new()
                            .push(
                                Text::new(&ctx.converter.converts(vault.amount))
                                    .bold()
                                    .color(color::PRIMARY),
                            )
                            .push(
                                Text::new(&format!(" {}", ctx.converter.unit))
                                    .small()
                                    .color(color::PRIMARY),
                            )
                            .align_items(Alignment::Center)
                    } else {
                        Row::new()
                            .push(if vault.status == VaultStatus::Activating {
                                Text::new(&ctx.converter.converts(vault.amount))
                                    .bold()
                                    .color(color::DARK_GREY)
                            } else {
                                Text::new(&ctx.converter.converts(vault.amount)).bold()
                            })
                            .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                            .align_items(Alignment::Center)
                    })
                    .width(Length::Shrink),
                )
                .spacing(20)
                .align_items(Alignment::Center),
        );

        if vault.status == VaultStatus::Secured {
            button::white_card_button(&mut self.select_button, content)
                .on_press(Message::SelectVault(outpoint(vault)))
                .into()
        } else {
            card::white(content).padding(15).into()
        }
    }
}
