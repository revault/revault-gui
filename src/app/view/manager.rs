use bitcoin::{util::psbt::PartiallySignedTransaction as Psbt, Amount};

use iced::{
    alignment, pick_list, scrollable, text_input, tooltip, Alignment, Column, Container, Element,
    Length, Row, Space, TextInput, Tooltip,
};

use revaultd::revault_tx::transactions::RevaultTransaction;

use revault_ui::{
    color,
    component::{
        badge, button, card, form, scroll, separation, text::Text, ContainerBackgroundStyle,
        ContainerForegroundStyle, ProgressBar, TooltipStyle, TransparentPickListStyle,
    },
    icon::{tooltip_icon, trash_icon},
};

use crate::{
    app::{
        context::Context,
        error::Error,
        menu::Menu,
        message::{InputMessage, Message, RecipientMessage, SpendTxMessage},
        view::{layout, warning::warn},
    },
    daemon::model,
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
            modal: layout::Modal::default(),
            psbt_input: iced::text_input::State::new(),
            import_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
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

        if psbt_imported.is_some() {
            col = col.push(
                card::success(Container::new(
                    Column::new()
                        .align_items(Alignment::Center)
                        .push(Text::new("Transaction imported"))
                        .spacing(20),
                ))
                .center_x()
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
                .center_x()
                .width(Length::Fill),
            None,
            Message::Menu(Menu::Send),
        )
    }
}

#[derive(Debug, Default)]
pub struct ManagerSendView {
    dashboard: layout::Dashboard,
    create_transaction_button: iced::button::State,
    import_transaction_button: iced::button::State,
    pick_filter: pick_list::State<SpendTxsFilter>,
}

impl ManagerSendView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        txs: Vec<Element<'a, Message>>,
        txs_status_filter: &[model::SpendTxStatus],
    ) -> Element<'a, Message> {
        self.dashboard.view(
            ctx,
            None,
            Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                button::primary(
                                    &mut self.create_transaction_button,
                                    button::button_content(None, "Initiate a spending"),
                                )
                                .on_press(Message::Menu(Menu::CreateSpend)),
                            )
                            .push(
                                button::primary(
                                    &mut self.import_transaction_button,
                                    button::button_content(None, "Take part in a spending"),
                                )
                                .on_press(Message::Menu(Menu::ImportSpend)),
                            )
                            .spacing(20),
                    )
                    .push(
                        Row::new()
                            .push(
                                Container::new(
                                    Row::new()
                                        .push(Text::new(&format!(" {}", txs.len())).bold())
                                        .push(Text::new(" transactions")),
                                )
                                .width(Length::Fill),
                            )
                            .push(
                                pick_list::PickList::new(
                                    &mut self.pick_filter,
                                    &SpendTxsFilter::ALL[..],
                                    Some(SpendTxsFilter::new(txs_status_filter)),
                                    |filter| Message::FilterTxs(filter.statuses()),
                                )
                                .text_size(20)
                                .padding(10)
                                .width(Length::Units(200))
                                .style(TransparentPickListStyle),
                            )
                            .align_items(Alignment::Center),
                    )
                    .push(Column::with_children(txs).spacing(5))
                    .spacing(20),
            )
            .width(Length::Fill)
            .center_x(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpendTxsFilter {
    All,
    Current,
    Processing,
    Confirmed,
    Deprecated,
}

impl SpendTxsFilter {
    pub const ALL: [SpendTxsFilter; 5] = [
        SpendTxsFilter::All,
        SpendTxsFilter::Current,
        SpendTxsFilter::Processing,
        SpendTxsFilter::Confirmed,
        SpendTxsFilter::Deprecated,
    ];

    pub fn new(statuses: &[model::SpendTxStatus]) -> SpendTxsFilter {
        if statuses == model::ALL_SPEND_TX_STATUSES {
            SpendTxsFilter::All
        } else if statuses == [model::SpendTxStatus::NonFinal] {
            SpendTxsFilter::Current
        } else if statuses == model::PROCESSING_SPEND_TX_STATUSES {
            SpendTxsFilter::Processing
        } else if statuses == [model::SpendTxStatus::Confirmed] {
            SpendTxsFilter::Confirmed
        } else if statuses == [model::SpendTxStatus::Deprecated] {
            SpendTxsFilter::Deprecated
        } else {
            SpendTxsFilter::Deprecated
        }
    }

    pub fn statuses(&self) -> &'static [model::SpendTxStatus] {
        match self {
            Self::All => &model::ALL_SPEND_TX_STATUSES,
            Self::Current => &[model::SpendTxStatus::NonFinal],
            Self::Processing => &model::PROCESSING_SPEND_TX_STATUSES,
            Self::Confirmed => &[model::SpendTxStatus::Confirmed],
            Self::Deprecated => &[model::SpendTxStatus::Deprecated],
        }
    }
}

