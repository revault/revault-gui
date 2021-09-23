use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{
    scrollable, text_input, Align, Checkbox, Column, Container, Element, Length, Row, Space,
    TextInput,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{InputMessage, Message, RecipientMessage, SpendTxMessage},
    },
    daemon::model,
    ui::{
        component::{button, card, form, scroll, separation, text, ContainerBackgroundStyle},
        icon::trash_icon,
    },
};

#[derive(Debug)]
pub struct ManagerImportTransactionView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    psbt_input: iced::text_input::State,
    import_button: iced::button::State,
}

impl ManagerImportTransactionView {
    pub fn new() -> Self {
        ManagerImportTransactionView {
            cancel_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            psbt_input: iced::text_input::State::new(),
            import_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        psbt_input: &str,
        psbt_imported: Option<&Psbt>,
        warning: Option<&String>,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .spacing(20)
            .push(text::bold(text::simple("Import spend transaction")))
            .push(text::simple("Enter PSBT:"))
            .push(
                TextInput::new(&mut self.psbt_input, "Signed PSBT", &psbt_input, |p| {
                    Message::SpendTx(SpendTxMessage::PsbtEdited(p))
                })
                .size(15)
                .width(Length::Fill)
                .padding(10),
            );
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(error))))
        }

        if let Some(psbt) = psbt_imported {
            col = col.push(card::success(Container::new(
                Column::new()
                    .push(text::simple("Transaction imported"))
                    .push(
                        button::success(
                            &mut self.import_button,
                            button::button_content(None, "See transaction detail"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::Select(psbt.clone()))),
                    )
                    .spacing(20),
            )));
        } else {
            col = col.push(
                button::primary(
                    &mut self.import_button,
                    button::button_content(None, "Import transaction"),
                )
                .on_press(Message::SpendTx(SpendTxMessage::Import)),
            );
        }
        Container::new(scroll(
            &mut self.scroll,
            Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Home)),
                            )
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(
                            Column::new()
                                .push(card::white(Container::new(col)).width(Length::Fill))
                                .spacing(20),
                        )
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
pub struct ManagerSendWelcomeView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    crate_transaction_button: iced::button::State,
    import_transaction_button: iced::button::State,
}

impl ManagerSendWelcomeView {
    pub fn new() -> Self {
        ManagerSendWelcomeView {
            cancel_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            crate_transaction_button: iced::button::State::new(),
            import_transaction_button: iced::button::State::new(),
        }
    }

    pub fn view(&mut self) -> Element<'_, Message> {
        Container::new(scroll(
            &mut self.scroll,
            Container::new(
                Column::new()
                    .push(
                        Row::new().push(Column::new().width(Length::Fill)).push(
                            Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Home)),
                            )
                            .width(Length::Shrink),
                        ),
                    )
                    .push(
                        Container::new(
                            Column::new()
                                .push(
                                    button::primary(
                                        &mut self.crate_transaction_button,
                                        button::button_content(None, "Initiate a spending"),
                                    )
                                    .on_press(Message::Next),
                                )
                                .push(
                                    button::primary(
                                        &mut self.import_transaction_button,
                                        button::button_content(None, "Participate in a spending"),
                                    )
                                    .on_press(Message::SpendTx(SpendTxMessage::Import)),
                                )
                                .spacing(20),
                        )
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
pub struct ManagerSelectOutputsView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    new_output_button: iced::button::State,
}

impl ManagerSelectOutputsView {
    pub fn new() -> Self {
        ManagerSelectOutputsView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            new_output_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        selected_outputs: Vec<Element<'a, Message>>,
        valid: bool,
        no_duplicate: bool,
    ) -> Element<'a, Message> {
        let header = Row::new()
            .push(Column::new().width(Length::Fill))
            .push(crate::ui::component::ProgressBar::spend_bar().draw(0))
            .push(
                Column::new()
                    .push(
                        button::close_button(&mut self.cancel_button)
                            .on_press(Message::Menu(Menu::Home)),
                    )
                    .align_items(Align::End)
                    .width(Length::Fill),
            )
            .width(Length::Fill)
            .align_items(Align::End)
            .padding(10)
            .spacing(10);
        let mut col_outputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(Align::Center);
        for (i, element) in selected_outputs.into_iter().enumerate() {
            if i > 0 {
                col_outputs = col_outputs.push(separation().width(Length::Fill));
            }
            col_outputs = col_outputs.push(element);
        }
        let element: Element<_> = col_outputs.max_width(1000).into();

