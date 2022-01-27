use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{scrollable, Align, Column, Container, Element, Length, Row};

use revault_ui::{
    color,
    component::{badge, button, card, form, scroll, text::Text, ContainerBackgroundStyle},
    icon,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{Message, SpendTxMessage},
        view::{manager::spend_tx_with_feerate_view, warning::warn},
    },
    daemon::model,
};

#[derive(Debug)]
pub struct SpendTransactionView {
    scroll: scrollable::State,
    delete_button: iced::button::State,
    cancel_button: iced::button::State,
}

impl SpendTransactionView {
    pub fn new() -> Self {
        SpendTransactionView {
            delete_button: iced::button::State::new(),
            cancel_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        psbt: &Psbt,
        cpfp_index: usize,
        change_index: Option<usize>,
        spent_vaults: &[model::Vault],
        action: Element<'a, Message>,
        warning: Option<&Error>,
        show_delete_button: bool,
    ) -> Element<'a, Message> {
        let col = Column::new()
            .push(spend_tx_with_feerate_view(
                ctx,
                spent_vaults,
                psbt,
                change_index,
                cpfp_index,
                None,
            ))
            .push(action)
            .spacing(20);

        let row = if show_delete_button {
            Row::new().push(
                Container::new(
                    button::primary(
                        &mut self.delete_button,
                        button::button_content(Some(icon::trash_icon()), "Delete")
                            .padding(5)
                            .width(Length::Units(100))
                            .align_x(Align::Center),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectDelete)),
                )
                .width(Length::Fill),
            )
        } else {
            Row::new().push(Column::new().width(Length::Fill))
        };

        let spend_amount = psbt
            .global
            .unsigned_tx
            .output
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(i) != change_index.as_ref() && i != &cpfp_index)
            .fold(0, |acc, (_, output)| acc + output.value);
        let change_amount = change_index
            .map(|i| psbt.global.unsigned_tx.output[i].value)
            .unwrap_or(0);

        let vaults_amount = spent_vaults.iter().fold(0, |acc, v| acc + v.amount);
        let fees = if vaults_amount == 0 {
            // Vaults are still loading
            0
        } else {
            vaults_amount - spend_amount - change_amount
        };

        let mut col_header = Column::new().push(
            Text::new(&format!(
                "txid: {}",
                psbt.global.unsigned_tx.txid().to_string()
            ))
            .small()
            .bold(),
        );
        if psbt.inputs.len() > 0 {
            col_header = col_header.push(
                Row::new()
                    .push(Text::new(&format!(
                        "Total number of signatures: {} / {}",
                        psbt.inputs[0].partial_sigs.len(),
                        ctx.managers_threshold,
                    )))
                    .push(icon::key_icon())
                    .align_items(Align::Center)
                    .spacing(5),
            )
        }

        Container::new(scroll(
            &mut self.scroll,
            Container::new(
                Column::new()
                    .push(warn(warning))
                    .push(
                        row.push(
                            Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Home)),
                            )
                            .width(Length::Shrink),
                        )
                        .align_items(Align::Center),
                    )
                    .push(card::white(Container::new(
                        Row::new()
                            .push(
                                Container::new(
                                    Row::new()
                                        .push(badge::pending_spent_tx())
                                        .push(col_header)
                                        .spacing(20),
                                )
                                .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    Column::new()
                                        .push(
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
                                                .align_items(Align::Center),
                                        )
                                        .push(
                                            Row::new()
                                                .push(
                                                    Text::new(&format!(
                                                        "Fees: {}",
                                                        ctx.converter.converts(fees),
                                                    ))
                                                    .small(),
                                                )
                                                .push(
                                                    Text::new(&format!(" {}", ctx.converter.unit))
                                                        .small(),
                                                )
                                                .align_items(Align::Center),
                                        )
                                        .align_items(Align::End),
                                )
                                .width(Length::Shrink),
                            )
                            .spacing(20)
                            .align_items(Align::Center),
                    )))
                    .push(
                        Container::new(col)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .spacing(20),
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
                .push(
                    card::success(
                        Row::new()
                            .push(Text::from(icon::done_icon()).size(20).success())
                            .push(Text::new("You signed").success())
                            .spacing(20),
                    )
                    .width(Length::Fill)
                    .align_x(Align::Center),
                )
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
                                    .align_x(Align::Center),
                            )
                            .on_press(Message::SpendTx(SpendTxMessage::UnselectDelete)),
                        )
                        .push(
                            button::primary(
                                &mut self.confirm_button,
                                button::button_content(None, "Delete transaction")
                                    .align_x(Align::Center),
                            )
                            .on_press(Message::SpendTx(SpendTxMessage::Delete)),
                        )
                        .spacing(10),
                )
        };

        Container::new(
            card::white(Container::new(
                col_action.align_items(Align::Center).spacing(20),
            ))
            .width(Length::Fill)
            .align_x(Align::Center)
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
            col_action = col_action
                .push(Text::new("Transaction is fully signed"))
                .push(button::important(
                    &mut self.confirm_button,
                    button::button_content(None, "Broadcasting"),
                ));
        } else if *success {
            col_action = col_action.push(
                card::success(Text::new("Transaction is broadcasted"))
                    .padding(20)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            );
        } else {
            col_action = col_action
                .push(Text::new("Transaction is fully signed"))
                .push(
                    button::important(
                        &mut self.confirm_button,
                        button::button_content(None, "Broadcast"),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::Broadcast)),
                );
        }

        Container::new(
            Column::new()
                .push(
                    card::success(
                        Row::new()
                            .push(Text::from(icon::done_icon()).size(20).success())
                            .push(Text::new("You signed").success())
                            .spacing(20),
                    )
                    .width(Length::Fill)
                    .align_x(Align::Center),
                )
                .push(
                    card::white(Container::new(
                        col_action.align_items(Align::Center).spacing(20),
                    ))
                    .width(Length::Fill)
                    .align_x(Align::Center)
                    .padding(20),
                )
                .spacing(20),
        )
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
        spend_amount: u64,
        fees: u64,
    ) -> Element<SpendTxMessage> {
        let mut col = Column::new().push(
            Text::new(&format!(
                "txid: {}",
                tx.psbt.global.unsigned_tx.txid().to_string()
            ))
            .small()
            .bold(),
        );
        if tx.psbt.inputs.len() > 0 {
            col = col.push(
                Row::new()
                    .push(Text::new(&format!(
                        "{} / {}",
                        tx.psbt.inputs[0].partial_sigs.len(),
                        ctx.managers_threshold
                    )))
                    .push(icon::key_icon())
                    .spacing(5)
                    .align_items(Align::Center),
            )
        }
        button::white_card_button(
            &mut self.select_button,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::pending_spent_tx())
                                .push(col)
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Column::new()
                                .push(
                                    Row::new()
                                        .push(
                                            Text::new(&format!(
                                                "{}",
                                                ctx.converter.converts(spend_amount),
                                            ))
                                            .bold(),
                                        )
                                        .push(
                                            Text::new(&format!(" {}", ctx.converter.unit)).small(),
                                        )
                                        .align_items(Align::Center),
                                )
                                .push(
                                    Row::new()
                                        .push(
                                            Text::new(&format!(
                                                "Fees: {}",
                                                ctx.converter.converts(fees),
                                            ))
                                            .small(),
                                        )
                                        .push(
                                            Text::new(&format!(" {}", ctx.converter.unit)).small(),
                                        )
                                        .align_items(Align::Center),
                                )
                                .align_items(Align::End),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ),
        )
        .on_press(SpendTxMessage::Select(tx.psbt.clone()))
        .width(Length::Fill)
        .into()
    }
}
