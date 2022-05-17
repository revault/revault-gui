use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, Amount};

use iced::{
    alignment::Horizontal, scrollable, tooltip, Alignment, Checkbox, Column, Container, Element,
    Length, Row, Tooltip,
};

use revaultd::revault_tx::transactions::RevaultTransaction;

use revault_ui::{
    color,
    component::{
        badge, button, card, form, scroll, separation, text::Text, ContainerBackgroundStyle,
        TooltipStyle,
    },
    icon,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, SpendTxMessage},
        view::{manager::spend_tx_with_feerate_view, warning::warn},
    },
    daemon::model,
};

#[derive(Debug, Default)]
pub struct SpendTransactionView {
    scroll: scrollable::State,
    delete_button: iced::button::State,
    cancel_button: iced::button::State,
    copy_button: iced::button::State,
}

impl SpendTransactionView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        tx: &model::SpendTx,
        psbt: &Psbt,
        spent_vaults: &[model::Vault],
        action: Element<'a, Message>,
        warning: Option<&Error>,
        show_delete_button: bool,
        user_signed: bool,
    ) -> Element<'a, Message> {
        let row = if show_delete_button {
            Row::new().push(
                Container::new(
                    button::primary(
                        &mut self.delete_button,
                        button::button_content(Some(icon::trash_icon()), "Delete")
                            .padding(5)
                            .width(Length::Units(100))
                            .align_x(Horizontal::Center),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectDelete)),
                )
                .width(Length::Fill),
            )
        } else {
            Row::new().push(Column::new().width(Length::Fill))
        };

        let (change_amount, spend_amount) = tx
            .psbt
            .psbt()
            .global
            .unsigned_tx
            .output
            .iter()
            .enumerate()
            .fold(
                (bitcoin::Amount::from_sat(0), bitcoin::Amount::from_sat(0)),
                |(change, spend), (i, output)| {
                    if Some(i) == tx.change_index {
                        (change + bitcoin::Amount::from_sat(output.value), spend)
                    } else if i == tx.cpfp_index {
                        (change, spend)
                    } else {
                        (change, spend + bitcoin::Amount::from_sat(output.value))
                    }
                },
            );

        let fees = tx.deposit_amount - tx.cpfp_amount - spend_amount - change_amount;

        Container::new(scroll(
            &mut self.scroll,
            Container::new(
                Column::new()
                    .push(warn(warning))
                    .push(
                        row.push(
                            Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Close),
                            )
                            .width(Length::Shrink),
                        )
                        .align_items(Alignment::Center),
                    )
                    .push(
                        Column::new()
                            .push(
                                Row::new()
                                    .push(badge::pending_spent_tx())
                                    .push(Text::new("Spend").bold())
                                    .spacing(5)
                                    .align_items(Alignment::Center),
                            )
                            .push(
                                Column::new()
                                    .push(
                                        Text::new(&format!(
                                            "- {} {}",
                                            ctx.converter.converts(spend_amount),
                                            ctx.converter.unit,
                                        ))
                                        .bold()
                                        .size(50),
                                    )
                                    .push(Container::new(Text::new(&format!(
                                        "Miner Fee: {} {}",
                                        ctx.converter.converts(fees),
                                        ctx.converter.unit,
                                    ))))
                                    .push(Container::new(Text::new(&format!(
                                        "Cpfp Amount: {} {}",
                                        ctx.converter.converts(tx.cpfp_amount),
                                        ctx.converter.unit,
                                    ))))
                                    .align_items(Alignment::Center),
                            )
                            .push(card::white(
                                Column::new()
                                    .push(Container::new(
                                        Row::new()
                                            .push(
                                                Container::new(
                                                    Row::new()
                                                        .push(Container::new(
                                                            icon::key_icon()
                                                                .size(30)
                                                                .width(Length::Fill),
                                                        ))
                                                        .push(
                                                            Column::new()
                                                                .push(
                                                                    Text::new(
                                                                        "Number of signatures:",
                                                                    )
                                                                    .bold(),
                                                                )
                                                                .push(Text::new(&format!(
                                                                    "{}",
                                                                    psbt.inputs[0]
                                                                        .partial_sigs
                                                                        .len(),
                                                                ))),
                                                        )
                                                        .align_items(Alignment::Center)
                                                        .spacing(20),
                                                )
                                                .align_x(Horizontal::Center)
                                                .width(Length::FillPortion(1)),
                                            )
                                            .push(
                                                Container::new(if user_signed {
                                                    Row::new()
                                                        .push(Container::new(
                                                            Text::from(
                                                                icon::done_icon()
                                                                    .size(30)
                                                                    .width(Length::Fill),
                                                            )
                                                            .success(),
                                                        ))
                                                        .push(Text::new("You signed").success())
                                                        .align_items(Alignment::Center)
                                                        .spacing(20)
                                                } else {
                                                    Row::new()
                                                        .push(Container::new(Text::from(
                                                            icon::cross_icon()
                                                                .size(30)
                                                                .width(Length::Fill),
                                                        )))
                                                        .push(Text::new("You did not sign"))
                                                        .align_items(Alignment::Center)
                                                        .spacing(20)
                                                })
                                                .align_x(Horizontal::Center)
                                                .width(Length::FillPortion(1)),
                                            )
                                            .align_items(Alignment::Center)
                                            .spacing(20),
                                    ))
                                    .push(separation().width(Length::Fill))
                                    .push(
                                        Column::new()
                                            .push(
                                                Row::new()
                                                    .push(
                                                        Text::new("Tx ID:")
                                                            .bold()
                                                            .width(Length::Fill),
                                                    )
                                                    .push(
                                                        Text::new(&format!(
                                                            "{}",
                                                            tx.psbt
                                                                .psbt()
                                                                .global
                                                                .unsigned_tx
                                                                .txid()
                                                        ))
                                                        .small(),
                                                    )
                                                    .align_items(Alignment::Center),
                                            )
                                            .push(
                                                Row::new()
                                                    .push(
                                                        Text::new("Psbt:")
                                                            .bold()
                                                            .width(Length::Fill),
                                                    )
                                                    .push(Text::new("copy").small())
                                                    .push(button::clipboard(
                                                        &mut self.copy_button,
                                                        Message::Clipboard(psbt.to_string()),
                                                    ))
                                                    .align_items(Alignment::Center),
                                            ),
                                    )
                                    .spacing(20),
                            ))
                            .push(action)
                            .push(spend_tx_with_feerate_view(
                                ctx,
                                spent_vaults,
                                tx.psbt.psbt(),
                                tx.change_index,
                                tx.cpfp_index,
                            ))
                            .spacing(20)
                            .max_width(800)
                            .align_items(Alignment::Center),
                    )
                    .align_items(Alignment::Center),
            ),
        ))
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionSharePsbtView {
    psbt_input: iced::text_input::State,
    copy_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionSharePsbtView {
    pub fn new() -> Self {
        Self {
            psbt_input: iced::text_input::State::new(),
            copy_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        psbt_input: &form::Value<String>,
        processing: &bool,
        success: &bool,
        psbt: &Psbt,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let psbt_str = bitcoin::base64::encode(&bitcoin::consensus::serialize(psbt));
        let mut col_action = Column::new().spacing(20).push(
            Column::new().push(
                Row::new()
                    .push(Container::new(Text::new(&psbt_str).small()).width(Length::Fill))
                    .push(
                        button::clipboard(&mut self.copy_button, Message::Clipboard(psbt_str))
                            .width(Length::Shrink),
                    )
                    .width(Length::Fill),
            ),
        );
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        let mut button_update_action = button::important(
            &mut self.confirm_button,
            button::button_content(None, "Update transaction"),
        );
        if !*processing {
            button_update_action =
                button_update_action.on_press(Message::SpendTx(SpendTxMessage::Update));
        }
        if *success {
            col_action = col_action.push(Text::new("Transaction updated").success());
        }
        Container::new(
            Column::new()
                .push(card::white(Container::new(
                    col_action
                        .push(Text::new("Enter PSBT:"))
                        .push(
                            form::Form::new(
                                &mut self.psbt_input,
                                "Signed PSBT",
                                &psbt_input,
                                |p| Message::SpendTx(SpendTxMessage::PsbtEdited(p)),
                            )
                            .warning("PSBT is not valid or signatures are from unknown sources")
                            .size(20)
                            .padding(10)
                            .render(),
                        )
                        .push(button_update_action),
                )))
                .spacing(20),
        )
        .into()
    }
}

pub fn spend_tx_confirmed<'a, T: 'a>() -> Element<'a, T> {
    card::white(
        Row::new()
            .push(badge::Badge::new(icon::send_icon()).style(badge::Style::Success))
            .push(Text::new("Transaction was confirmed in the blockchain").color(color::SUCCESS))
            .align_items(Alignment::Center)
            .spacing(20),
    )
    .align_x(Horizontal::Center)
    .width(Length::Fill)
    .into()
}

pub fn spend_tx_processing<'a, T: 'a>() -> Element<'a, T> {
    card::white(
        Row::new()
            .push(badge::Badge::new(icon::send_icon()).style(badge::Style::Warning))
            .push(Text::new("Transaction is being processed").color(color::WARNING))
            .align_items(Alignment::Center)
            .spacing(20),
    )
    .align_x(Horizontal::Center)
    .width(Length::Fill)
    .into()
}