        let mut footer = Row::new()
            .spacing(20)
            .push(Space::with_width(Length::Fill))
            .push(
                button::cancel(
                    &mut self.new_output_button,
                    Container::new(text::simple("Add recipient"))
                        .width(Length::Units(200))
                        .align_x(Align::Center)
                        .padding(10),
                )
                .on_press(Message::AddRecipient),
            );

        if valid {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue"))
                        .width(Length::Units(200))
                        .align_x(Align::Center)
                        .padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue"))
                    .width(Length::Units(200))
                    .align_x(Align::Center)
                    .padding(10),
            )))
        }
        footer = footer.push(Space::with_width(Length::Fill));

        Container::new(
            Column::new()
                .push(header)
                .push(
                    Container::new(text::bold(text::simple("Add recipients")))
                        .width(Length::Fill)
                        .align_x(Align::Center),
                )
                .push(
                    scroll(
                        &mut self.scroll,
                        Container::new(
                            Column::new()
                                .push(
                                    Container::new(element)
                                        .width(Length::Fill)
                                        .align_x(Align::Center),
                                )
                                .spacing(20),
                        ),
                    )
                    .height(Length::FillPortion(4)),
                )
                .push(if no_duplicate {
                    Container::new(footer)
                } else {
                    Container::new(
                        Column::new()
                            .push(
                                Container::new(card::alert_warning(Container::new(text::simple(
                                    "Please merge recipients with the same address",
                                ))))
                                .width(Length::Fill)
                                .align_x(Align::Center),
                            )
                            .spacing(20)
                            .push(footer),
                    )
                })
                .spacing(20),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct ManagerSendOutputView {
    address_input: text_input::State,
    amount_input: text_input::State,
    delete_button: iced::button::State,
}

impl ManagerSendOutputView {
    pub fn new() -> Self {
        Self {
            address_input: text_input::State::focused(),
            amount_input: text_input::State::new(),
            delete_button: iced::button::State::new(),
        }
    }
    pub fn view(
        &mut self,
        address: &form::Value<String>,
        amount: &form::Value<String>,
    ) -> Element<RecipientMessage> {
        Row::new()
            .push(
                form::Form::new(
                    &mut self.address_input,
                    "Address",
                    &address,
                    RecipientMessage::AddressEdited,
                )
                .warning("Please enter a valid bitcoin address")
                .padding(10)
                .render()
                .width(Length::FillPortion(2)),
            )
            .push(
                form::Form::new(
                    &mut self.amount_input,
                    "Amount in BTC, ex: 0.123",
                    &amount,
                    RecipientMessage::AmountEdited,
                )
                .warning("Please enter a valid amount")
                .padding(10)
                .render()
                .width(Length::FillPortion(1)),
            )
            .push(
                Container::new(
                    button::transparent(&mut self.delete_button, Container::new(trash_icon()))
                        .on_press(RecipientMessage::Delete),
                )
                .width(Length::Shrink)
                .align_x(Align::End),
            )
            .spacing(20)
            .into()
    }
}

#[derive(Debug)]
pub struct ManagerSelectInputsView {
    scroll: scrollable::State,
    back_button: iced::button::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    new_output_button: iced::button::State,
}

impl ManagerSelectInputsView {
    pub fn new() -> Self {
        ManagerSelectInputsView {
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            new_output_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        inputs: Vec<Element<'a, Message>>,
        input_amount: u64,
        output_amount: u64,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let header = Row::new()
            .push(
                Column::new()
                    .push(
                        button::transparent(
                            &mut self.back_button,
                            Container::new(text::simple("< Go back"))
                                .padding(10)
                                .width(Length::Units(100))
                                .align_x(Align::Center),
                        )
                        .on_press(Message::Previous),
                    )
                    .width(Length::Fill),
            )
            .push(crate::ui::component::ProgressBar::spend_bar().draw(2))
            .push(
                Column::new()
                    .push(
                        button::close_button(&mut self.cancel_button)
                            .on_press(Message::Menu(Menu::Home)),
                    )
                    .align_items(Align::End)
                    .width(Length::Fill),
            )
            .width(Length::Fill)
            .align_items(Align::End)
            .padding(10)
            .spacing(10);
        let mut col_inputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(Align::Center);
        for (i, element) in inputs.into_iter().enumerate() {
            if i > 0 {
                col_inputs = col_inputs.push(separation().width(Length::Fill));
            }
            col_inputs = col_inputs.push(element);
        }
        let element: Element<_> = col_inputs.max_width(1000).into();

        let mut footer = Column::new().spacing(10).align_items(Align::Center);
        if let Some(error) = warning {
            footer = footer.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }
        if input_amount < output_amount {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple(&format!(
                    "Missing {} {}",
                    &ctx.converter.converts(output_amount - input_amount),
                    ctx.converter.unit
                )))
                .width(Length::Units(200))
                .align_x(Align::Center)
                .padding(10),
            )));
        } else {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue"))
                        .padding(10)
                        .width(Length::Units(200))
                        .align_x(Align::Center),
                )
                .on_press(Message::SpendTx(SpendTxMessage::Generate)),
            ));
        }

        Container::new(
            Column::new()
                .push(header)
                .push(
                    Container::new(text::bold(text::simple(&format!(
                        "Select coins worth at least {} {}",
                        &ctx.converter.converts(output_amount),
                        ctx.converter.unit
                    ))))
                    .width(Length::Fill)
                    .align_x(Align::Center),
                )
                .push(
                    scroll(
                        &mut self.scroll,
                        Container::new(
                            Column::new()
                                .push(
                                    Container::new(element)
                                        .width(Length::Fill)
                                        .align_x(Align::Center),
                                )
                                .align_items(Align::Center)
                                .spacing(20),
                        ),
                    )
                    .height(Length::FillPortion(4)),
                )
                .push(
                    Column::new()
                        .push(footer)
                        .width(Length::Fill)
                        .align_items(Align::Center),
                )
                .spacing(20),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub fn manager_send_input_view<'a>(
    ctx: &Context,
    outpoint: &str,
    amount: &u64,
    selected: bool,
) -> Element<'a, InputMessage> {
    let checkbox = Checkbox::new(selected, "", InputMessage::Selected).text_size(10);
    let row = Row::new()
        .push(checkbox)
        .push(
            Container::new(
                Row::new()
                    .push(text::bold(text::simple(&format!(
                        "{}",
                        ctx.converter.converts(*amount)
                    ))))
                    .push(text::small(&ctx.converter.unit.to_string()))
                    .align_items(Align::Center),
            )
            .width(Length::Fill),
        )
        .push((Column::new().push(text::bold(text::small(outpoint)))).width(Length::Shrink))
        .align_items(Align::Center)
        .spacing(20);
    card::white(Container::new(row)).width(Length::Fill).into()
}

