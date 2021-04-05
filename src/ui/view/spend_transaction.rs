use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{scrollable, Align, Column, Container, Element, Length, Row, Scrollable, TextInput};

use crate::{
    revaultd::model,
    ui::{
        component::{badge, button, card, text, ContainerBackgroundStyle},
        error::Error,
        menu::Menu,
        message::{Message, SpendTxMessage},
        view::{manager::spend_tx_with_feerate_view, Context},
    },
};

#[derive(Debug)]
pub struct SpendTransactionView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    psbt_input: iced::text_input::State,
    import_button: iced::button::State,
}

impl SpendTransactionView {
    pub fn new() -> Self {
        SpendTransactionView {
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
        spent_vaults: &[model::Vault],
        action: Element<'a, Message>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().spacing(20);
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))))
        }
        col = col
            .push(spend_tx_with_feerate_view(ctx, spent_vaults, psbt, None))
            .push(action);
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
                                .on_press(Message::Menu(Menu::Home)),
                            )
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(col)
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
        .into()
    }
}

#[derive(Debug)]
pub struct SpendTransactionSharePsbtView {
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    delete_button: iced::button::State,
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
            delete_button: iced::button::State::new(),
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
                )
                .push(
                    button::transparent(
                        &mut self.delete_button,
                        button::button_content(None, "Delete"),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectDelete)),
                ),
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
    delete_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionSignView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            delete_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        signer: Element<'a, Message>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let col = Column::new().push(
            Row::new()
                .push(
                    button::transparent(
                        &mut self.share_button,
                        button::button_content(None, "Share and update"),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectShare)),
                )
                .push(
                    button::primary(&mut self.sign_button, button::button_content(None, "Sign"))
                        .on_press(Message::SpendTx(SpendTxMessage::SelectSign)),
                )
                .push(
                    button::transparent(
                        &mut self.broadcast_button,
                        button::button_content(None, "Broadcast"),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectBroadcast)),
                )
                .push(
                    button::transparent(
                        &mut self.delete_button,
                        button::button_content(None, "Delete"),
                    )
                    .on_press(Message::SpendTx(SpendTxMessage::SelectDelete)),
                ),
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
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    delete_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionDeleteView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            delete_button: iced::button::State::new(),
            confirm_button: iced::button::State::new(),
        }
    }

    pub fn view(
        &mut self,
        processing: &bool,
        success: &bool,
        warning: Option<&Error>,
    ) -> Element<Message> {
        let button_delete_action = if *processing {
            button::important(
                &mut self.confirm_button,
                button::button_content(None, "Deleting"),
            )
        } else if *success {
            button::success(
                &mut self.confirm_button,
                button::button_content(None, "Deleted"),
            )
        } else {
            button::important(
                &mut self.confirm_button,
                button::button_content(None, "Delete transaction"),
            )
            .on_press(Message::SpendTx(SpendTxMessage::Delete))
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

        let mut button_broadcast = button::transparent(
            &mut self.broadcast_button,
            button::button_content(None, "Broadcast"),
        );
        if !*success {
            button_broadcast =
                button_broadcast.on_press(Message::SpendTx(SpendTxMessage::SelectBroadcast));
        }

        let mut button_delete = button::primary(
            &mut self.delete_button,
            button::button_content(None, "Delete"),
        );
        if !*success {
            button_delete = button_delete.on_press(Message::SpendTx(SpendTxMessage::SelectDelete));
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
                    Row::new()
                        .push(button_share)
                        .push(button_sign)
                        .push(button_broadcast)
                        .push(button_delete),
                )
                .push(
                    card::white(Container::new(
                        col_action
                            .push(text::simple(
                                "Are you sure you want to delete this transaction?",
                            ))
                            .push(button_delete_action)
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
pub struct SpendTransactionBroadcastView {
    share_button: iced::button::State,
    sign_button: iced::button::State,
    broadcast_button: iced::button::State,
    delete_button: iced::button::State,
    confirm_button: iced::button::State,
}

impl SpendTransactionBroadcastView {
    pub fn new() -> Self {
        Self {
            share_button: iced::button::State::new(),
            sign_button: iced::button::State::new(),
            broadcast_button: iced::button::State::new(),
            delete_button: iced::button::State::new(),
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

        let mut button_delete = button::transparent(
            &mut self.delete_button,
            button::button_content(None, "Delete"),
        );
        if !*success {
            button_delete = button_delete.on_press(Message::SpendTx(SpendTxMessage::SelectDelete));
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
                    Row::new()
                        .push(button_share)
                        .push(button_sign)
                        .push(button_broadcast)
                        .push(button_delete),
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

    pub fn view(&mut self, ctx: &Context, tx: &model::SpendTx) -> Element<SpendTxMessage> {
        let amount = tx
            .psbt
            .global
            .unsigned_tx
            .output
            .iter()
            .fold(0, |acc, output| acc + output.value);
        button::white_card_button(
            &mut self.select_button,
            Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::pending_spent_tx())
                                .push(Column::new().push(text::bold(text::small(&format!(
                                    "txid: {}",
                                    tx.psbt.global.unsigned_tx.txid().to_string()
                                )))))
                                .spacing(20),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(amount),
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
        .on_press(SpendTxMessage::Select(tx.psbt.clone()))
        .width(Length::Fill)
        .into()
    }
}
