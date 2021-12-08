use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use iced::{
    scrollable, text_input, Align, Checkbox, Column, Container, Element, Length, Row, Space,
    TextInput,
};

use revault_ui::{
    component::{
        button, card, form, scroll, separation, text::Text, ContainerBackgroundStyle, ProgressBar,
    },
    icon::trash_icon,
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{InputMessage, Message, RecipientMessage, SpendTxMessage},
        view::layout,
    },
    daemon::{client::Client, model},
};

#[derive(Debug)]
pub struct ManagerImportTransactionView {
    modal: layout::Modal,
    psbt_input: iced::text_input::State,
    import_button: iced::button::State,
}

impl ManagerImportTransactionView {
    pub fn new() -> Self {
        ManagerImportTransactionView {
            modal: layout::Modal::new(),
            psbt_input: iced::text_input::State::new(),
            import_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        psbt_input: &form::Value<String>,
        psbt_imported: Option<&Psbt>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .spacing(20)
            .push(Text::new("Import spend transaction").bold())
            .push(Text::new("Enter PSBT:"))
            .push(
                form::Form::new(&mut self.psbt_input, "PSBT", psbt_input, |p| {
                    Message::SpendTx(SpendTxMessage::PsbtEdited(p))
                })
                .warning("Please enter a valid PSBT")
                .size(20)
                .padding(10)
                .render(),
            );

        if let Some(psbt) = psbt_imported {
            col = col.push(
                card::success(Container::new(
                    Column::new()
                        .align_items(Align::Center)
                        .push(Text::new("Transaction imported"))
                        .push(
                            button::success(
                                &mut self.import_button,
                                button::button_content(None, "See transaction detail"),
                            )
                            .on_press(Message::SpendTx(SpendTxMessage::Select(psbt.clone()))),
                        )
                        .spacing(20),
                ))
                .align_x(Align::Center)
                .width(Length::Fill),
            );
        } else {
            col = col.push(
                button::primary(
                    &mut self.import_button,
                    button::button_content(None, "Import transaction"),
                )
                .on_press(Message::SpendTx(SpendTxMessage::Import)),
            );
        }

        self.modal.view(
            ctx,
            warning,
            Container::new(card::white(col).max_width(1000))
                .align_x(Align::Center)
                .width(Length::Fill),
            None,
            Message::Menu(Menu::Home),
        )
    }
}

#[derive(Debug)]
pub struct ManagerSendWelcomeView {
    modal: layout::Modal,
    create_transaction_button: iced::button::State,
    import_transaction_button: iced::button::State,
}

impl ManagerSendWelcomeView {
    pub fn new() -> Self {
        ManagerSendWelcomeView {
            modal: layout::Modal::new(),
            create_transaction_button: iced::button::State::new(),
            import_transaction_button: iced::button::State::new(),
        }
    }

    pub fn view<C: Client>(&mut self, ctx: &Context<C>) -> Element<'_, Message> {
        self.modal.view(
            ctx,
            None,
            Container::new(
                Column::new()
                    .push(
                        button::primary(
                            &mut self.create_transaction_button,
                            button::button_content(None, "Initiate a spending"),
                        )
                        .on_press(Message::Next),
                    )
                    .push(
                        button::primary(
                            &mut self.import_transaction_button,
                            button::button_content(None, "Take part in a spending"),
                        )
                        .on_press(Message::SpendTx(SpendTxMessage::Import)),
                    )
                    .align_items(Align::Center)
                    .spacing(20),
            )
            .width(Length::Fill)
            .align_x(Align::Center),
            None,
            Message::Menu(Menu::Home),
        )
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
        let header = Column::new()
            .push(
                Row::new()
                    .push(Column::new().width(Length::Fill))
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
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(0))
            .align_items(Align::Center);

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
                    Container::new(Text::new("Add recipient"))
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
                    Container::new(Text::new("Continue"))
                        .width(Length::Units(200))
                        .align_x(Align::Center)
                        .padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(Text::new("Continue"))
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
                    Column::new()
                        .push(
                            Container::new(Text::new("Add recipients").bold())
                                .padding(20)
                                .width(Length::Fill)
                                .align_x(Align::Center),
                        )
                        .push(scroll(
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
                        ))
                        .height(Length::Fill),
                )
                .push(if no_duplicate {
                    Container::new(footer)
                } else {
                    Container::new(
                        Column::new()
                            .push(
                                Container::new(card::alert_warning(Container::new(Text::new(
                                    "Please merge recipients with the same address",
                                ))))
                                .width(Length::Fill)
                                .align_x(Align::Center),
                            )
                            .spacing(20)
                            .push(footer),
                    )
                })
                .height(Length::Fill)
                .spacing(30),
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

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        inputs: Vec<Element<'a, Message>>,
        input_amount: u64,
        output_amount: u64,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let header = Column::new()
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                button::transparent(
                                    &mut self.back_button,
                                    Container::new(Text::new("< Go back"))
                                        .padding(10)
                                        .align_x(Align::Center),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
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
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(2))
            .align_items(Align::Center);