impl std::fmt::Display for SpendTxsFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Current => write!(f, "Current"),
            Self::Processing => write!(f, "Processing"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Deprecated => write!(f, "Deprecated"),
        }
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
                                    .on_press(Message::Menu(Menu::Send)),
                            )
                            .align_items(Alignment::End)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .padding(10)
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(0))
            .align_items(Alignment::Center);

        let mut col_outputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(Alignment::Center);
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
                        .center_x()
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
                        .center_x()
                        .padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(Text::new("Continue"))
                    .width(Length::Units(200))
                    .center_x()
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
                                .center_x(),
                        )
                        .push(scroll(
                            &mut self.scroll,
                            Container::new(
                                Column::new()
                                    .push(Container::new(element).width(Length::Fill).center_x())
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
                                .center_x(),
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
                    address,
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
                    amount,
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
                .align_x(alignment::Horizontal::Right),
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
}

impl ManagerSelectInputsView {
    pub fn new() -> Self {
        ManagerSelectInputsView {
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
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
                                        .center_x(),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
                    .push(
                        Column::new()
                            .push(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Send)),
                            )
                            .align_items(Alignment::End)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .padding(10)
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(2))
            .align_items(Alignment::Center);

        let mut footer = Column::new().spacing(10).align_items(Alignment::Center);
        if let Some(error) = warning {
            footer = footer.push(card::alert_warning(Container::new(
                Text::new(&error.to_string()).small(),
            )));
        }
        if input_amount != 0 {
            footer = footer.push(
                Container::new(
                    Row::new()
                        .push(
                            Row::new()
                                .push(
                                    Text::new(
                                        &ctx.converter.converts(Amount::from_sat(input_amount)),
                                    )
                                    .bold(),
                                )
                                .push(Text::new(&format!(" {}", ctx.converter.unit)))
                                .width(Length::Fill),
                        )
                        .push(
                            Container::new(if input_amount > output_amount {
                                button::primary(
                                    &mut self.next_button,
                                    button::button_content(None, "Next"),
                                )
                                .on_press(Message::SpendTx(SpendTxMessage::Generate))
                                .width(Length::Units(200))
                            } else {
                                button::primary(
                                    &mut self.next_button,
                                    button::button_content(None, "Next"),
                                )
                                .width(Length::Units(200))
                            })
                            .width(Length::Shrink),
                        )
                        .align_items(Alignment::Center)
                        .max_width(1000),
                )
                .padding(30)
                .width(Length::Fill)
                .center_x()
                .style(ContainerForegroundStyle),
            )
        }

        Container::new(
            Column::new()
                .push(header.padding(20))
                .push(
                    Column::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(
                                        Text::new(&format!(
                                            "Select coins worth at least {} {}",
                                            &ctx.converter
                                                .converts(Amount::from_sat(output_amount)),
                                            ctx.converter.unit
                                        ))
                                        .bold(),
                                    )
                                    .push(
                                        Tooltip::new(
                                            tooltip_icon().size(15),
                                            "The amounts are the vaults amounts less the miner fees \nand the cpfp outputs of the unvault transaction",
                                            tooltip::Position::Right,
                                        )
                                        .gap(5)
                                        .size(20)
                                        .padding(10)
                                        .style(TooltipStyle),
                                    )
                                    .spacing(10),
                            )
                            .padding(20)
                            .width(Length::Fill)
                            .center_x()
                        )
                        .push(scroll(
                            &mut self.scroll,
                            Container::new(
                                Column::new()
                                    .push(
                                        Container::new(
                                            Column::with_children(inputs)
                                                .spacing(5)
                                                .max_width(1000)
                                                .width(Length::Fill)
                                                .align_items(Alignment::Center),
                                        )
                                        .width(Length::Fill)
                                        .center_x()
                                    )
                                    .align_items(Alignment::Center)
                                    .spacing(20),
                            ),
                        ))
                        .height(Length::Fill),
                )
                .push(
                    Column::new()
                        .push(footer)
                        .width(Length::Fill)
                        .align_items(Alignment::Center),
                )
                .height(Length::Fill),
        )
        .style(ContainerBackgroundStyle)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManagerSendInputView {
    select_button: iced::button::State,
}

impl ManagerSendInputView {
    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        amount: &bitcoin::Amount,
        selected: bool,
    ) -> Element<'a, InputMessage> {
        let row = Row::new()
            .push(
                Container::new(if selected {
                    badge::square_check()
                } else {
                    badge::square()
                })
                .width(Length::Fill),
            )
            .push(
                Container::new(if selected {
                    Row::new()
                        .push(
                            Text::new(&ctx.converter.converts(*amount))
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
                        .push(Text::new(&ctx.converter.converts(*amount)).bold())
                        .push(Text::new(&format!(" {}", ctx.converter.unit)).small())
                        .align_items(Alignment::Center)
                })
                .width(Length::Shrink),
            )
            .align_items(Alignment::Center)
            .spacing(20);
        button::white_card_button(&mut self.select_button, Container::new(row))
            .on_press(InputMessage::Select)
            .width(Length::Fill)
            .into()
    }
}