#[derive(Debug)]
pub struct ManagerSelectFeeView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    back_button: iced::button::State,
    slider: iced::slider::State,
    generate_button: iced::button::State,
    feerate_input: iced::text_input::State,
}

impl ManagerSelectFeeView {
    pub fn new() -> Self {
        ManagerSelectFeeView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
            slider: iced::slider::State::new(),
            generate_button: iced::button::State::new(),
            feerate_input: iced::text_input::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        feerate: Option<u32>,
        valid_feerate: bool,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let header = Row::new()
            .push(
                Column::new()
                    .push(
                        button::transparent(
                            &mut self.back_button,
                            Container::new(text::simple("< Go back"))
                                .padding(10)
                                .width(Length::Units(100))
                                .align_x(Align::Center),
                        )
                        .on_press(Message::Previous),
                    )
                    .width(Length::Fill),
            )
            .push(crate::ui::component::ProgressBar::spend_bar().draw(1))
            .push(
                Column::new()
                    .push(Container::new(
                        button::close_button(&mut self.cancel_button)
                            .on_press(Message::Menu(Menu::Home)),
                    ))
                    .width(Length::Fill)
                    .align_items(Align::End),
            )
            .width(Length::Fill)
            .align_items(Align::End)
            .padding(10)
            .spacing(10);
        let fee_button = if valid_feerate {
            button::primary(
                &mut self.generate_button,
                Container::new(text::simple("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .align_x(Align::Center),
            )
            .on_press(Message::Next)
        } else {
            button::primary_disable(
                &mut self.generate_button,
                Container::new(text::simple("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .align_x(Align::Center),
            )
        };

        let mut col_fee = Column::new()
            .push(
                Container::new(text::bold(text::simple("Select fee")))
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(
                            TextInput::new(
                                &mut self.feerate_input,
                                "",
                                &feerate
                                    .map(|f| f.to_string())
                                    .unwrap_or_else(|| "".to_string()),
                                |f| Message::SpendTx(SpendTxMessage::FeerateEdited(f)),
                            )
                            .width(Length::Units(70))
                            .padding(10),
                        )
                        .push(text::simple("sats/vbyte"))
                        .spacing(5)
                        .align_items(Align::Center),
                )
                .height(Length::Fill),
            )
            .spacing(20)
            .align_items(Align::Center);

        if let Some(error) = warning {
            col_fee = col_fee.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }

        col_fee = col_fee.push(fee_button);

        let col = Column::new()
            .push(header)
            .push(
                Container::new(
                    Column::new()
                        .push(col_fee)
                        .align_items(Align::Center)
                        .max_width(1000),
                )
                .align_x(Align::Center),
            )
            .align_items(Align::Center)
            .spacing(20)
            .padding(20);

        Container::new(col)
            .style(ContainerBackgroundStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub fn spend_tx_with_feerate_view<'a, T: 'a>(
    ctx: &Context,
    inputs: &[model::Vault],
    psbt: &Psbt,
    change_index: Option<usize>,
    cpfp_index: usize,
    feerate: Option<&u32>,
) -> Container<'a, T> {
    let mut total_fees = 0;
    let mut col_input = Column::new()
        .push(text::bold(text::simple(&format!(
            "{} {} consumed",
            inputs.len(),
            if inputs.len() == 1 { "Vault" } else { "Vaults" }
        ))))
        .spacing(10);

    for input in inputs {
        total_fees += input.amount;
        col_input = col_input.push(card::simple(Container::new(
            Row::new()
                .push(Container::new(text::small(&input.address.to_string())).width(Length::Fill))
                .push(
                    Container::new(text::bold(text::small(&format!(
                        "{}",
                        ctx.converter.converts(input.amount),
                    ))))
                    .width(Length::Shrink),
                )
                .spacing(5)
                .align_items(Align::Center),
        )));
    }

    // number recipients = number output - change output - cpfp output
    let number_recipients = if change_index.is_some() {
        psbt.global.unsigned_tx.output.len() - 2
    } else {
        psbt.global.unsigned_tx.output.len() - 1
    };

    let mut col_output = Column::new()
        .push(text::bold(text::simple(&format!(
            "{} {}",
            number_recipients,
            if number_recipients == 1 {
                "Recipient"
            } else {
                "Recipients"
            }
        ))))
        .spacing(10);
    for (i, output) in psbt.global.unsigned_tx.output.iter().enumerate() {
        if i == cpfp_index {
            continue;
        }

        if total_fees > output.value {
            total_fees -= output.value;
        } else {
            total_fees = 0;
        }

        if Some(i) == change_index {
            continue;
        }

        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network).unwrap();
        col_output = col_output.push(card::simple(Container::new(
            Row::new()
                .push(Container::new(text::small(&addr.to_string())).width(Length::Fill))
                .push(Container::new(
                    text::bold(text::small(&format!(
                        "{}",
                        ctx.converter.converts(output.value)
                    )))
                    .width(Length::Shrink),
                ))
                .spacing(5)
                .align_items(Align::Center),
        )));
    }
    let mut column_fee = Column::new();
    if let Some(feerate) = feerate {
        column_fee = column_fee.push(
            Row::new()
                .push(text::simple("Feerate: "))
                .push(text::bold(text::simple(&format!("{} sats/vbyte", feerate)))),
        )
    }

    let right_column = if let Some(index) = change_index {
        let change_output = &psbt.global.unsigned_tx.output[index];
        let addr =
            bitcoin::Address::from_script(&change_output.script_pubkey, ctx.network).unwrap();
        Column::new()
            .push(
                Column::new()
                    .push(text::bold(text::simple("Change")))
                    .push(card::simple(Container::new(
                        Row::new()
                            .push(
                                Container::new(text::small(&addr.to_string())).width(Length::Fill),
                            )
                            .push(Container::new(
                                text::bold(text::small(&format!(
                                    "{}",
                                    ctx.converter.converts(change_output.value)
                                )))
                                .width(Length::Shrink),
                            ))
                            .spacing(5)
                            .align_items(Align::Center),
                    )))
                    .spacing(10),
            )
            .push(col_output)
            .spacing(10)
    } else {
        col_output
    };

    Container::new(
        Column::new()
            .push(
                column_fee.push(
                    Row::new()
                        .push(text::simple("Total fees: "))
                        .push(text::bold(text::simple(&format!(
                            "{}",
                            ctx.converter.converts(total_fees)
                        ))))
                        .push(text::simple(&format!(" {}", ctx.converter.unit))),
                ),
            )
            .push(
                Row::new()
                    .push(col_input.width(Length::FillPortion(1)))
                    .push(right_column.width(Length::FillPortion(1)))
                    .spacing(20),
            )
            .spacing(20),
    )
}

#[derive(Debug)]
pub struct ManagerSignView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerSignView {
    pub fn new() -> Self {
        ManagerSignView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        inputs: &[model::Vault],
        psbt: &Psbt,
        cpfp_index: usize,
        change_index: Option<usize>,
        feerate: &u32,
        warning: Option<&Error>,
        signer: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let header = Row::new()
            .push(
                Column::new()
                    .push(
                        button::transparent(
                            &mut self.back_button,
                            Container::new(text::simple("< Go back"))
                                .padding(10)
                                .width(Length::Units(100))
                                .align_x(Align::Center),
                        )
                        .on_press(Message::Previous),
                    )
                    .width(Length::Fill),
            )
            .push(crate::ui::component::ProgressBar::spend_bar().draw(3))
            .push(
                Column::new()
                    .push(
                        button::close_button(&mut self.cancel_button)
                            .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Fill)
                    .align_items(Align::End),
            )
            .align_items(Align::End)
            .width(Length::Fill)
            .align_items(Align::End)
            .padding(10)
            .spacing(10);
        let mut col = Column::new()
            .push(spend_tx_with_feerate_view(
                ctx,
                inputs,
                psbt,
                change_index,
                cpfp_index,
                Some(feerate),
            ))
            .spacing(20)
            .max_width(1000);
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::small(
                &error.to_string(),
            ))));
        }
        col = col.push(card::white(Container::new(signer)));
        Container::new(
            Column::new()
                .push(header)
                .push(
                    Container::new(text::bold(text::simple("Sign transaction")))
                        .width(Length::Fill)
                        .align_x(Align::Center),
                )
                .push(
                    scroll(
                        &mut self.scroll,
                        Container::new(col)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .align_items(Align::Center)
                    .width(Length::Fill),
                )
                .spacing(20),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug)]
pub struct ManagerSpendTransactionCreatedView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerSpendTransactionCreatedView {
    pub fn new() -> Self {
        ManagerSpendTransactionCreatedView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        inputs: &[model::Vault],
        psbt: &Psbt,
        cpfp_index: usize,
        change_index: Option<usize>,
        feerate: &u32,
    ) -> Element<'a, Message> {
        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(Column::new().width(Length::Fill))
                        .push(crate::ui::component::ProgressBar::spend_bar().draw(4))
                        .push(
                            Column::new()
                                .push(
                                    button::close_button(&mut self.cancel_button)
                                        .on_press(Message::Menu(Menu::Home)),
                                )
                                .width(Length::Fill)
                                .align_items(Align::End),
                        )
                        .align_items(Align::End)
                        .padding(10)
                        .spacing(10),
                )
                .push(
                    scroll(
                        &mut self.scroll,
                        Container::new(
                            Column::new()
                                .push(spend_tx_with_feerate_view(
                                    ctx,
                                    inputs,
                                    psbt,
                                    change_index,
                                    cpfp_index,
                                    Some(feerate),
                                ))
                                .spacing(20)
                                .max_width(1000),
                        )
                        .width(Length::Fill)
                        .align_x(Align::Center),
                    )
                    .align_items(Align::Center)
                    .width(Length::Fill),
                )
                .push(
                    card::success(Container::new(
                        Column::new()
                            .push(text::simple("Your transaction has been saved"))
                            .push(
                                button::success(
                                    &mut self.next_button,
                                    button::button_content(None, "Continue"),
                                )
                                .on_press(Message::SpendTx(SpendTxMessage::Select(psbt.clone()))),
                            )
                            .spacing(20)
                            .align_items(Align::Center),
                    ))
                    .padding(20)
                    .width(Length::Fill)
                    .align_x(Align::Center),
                )
                .spacing(20),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
