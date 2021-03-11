use chrono::NaiveDateTime;
use iced::{scrollable, Align, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    component::{badge, button, card, separation, text, ContainerBackgroundStyle},
    message::Message,
    view::Context,
};

use crate::revaultd::model::{BroadcastedTransaction, Vault, VaultStatus, VaultTransactions};

#[derive(Debug)]
pub struct VaultModal {
    copy_button: iced::button::State,
    cancel_button: iced::button::State,
    scroll: scrollable::State,
}

impl VaultModal {
    pub fn new() -> Self {
        VaultModal {
            copy_button: iced::button::State::default(),
            cancel_button: iced::button::State::default(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vlt: &Vault,
        txs: &VaultTransactions,
    ) -> Element<'a, Message> {
        let tx = txs.last_broadcasted_tx();
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(
                                button::cancel(
                                    &mut self.cancel_button,
                                    Container::new(text::simple("X Close")).padding(10),
                                )
                                .on_press(Message::SelectVault(vlt.outpoint())),
                            )
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(text::simple("Transaction Details"))
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .push(
                        Container::new(
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
                                                                        .push(text::small(
                                                                            &vlt.txid,
                                                                        ))
                                                                        .push(button::clipboard(
                                                                            &mut self.copy_button,
                                                                            Message::Clipboard(
                                                                                vlt.txid
                                                                                    .to_string(),
                                                                            ),
                                                                        ))
                                                                        .align_items(Align::Center),
                                                                )
                                                                .push(text::small(&format!(
                                                                    "{} ( {} )",
                                                                    &vlt.status,
                                                                    NaiveDateTime::from_timestamp(
                                                                        tx.received_at,
                                                                        0
                                                                    )
                                                                ))),
                                                        )
                                                        .spacing(20),
                                                )
                                                .width(Length::Fill),
                                            )
                                            .push(
                                                Container::new(
                                                    Row::new()
                                                        .push(text::bold(text::simple(&format!(
                                                            "{}",
                                                            ctx.converter.converts(vlt.amount),
                                                        ))))
                                                        .push(text::simple(&format!(
                                                            "{}",
                                                            ctx.converter.unit,
                                                        ))),
                                                )
                                                .width(Length::Shrink),
                                            )
                                            .spacing(20)
                                            .align_items(Align::Center),
                                    )
                                    .push(separation().width(Length::Fill))
                                    .push(
                                        Row::new()
                                            .push(
                                                Container::new(
                                                    Column::new()
                                                        .push(text::bold(text::simple(
                                                            "Blockheight",
                                                        )))
                                                        .push(text::simple(&if let Some(
                                                            blockheight,
                                                        ) = &tx.blockheight
                                                        {
                                                            format!("{}", blockheight)
                                                        } else {
                                                            "Not in a block".to_string()
                                                        })),
                                                )
                                                .width(Length::FillPortion(2)),
                                            )
                                            .push(
                                                Container::new(
                                                    Column::new()
                                                        .push(text::bold(text::simple("Fee"))),
                                                )
                                                .width(Length::FillPortion(2)),
                                            ),
                                    )
                                    .spacing(20),
                            ))
                            .max_width(1000)
                            .padding(20),
                        )
                        .width(Length::Fill)
                        .align_x(Align::Center),
                    )
                    .push(
                        input_and_outputs(ctx, &tx)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .into()
    }
}

fn input_and_outputs<'a, T: 'a>(
    ctx: &Context,
    broadcasted: &BroadcastedTransaction,
) -> Container<'a, T> {
    let mut col_input = Column::new()
        .push(text::bold(text::simple("Inputs")))
        .spacing(10);
    for input in &broadcasted.tx.input {
        col_input = col_input.push(card::simple(Container::new(text::small(&format!(
            "{}",
            input.previous_output
        )))));
    }
    let mut col_output = Column::new()
        .push(text::bold(text::simple("Outputs")))
        .spacing(10);
    for output in &broadcasted.tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network);
        let mut col = Column::new();
        if let Some(a) = addr {
            col = col.push(text::small(&format!("{}", a,)))
        } else {
            col = col.push(text::small(&format!("{}", &output.script_pubkey)))
        }
        col_output = col_output.push(card::simple(Container::new(col.push(text::bold(
            text::small(&format!("{}", ctx.converter.converts(output.value))),
        )))));
    }
    Container::new(Row::new().push(col_input).push(col_output).spacing(20))
}

/// vault_badge returns a badge headlining the vault status.
fn vault_badge<'a, T: 'a>(vault: &Vault) -> Container<'a, T> {
    match &vault.status {
        VaultStatus::Unconfirmed => badge::vault_unconfirmed(),
        VaultStatus::Funded | VaultStatus::Secured | VaultStatus::Active => badge::tx_deposit(),
        VaultStatus::Unvaulting | VaultStatus::Unvaulted => badge::vault_unvaulting(),
        VaultStatus::Canceling | VaultStatus::EmergencyVaulting => badge::vault_canceling(),
        VaultStatus::Canceled | VaultStatus::EmergencyVaulted => badge::vault_canceled(),
        VaultStatus::Spendable | VaultStatus::Spending => badge::vault_spending(),
        VaultStatus::Spent => badge::vault_spent(),
    }
}

#[derive(Debug, Clone)]
pub struct VaultListItemView {
    state: iced::button::State,
}

impl VaultListItemView {
    pub fn new() -> Self {
        VaultListItemView {
            state: iced::button::State::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context, vault: &Vault) -> iced::Element<Message> {
        let updated_at = NaiveDateTime::from_timestamp(vault.updated_at, 0);
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
                                        .push(text::bold(text::small(&vault.address)))
                                        .push(text::small(&format!(
                                            "{} ( {} )",
                                            &vault.status, updated_at
                                        ))),
                                )
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(vault.amount),
                                ))))
                                .push(text::small(&format!(" {}", ctx.converter.unit)))
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