pub fn spend_tx_deprecated<'a, T: 'a>() -> Element<'a, T> {
    card::white(
        Row::new()
            .push(badge::Badge::new(icon::cross_icon()))
            .push(
                Column::new()
                    .push(Text::new("Transaction is deprecated").bold())
                    .push(Text::new("One of its vaults was used by another transaction").small()),
            )
            .align_items(Alignment::Center)
            .spacing(20),
    )
    .align_x(Horizontal::Center)
    .width(Length::Fill)
    .into()
}

#[derive(Debug)]
pub struct SpendTransactionSignView {}

impl SpendTransactionSignView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn view<'a>(
        &'a mut self,
        signer: Element<'a, Message>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col_action = Column::new().push(signer);
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        Container::new(card::white(Container::new(col_action))).into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionDeleteView {
    unconfirm_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionDeleteView {
    pub fn new() -> Self {
        Self {
            unconfirm_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        processing: &bool,
        success: &bool,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let mut col_action = Column::new();
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        if *processing {
            col_action = col_action.push(Text::new("Deleting..."));
        } else if *success {
            col_action = col_action.push(Text::new("Deleted").color(color::SUCCESS));
        } else {
            col_action = col_action
                .push(Text::new(
                    "Are you sure you want to delete this transaction?",
                ))
                .push(
                    Row::new()
                        .push(
                            button::important(
                                &mut self.unconfirm_button,
                                button::button_content(None, "No")
                                    .width(Length::Units(100))
                                    .align_x(Horizontal::Center),
                            )
                            .on_press(Message::SpendTx(SpendTxMessage::UnselectDelete)),
                        )
                        .push(
                            button::primary(
                                &mut self.confirm_button,
                                button::button_content(None, "Delete transaction")
                                    .align_x(Horizontal::Center),
                            )
                            .on_press(Message::SpendTx(SpendTxMessage::Delete)),
                        )
                        .spacing(10),
                )
        };

        Container::new(
            card::white(Container::new(
                col_action.align_items(Alignment::Center).spacing(20),
            ))
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .padding(20),
        )
        .width(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionBroadcastView {
    confirm_button: iced::button::State,
}

impl SpendTransactionBroadcastView {
    pub fn new() -> Self {
        Self {
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        processing: bool,
        success: bool,
        with_priority: bool,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let mut col_action = Column::new();
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        if processing {
            col_action = col_action
                .push(Text::new("Transaction is fully signed"))
                .push(button::important(
                    &mut self.confirm_button,
                    button::button_content(None, "Broadcasting"),
                ));
        } else if success {
            col_action = col_action.push(
                card::success(Text::new("Transaction is broadcasted"))
                    .padding(20)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            );
        } else {
            col_action = col_action
                .push(Text::new("Transaction is fully signed"))
                .push(
                    Row::new()
                        .push(Checkbox::new(
                            with_priority,
                            String::from("Set high priority"),
                            |priority| Message::SpendTx(SpendTxMessage::WithPriority(priority)),
                        ))
                        .push(
                            Tooltip::new(
                                icon::tooltip_icon().size(15),
                                "try to feebump the transactions in the background if it does not confirm.",
                                tooltip::Position::Right,
                            )
                            .gap(5)
                            .size(20)
                            .padding(10)
                            .style(TooltipStyle),
                        )
                        .spacing(5),
                )
                .push(
                    button::important(
                        &mut self.confirm_button,
                        button::button_content(None, "Broadcast"),
                    )
                    .width(Length::Units(200))
                    .on_press(Message::SpendTx(SpendTxMessage::Broadcast)),
                );
        }

        card::white(Container::new(
            col_action.align_items(Alignment::Center).spacing(20),
        ))
        .width(Length::Fill)
        .align_x(Horizontal::Center)
        .padding(20)
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionListItemView {
    select_button: iced::button::State,
}

impl SpendTransactionListItemView {
    pub fn new() -> Self {
        Self {
            select_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        ctx: &Context,
        tx: &model::SpendTx,
        spend_amount: Amount,
        fees: Amount,
    ) -> Element<SpendTxMessage> {
        let n_sigs = tx
            .psbt
            .psbt()
            .inputs
            .get(0)
            .map(|output| output.partial_sigs.len())
            .unwrap_or(0);

        let row = Row::new()
            .push(match tx.status {
                model::ListSpendStatus::NonFinal => {
                    badge::Badge::new(icon::send_icon()).style(badge::Style::Standard)
                }
                model::ListSpendStatus::Pending | model::ListSpendStatus::Broadcasted => {
                    badge::Badge::new(icon::send_icon()).style(badge::Style::Warning)
                }
                model::ListSpendStatus::Confirmed => {
                    badge::Badge::new(icon::send_icon()).style(badge::Style::Success)
                }
                model::ListSpendStatus::Deprecated => {
                    badge::Badge::new(icon::cross_icon()).style(badge::Style::Standard)
                }
            })
            .push(
                match tx.status {
                    model::ListSpendStatus::NonFinal => {
                        if n_sigs < ctx.managers_threshold {
                            Text::new("Non final ")
                        } else {
                            Text::new("Ready    ").success()
                        }
                    }
                    model::ListSpendStatus::Pending | model::ListSpendStatus::Broadcasted => {
                        Text::new("Processing").color(color::WARNING)
                    }
                    model::ListSpendStatus::Confirmed => Text::new("Confirmed ").success(),
                    model::ListSpendStatus::Deprecated => Text::new("Deprecated"),
                }
                .small()
                .bold(),
            )
            .push(
                Row::new()
                    .push(Text::new(&format!(
                        "{} / {}",
                        if n_sigs > ctx.managers_threshold {
                            ctx.managers_threshold
                        } else {
                            n_sigs
                        },
                        ctx.managers_threshold
                    )))
                    .push(icon::key_icon())
                    .spacing(5)
                    .align_items(Alignment::Center),
            )
            .align_items(Alignment::Center)
            .spacing(20);

        button::white_card_button(
            &mut self.select_button,
            Container::new(
                Row::new()
                    .push(Container::new(row).width(Length::FillPortion(2)))
                    .push(
                        Container::new(
                            Row::new()
                                .push(
                                    Container::new(
                                        Text::new(&format!(
                                            "fee: -{}",
                                            ctx.converter.converts(fees),
                                        ))
                                        .small(),
                                    )
                                    .align_x(Horizontal::Right)
                                    .width(Length::FillPortion(1)),
                                )
                                .push(
                                    Container::new(
                                        Row::new()
                                            .push(
                                                Text::new(&format!(
                                                    "{}",
                                                    ctx.converter.converts(spend_amount),
                                                ))
                                                .bold(),
                                            )
                                            .push(
                                                Text::new(&format!(" {}", ctx.converter.unit))
                                                    .small(),
                                            )
                                            .align_items(Alignment::Center),
                                    )
                                    .align_x(Horizontal::Right)
                                    .width(Length::FillPortion(1)),
                                )
                                .align_items(Alignment::Center),
                        )
                        .align_x(Horizontal::Right)
                        .width(Length::FillPortion(2)),
                    )
                    .spacing(20)
                    .align_items(Alignment::Center),
            ),
        )
        .on_press(SpendTxMessage::Select(tx.psbt.psbt().clone()))
        .width(Length::Fill)
        .into()
    }
}