#[derive(Debug)]
pub struct ManagerSelectFeeView {
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
            generate_button: iced::button::State::new(),
            feerate_input: iced::text_input::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        feerate: Option<u64>,
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
                                        .center_x(),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
                    .push(
                        Column::new()
                            .push(Container::new(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Send)),
                            ))
                            .width(Length::Shrink)
                            .align_items(Alignment::End),
                    )
                    .width(Length::Fill)
                    .padding(10)
                    .spacing(10),
            )
            .push(
                Container::new(ProgressBar::spend_bar().draw(1))
                    .width(Length::Fill)
                    .center_x(),
            )
            .align_items(Alignment::Center);

        let fee_button = if valid_feerate {
            button::primary(
                &mut self.generate_button,
                Container::new(Text::new("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .center_x(),
            )
            .on_press(Message::Next)
        } else {
            button::primary_disable(
                &mut self.generate_button,
                Container::new(Text::new("Continue"))
                    .padding(10)
                    .width(Length::Units(200))
                    .center_x(),
            )
        };

        let mut col_fee = Column::new()
            .push(
                Column::new()
                    .push(
                        Container::new(Text::new("Select fee").bold())
                            .padding(20)
                            .width(Length::Fill)
                            .center_x(),
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
                            .align_items(Alignment::Center),
                    )
                    .align_items(Alignment::Center),
            )
            .height(Length::Fill)
            .align_items(Alignment::Center);

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
                    .align_items(Alignment::Center)
                    .max_width(1000),
            )
            .align_items(Alignment::Center)
            .spacing(30);

        Container::new(col)
            .style(ContainerBackgroundStyle)
            .padding(20)
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
        col_input = col_input.push(card::simple(Container::new(
            Row::new()
                .push(
                    Container::new(Text::new(&input.address.to_string()).small())
                        .width(Length::Fill),
                )
                .push(
                    Container::new(
                        Text::new(&ctx.converter.converts(input.amount))
                            .bold()
                            .small(),
                    )
                    .width(Length::Shrink),
                )
                .spacing(5)
                .align_items(Alignment::Center),
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

        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network()).unwrap();
        col_output = col_output.push(card::simple(Container::new(
            Row::new()
                .push(Container::new(Text::new(&addr.to_string()).small()).width(Length::Fill))
                .push(
                    Container::new(
                        Text::new(&format!(
                            "{}",
                            ctx.converter.converts(Amount::from_sat(output.value))
                        ))
                        .bold()
                        .small(),
                    )
                    .width(Length::Shrink),
                )
                .spacing(5)
                .align_items(Alignment::Center),
        )));
    }

    let right_column = if let Some(index) = change_index {
        let change_output = &psbt.global.unsigned_tx.output[index];
        let addr =
            bitcoin::Address::from_script(&change_output.script_pubkey, ctx.network()).unwrap();
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
                                        ctx.converter
                                            .converts(Amount::from_sat(change_output.value))
                                    ))
                                    .bold()
                                    .small(),
                                )
                                .width(Length::Shrink),
                            )
                            .spacing(5)
                            .align_items(Alignment::Center),
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
            .push(col_input.width(Length::Fill))
            .push(right_column.width(Length::Fill))
            .spacing(20),
    )
}

