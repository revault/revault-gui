use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{scrollable, Align, Column, Container, Element, Length, Row, TextInput};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{Message, SpendTxMessage},
        view::manager::spend_tx_with_feerate_view,
    },
    daemon::model,
    ui::{
        color,
        component::{badge, button, card, scroll, text, ContainerBackgroundStyle},
        icon,
    },
};

#[derive(Debug)]
pub struct SpendTransactionView {
    scroll: scrollable::State,
    delete_button: iced::button::State,
    cancel_button: iced::button::State,
    psbt_input: iced::text_input::State,
    import_button: iced::button::State,
}

impl SpendTransactionView {
    pub fn new() -> Self {
        SpendTransactionView {
            delete_button: iced::button::State::new(),
            cancel_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            psbt_input: iced::text_input::State::new(),
            import_button: iced::button::State::new(),
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
        let mut col = Column::new().spacing(20);
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))))
        }
        col = col
            .push(spend_tx_with_feerate_view(
                ctx,
                spent_vaults,
                psbt,
                change_index,
                cpfp_index,
                None,
            ))
            .push(action);

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

        let mut col_header = Column::new().push(text::bold(text::small(&format!(
            "txid: {}",
            psbt.global.unsigned_tx.txid().to_string()
        ))));
        if psbt.inputs.len() > 0 {
            col_header = col_header.push(
                Row::new()
                    .push(text::simple(&format!(
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
                                                .push(text::bold(text::simple(&format!(
                                                    "{}",
                                                    ctx.converter.converts(spend_amount),
                                                ))))
                                                .push(text::small(&format!(
                                                    " {}",
                                                    ctx.converter.unit
                                                )))
                                                .align_items(Align::Center),
                                        )
                                        .push(
                                            Row::new()
                                                .push(text::small(&format!(
                                                    "Fees: {}",
                                                    ctx.converter.converts(fees),
                                                )))
                                                .push(text::small(&format!(
                                                    " {}",
                                                    ctx.converter.unit
                                                )))
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
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    psbt_input: iced::text_input::State,
    copy_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionSharePsbtView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            psbt_input: iced::text_input::State::new(),
            copy_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        psbt_input: &str,
        processing: &bool,
        success: &bool,
        psbt: &Psbt,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let col = Column::new().push(
            Container::new(
                Row::new()
                    .push(
                        button::primary(
                            &mut self.share_button,
                            button::button_content(None, "Share and update"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectShare)),
                    )
                    .push(
                        button::transparent(
                            &mut self.sign_button,
                            button::button_content(None, "Sign"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectSign)),
                    )
                    .push(
                        button::transparent(
                            &mut self.broadcast_button,
                            button::button_content(None, "Broadcast"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectBroadcast)),
                    ),
            )
            .width(Length::Fill)
            .align_x(Align::End),
        );

        let psbt_str = bitcoin::base64::encode(&bitcoin::consensus::serialize(psbt));
        let mut col_action = Column::new().spacing(20).push(
            Column::new().push(
                Row::new()
                    .push(Container::new(text::small(&psbt_str)).width(Length::Fill))
                    .push(
                        button::clipboard(&mut self.copy_button, Message::Clipboard(psbt_str))
                            .width(Length::Shrink),
                    )
                    .width(Length::Fill),
            ),
        );
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
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
            col_action = col_action.push(text::success(text::simple("Transaction updated")));
        }
        Container::new(
            col.push(card::white(Container::new(
                col_action
                    .push(text::simple("Enter PSBT:"))
                    .push(
                        TextInput::new(&mut self.psbt_input, "Signed PSBT", &psbt_input, |p| {
                            Message::SpendTx(SpendTxMessage::PsbtEdited(p))
                        })
                        .size(15)
                        .width(Length::Fill)
                        .padding(10),
                    )
                    .push(button_update_action),
            )))
            .spacing(20),
        )
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionSignView {
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionSignView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        signer: Element<'a, Message>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let col = Column::new().push(
            Container::new(
                Row::new()
                    .push(
                        button::transparent(
                            &mut self.share_button,
                            button::button_content(None, "Share and update"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectShare)),
                    )
                    .push(
                        button::primary(
                            &mut self.sign_button,
                            button::button_content(None, "Sign"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectSign)),
                    )
                    .push(
                        button::transparent(
                            &mut self.broadcast_button,
                            button::button_content(None, "Broadcast"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::SelectBroadcast)),
                    ),
            )
            .width(Length::Fill)
            .align_x(Align::End),
        );

        let mut col_action = Column::new().push(signer);
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }

        Container::new(
            col.push(card::white(Container::new(col_action)))
                .spacing(20),
        )
        .into()
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
            col_action = col_action.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }

        if *processing {
            col_action = col_action.push(text::simple("Deleting..."));
        } else if *success {
            col_action = col_action.push(text::simple("Deleted").color(color::SUCCESS));
        } else {
            col_action = col_action
                .push(text::simple(
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
                            .on_press(Message::SpendTx(SpendTxMessage::SelectShare)),
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
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionBroadcastView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        processing: &bool,
        success: &bool,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let button_broadcast_action = if *processing {
            button::important(
                &mut self.confirm_button,
                button::button_content(None, "Broadcasting"),
            )
        } else if *success {
            button::success(
                &mut self.confirm_button,
                button::button_content(None, "Broadcasted"),
            )
        } else {
            button::important(
                &mut self.confirm_button,
                button::button_content(None, "Yes broadcast"),
            )
            .on_press(Message::SpendTx(SpendTxMessage::Broadcast))
        };

        let mut button_share = button::transparent(
            &mut self.share_button,
            button::button_content(None, "Share and update"),
        );
        if !*success {
            button_share = button_share.on_press(Message::SpendTx(SpendTxMessage::SelectShare));
        }

        let mut button_sign =
            button::transparent(&mut self.sign_button, button::button_content(None, "Sign"));
        if !*success {
            button_sign = button_sign.on_press(Message::SpendTx(SpendTxMessage::SelectSign));
        }

        let mut button_broadcast = button::primary(
            &mut self.broadcast_button,
            button::button_content(None, "Broadcast"),
        );
        if !*success {
            button_broadcast =
                button_broadcast.on_press(Message::SpendTx(SpendTxMessage::SelectBroadcast));
        }

        let mut col_action = Column::new();
        if let Some(error) = warning {
            col_action = col_action.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }

        Container::new(
            Column::new()
                .push(
                    Container::new(
                        Row::new()
                            .push(button_share)
                            .push(button_sign)
                            .push(button_broadcast),
                    )
                    .width(Length::Fill)
                    .align_x(Align::End),
                )
                .push(
                    card::white(Container::new(
                        col_action
                            .push(text::simple(
                                "Are you sure you want to broadcast this transaction ?",
                            ))
                            .push(button_broadcast_action)
                            .align_items(Align::Center)
                            .spacing(20),
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
        let mut col = Column::new().push(text::bold(text::small(&format!(
            "txid: {}",
            tx.psbt.global.unsigned_tx.txid().to_string()
        ))));
        if tx.psbt.inputs.len() > 0 {
            col = col.push(
                Row::new()
                    .push(text::simple(&format!(
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
                                        .push(text::bold(text::simple(&format!(
                                            "{}",
                                            ctx.converter.converts(spend_amount),
                                        ))))
                                        .push(text::small(&format!(" {}", ctx.converter.unit)))
                                        .align_items(Align::Center),
                                )
                                .push(
                                    Row::new()
                                        .push(text::small(&format!(
                                            "Fees: {}",
                                            ctx.converter.converts(fees),
                                        )))
                                        .push(text::small(&format!(" {}", ctx.converter.unit)))
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