        let mut footer = Column::new().spacing(10).align_items(Align::Center);
        if let Some(error) = warning {
            footer = footer.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }
        if input_amount < output_amount {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(Text::new(&format!(
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
                    Container::new(Text::new("Continue"))
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
                    Column::new()
                        .push(
                            Container::new(
                                Text::new(&format!(
                                    "Select coins worth at least {} {}",
                                    &ctx.converter.converts(output_amount),
                                    ctx.converter.unit
                                ))
                                .bold(),
                            )
                            .padding(20)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                        )
                        .push(scroll(
                            &mut self.scroll,
                            Container::new(
                                Column::new()
                                    .push(
                                        Container::new(
                                            Column::with_children(inputs)
                                                .spacing(20)
                                                .max_width(1000)
                                                .width(Length::Fill)
                                                .align_items(Align::Center),
                                        )
                                        .width(Length::Fill)
                                        .align_x(Align::Center),
                                    )
                                    .align_items(Align::Center)
                                    .spacing(20),
                            ),
                        ))
                        .height(Length::Fill),
                )
                .push(
                    Column::new()
                        .push(footer)
                        .width(Length::Fill)
                        .align_items(Align::Center),
                )
                .height(Length::Fill)
                .spacing(30),
        )
        .style(ContainerBackgroundStyle)
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub fn manager_send_input_view<'a, C: Client>(
    ctx: &Context<C>,
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
                    .push(Text::new(&format!("{}", ctx.converter.converts(*amount))).bold())
                    .push(Text::new(&ctx.converter.unit.to_string()).small())
                    .align_items(Align::Center),
            )
            .width(Length::Fill),
        )
        .push(
            Column::new()
                .push(Text::new(outpoint).bold().small())
                .width(Length::Shrink),
        )
        .align_items(Align::Center)
        .spacing(20);
    card::white(Container::new(row)).width(Length::Fill).into()
}

#[derive(Debug)]
pub struct ManagerSelectFeeView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    back_button: iced::button::State,
    generate_button: iced::button::State,
    feerate_input: iced::text_input::State,
}