#[derive(Debug)]
pub struct ManagerStepSignView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerStepSignView {
    pub fn new() -> Self {
        ManagerStepSignView {
            scroll: scrollable::State::new(),
            cancel_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        inputs: &[model::Vault],
        tx: &model::SpendTx,
        feerate: &u64,
        warning: Option<&Error>,
        signer: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let header = Column::new()
            .push(warn(warning))
            .push(
                Row::new()
                    .push(
                        Column::new()
                            .push(
                                button::transparent(
                                    &mut self.back_button,
                                    Container::new(Text::new("< Go back"))
                                        .padding(10)
                                        .center_x(),
                                )
                                .on_press(Message::Previous),
                            )
                            .width(Length::Fill),
                    )
                    .push(
                        Column::new()
                            .push(
                                button::close_button(&mut self.cancel_button)
                                    .on_press(Message::Menu(Menu::Send)),
                            )
                            .align_items(Alignment::End)
                            .width(Length::Fill),
                    )
                    .width(Length::Fill)
                    .align_items(Alignment::End)
                    .padding(10)
                    .spacing(10),
            )
            .push(ProgressBar::spend_bar().draw(3))
            .align_items(Alignment::Center);

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

        Container::new(
            Column::new()
                .push(header)
                .push(
                    Container::new(
                        Column::new()
                            .push(
                                Column::new()
                                    .push(
                                        Container::new(Text::new("Sign transaction").bold())
                                            .width(Length::Fill)
                                            .center_x()
                                            .padding(20),
                                    )
                                    .push(
                                        Column::new()
                                            .push(
                                                scroll(
                                                    &mut self.scroll,
                                                    Container::new(
                                                        Column::new()
                                                            .push(
                                                                Column::new()
                                                                    .spacing(5)
                                                                    .push(
                                                                        Container::new(
                                                                            Text::new(&format!(
                                                                                "- {} {}",
                                                                                ctx.converter
                                                                                    .converts(
                                                                                    spend_amount
                                                                                ),
                                                                                ctx.converter.unit,
                                                                            ))
                                                                            .bold()
                                                                            .size(50),
                                                                        )
                                                                        .padding(10),
                                                                    )
                                                                    .push(
                                                                        Row::new()
                                                                            .push(Text::new(
                                                                                "Feerate: ",
                                                                            ))
                                                                            .push(
                                                                                Text::new(
                                                                                    &format!(
                                                                                "{} sats/vbyte",
                                                                                feerate
                                                                            ),
                                                                                )
                                                                                .bold(),
                                                                            ),
                                                                    )
                                                                    .push(Container::new(
                                                                        Text::new(&format!(
                                                                            "Miner Fee: {} {}",
                                                                            ctx.converter
                                                                                .converts(fees),
                                                                            ctx.converter.unit,
                                                                        )),
                                                                    ))
                                                                    .push(Container::new(
                                                                        Text::new(&format!(
                                                                            "Cpfp Amount: {} {}",
                                                                            ctx.converter.converts(
                                                                                tx.cpfp_amount
                                                                            ),
                                                                            ctx.converter.unit,
                                                                        )),
                                                                    ))
                                                                    .width(Length::Fill)
                                                                    .align_items(Alignment::Center),
                                                            )
                                                            .push(spend_tx_with_feerate_view(
                                                                ctx,
                                                                inputs,
                                                                tx.psbt.psbt(),
                                                                tx.change_index,
                                                                tx.cpfp_index,
                                                            ))
                                                            .spacing(20),
                                                    ),
                                                )
                                                .height(Length::Fill),
                                            )
                                            .push(
                                                card::white(Container::new(signer))
                                                    .center_x()
                                                    .width(Length::Fill),
                                            )
                                            .spacing(20),
                                    ),
                            )
                            .align_items(Alignment::Center)
                            .max_width(1000),
                    )
                    .center_x(),
                )
                .align_items(Alignment::Center)
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
}

impl ManagerSpendTransactionCreatedView {
    pub fn new() -> Self {
        ManagerSpendTransactionCreatedView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        inputs: &[model::Vault],
        tx: &model::SpendTx,
        feerate: &u64,
    ) -> Element<'a, Message> {
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
                                        .on_press(Message::Menu(Menu::Send)),
                                )
                                .width(Length::Fill)
                                .align_items(Alignment::End),
                        )
                        .align_items(Alignment::End)
                        .padding(10)
                        .spacing(10),
                )
                .push(
                    scroll(
                        &mut self.scroll,
                        Container::new(
                            Column::new()
                                .push(
                                    Column::new()
                                        .spacing(5)
                                        .push(
                                            Container::new(
                                                Text::new(&format!(
                                                    "- {} {}",
                                                    ctx.converter.converts(spend_amount),
                                                    ctx.converter.unit,
                                                ))
                                                .bold()
                                                .size(50),
                                            )
                                            .padding(10),
                                        )
                                        .push(Row::new().push(Text::new("Feerate: ")).push(
                                            Text::new(&format!("{} sats/vbyte", feerate)).bold(),
                                        ))
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
                                        .width(Length::Fill)
                                        .align_items(Alignment::Center),
                                )
                                .push(spend_tx_with_feerate_view(
                                    ctx,
                                    inputs,
                                    tx.psbt.psbt(),
                                    tx.change_index,
                                    tx.cpfp_index,
                                ))
                                .spacing(20)
                                .max_width(1000),
                        )
                        .width(Length::Fill)
                        .center_x(),
                    )
                    .align_items(Alignment::Center)
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
                                .on_press(Message::Menu(Menu::Send)),
                            )
                            .spacing(20)
                            .align_items(Alignment::Center),
                    ))
                    .padding(20)
                    .width(Length::Fill)
                    .center_x(),
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