impl ManagerSelectFeeView {
    pub fn new() -> Self {
        ManagerSelectFeeView {
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
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
        let header = Column::new()
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                button::transparent(
                                    &mut self.back_button,
                                    Container::new(Text::new("< Go back"))
                                        .padding(10)
                                        .width(Length::Fill)
                                        .align_x(Align::Center),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
                    .push(
                        Column::new()
                            .push(Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Home)),
                            ))
                            .width(Length::Shrink)
                            .align_items(Align::End),
                    )
                    .width(Length::Fill)
                    .padding(10)
                    .spacing(10),
            )
            .push(
                Container::new(ProgressBar::spend_bar().draw(1))
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .align_items(Align::Center);

        let fee_button = if valid_feerate {
            button::primary(
                &mut self.generate_button,
                Container::new(Text::new("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .align_x(Align::Center),
            )
            .on_press(Message::Next)
        } else {
            button::primary_disable(
                &mut self.generate_button,
                Container::new(Text::new("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .align_x(Align::Center),
            )
        };

        let mut col_fee = Column::new()
            .push(
                Column::new()
                    .push(
                        Container::new(Text::new("Select fee").bold())
                            .padding(20)
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    )
                    .push(
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
                            .push(Text::new("sats/vbyte"))
                            .spacing(5)
                            .align_items(Align::Center),
                    )
                    .align_items(Align::Center),
            )
            .height(Length::Fill)
            .align_items(Align::Center);

        if let Some(error) = warning {
            col_fee = col_fee.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        let col = Column::new()
            .push(header)
            .push(
                Column::new()
                    .push(col_fee)
                    .push(fee_button)
                    .align_items(Align::Center)
                    .max_width(1000),
            )
            .align_items(Align::Center)
            .spacing(30);

        Container::new(col)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub fn spend_tx_with_feerate_view<'a, T: 'a, C: Client>(
    ctx: &Context<C>,
    inputs: &[model::Vault],
    psbt: &Psbt,
    change_index: Option<usize>,
    cpfp_index: usize,
    feerate: Option<&u32>,
) -> Container<'a, T> {
    let mut total_fees = 0;
    let mut col_input = Column::new()
        .push(
            Text::new(&format!(
                "{} {} consumed",
                inputs.len(),
                if inputs.len() == 1 { "Vault" } else { "Vaults" }
            ))
            .bold(),
        )
        .spacing(10);

    for input in inputs {
        total_fees += input.amount;
        col_input = col_input.push(card::simple(Container::new(
            Row::new()
                .push(
                    Container::new(Text::new(&input.address.to_string()).small())
                        .width(Length::Fill),
                )
                .push(
                    Container::new(
                        Text::new(&format!("{}", ctx.converter.converts(input.amount)))
                            .bold()
                            .small(),
                    )
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
        .push(
            Text::new(&format!(
                "{} {}",
                number_recipients,
                if number_recipients == 1 {
                    "Recipient"
                } else {
                    "Recipients"
                }
            ))
            .bold(),
        )
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
                .push(Container::new(Text::new(&addr.to_string()).small()).width(Length::Fill))
                .push(
                    Container::new(
                        Text::new(&format!("{}", ctx.converter.converts(output.value)))
                            .bold()
                            .small(),
                    )
                    .width(Length::Shrink),
                )
                .spacing(5)
                .align_items(Align::Center),
        )));
    }
    let mut column_fee = Column::new();
    if let Some(feerate) = feerate {
        column_fee = column_fee.push(
            Row::new()
                .push(Text::new("Feerate: "))
                .push(Text::new(&format!("{} sats/vbyte", feerate)).bold()),
        )
    }

    let right_column = if let Some(index) = change_index {
        let change_output = &psbt.global.unsigned_tx.output[index];
        let addr =
            bitcoin::Address::from_script(&change_output.script_pubkey, ctx.network).unwrap();
        Column::new()
            .push(
                Column::new()
                    .push(Text::new("Change").bold())
                    .push(card::simple(Container::new(
                        Row::new()
                            .push(
                                Container::new(Text::new(&addr.to_string()).small())
                                    .width(Length::Fill),
                            )
                            .push(
                                Container::new(
                                    Text::new(&format!(
                                        "{}",
                                        ctx.converter.converts(change_output.value)
                                    ))
                                    .bold()
                                    .small(),
                                )
                                .width(Length::Shrink),
                            )
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
                        .push(Text::new("Total fees: "))
                        .push(Text::new(&format!("{}", ctx.converter.converts(total_fees))).bold())
                        .push(Text::new(&format!(" {}", ctx.converter.unit))),
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
pub struct ManagerStepSignView {
    modal: layout::Modal,
    cancel_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerStepSignView {
    pub fn new() -> Self {
        ManagerStepSignView {
            modal: layout::Modal::new(),
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        inputs: &[model::Vault],
        psbt: &Psbt,
        cpfp_index: usize,
        change_index: Option<usize>,
        feerate: &u32,
        warning: Option<&Error>,
        signer: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let header = Column::new()
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                button::transparent(
                                    &mut self.back_button,
                                    Container::new(Text::new("< Go back"))
                                        .padding(10)
                                        .align_x(Align::Center),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
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
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(3))
            .align_items(Align::Center);

        let mut col = Column::new()
            .push(
                Container::new(Text::new("Sign transaction").bold())
                    .width(Length::Fill)
                    .align_x(Align::Center)
                    .padding(20),
            )
            .push(
                Column::new()
                    .push(spend_tx_with_feerate_view(
                        ctx,
                        inputs,
                        psbt,
                        change_index,
                        cpfp_index,
                        Some(feerate),
                    ))
                    .push(
                        card::white(Container::new(signer))
                            .align_x(Align::Center)
                            .width(Length::Fill),
                    )
                    .spacing(20),
            );

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }

        Container::new(
            Column::new()
                .push(header)
                .push(
                    Container::new(
                        Column::new()
                            .push(col)
                            .align_items(Align::Center)
                            .max_width(1000),
                    )
                    .align_x(Align::Center),
                )
                .align_items(Align::Center)
                .spacing(30)
                .padding(20),
        )
        .style(ContainerBackgroundStyle)
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

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
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
                        .push(ProgressBar::spend_bar().draw(4))
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
                            .push(Text::new("Your transaction has been saved"))
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
