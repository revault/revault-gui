use iced::{
    scrollable, text_input, Checkbox, Column, Container, Element, Length, Row, Scrollable,
    TextInput,
};

use crate::ui::{
    component::{button, card, separation, text},
    menu::Menu,
    message::{InputMessage, Message, RecipientMessage},
};

#[derive(Debug)]
pub enum ManagerSendView {
    SelectOutputs(ManagerSelectOutputsView),
    SelectInputs(ManagerSelectInputsView),
    SelectFee(ManagerSelectFeeView),
    Sign(ManagerSignView),
}

impl ManagerSendView {
    pub fn new() -> Self {
        Self::SelectOutputs(ManagerSelectOutputsView::new())
    }

    pub fn next(&self) -> ManagerSendView {
        match self {
            Self::SelectOutputs(_) => Self::SelectInputs(ManagerSelectInputsView::new()),
            Self::SelectInputs(_) => Self::SelectFee(ManagerSelectFeeView::new()),
            Self::SelectFee(_) => Self::Sign(ManagerSignView::new()),
            _ => Self::new(),
        }
    }

    pub fn previous(&self) -> ManagerSendView {
        match self {
            Self::SelectInputs(_) => Self::SelectOutputs(ManagerSelectOutputsView::new()),
            Self::SelectFee(_) => Self::SelectInputs(ManagerSelectInputsView::new()),
            Self::Sign(_) => Self::SelectFee(ManagerSelectFeeView::new()),
            _ => Self::new(),
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
    ) -> Element<'a, Message> {
        let mut col_outputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(iced::Align::Center);
        for (i, element) in selected_outputs.into_iter().enumerate() {
            if i > 0 {
                col_outputs = col_outputs.push(separation().width(Length::Fill));
            }
            col_outputs = col_outputs.push(element);
        }
        let element: Element<_> = col_outputs.max_width(500).into();

        let mut footer = Row::new().spacing(20).push(
            button::cancel(
                &mut self.new_output_button,
                Container::new(text::simple("Add recipient")).padding(10),
            )
            .on_press(Message::AddRecipient),
        );

        if valid {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue")).padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
            )));
        }
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
                        Container::new(element)
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
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
        address: &str,
        amount: &str,
        warning_address: &bool,
        warning_amount: &bool,
    ) -> Element<RecipientMessage> {
        let address = TextInput::new(
            &mut self.address_input,
            "Address",
            &address,
            RecipientMessage::AddressEdited,
        )
        .padding(10);
        let mut col = Column::with_children(vec![
            Container::new(
                button::transparent(
                    &mut self.delete_button,
                    Container::new(text::simple("X Remove")).padding(10),
                )
                .on_press(RecipientMessage::Delete),
            )
            .width(Length::Fill)
            .align_x(iced::Align::End)
            .into(),
            Container::new(text::bold(text::simple("Enter address:"))).into(),
            Container::new(address).into(),
        ]);

        if *warning_address {
            col = col.push(card::alert_warning(Container::new(text::simple(
                "Please enter a valid bitcoin address",
            ))))
        }
        col = col.push(text::bold(text::simple("Enter amount:"))).push(
            TextInput::new(
                &mut self.amount_input,
                "0.0",
                &format!("{}", amount),
                RecipientMessage::AmountEdited,
            )
            .padding(10),
        );

        if *warning_amount {
            col = col.push(card::alert_warning(Container::new(text::simple(
                "Please enter a valid amount",
            ))))
        }
        Container::new(col.spacing(10)).into()
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
        selected_inputs: Vec<Element<'a, Message>>,
        valid: bool,
    ) -> Element<'a, Message> {
        let mut col_inputs = Column::new()
            .spacing(20)
            .width(Length::Fill)
            .align_items(iced::Align::Center);
        for (i, element) in selected_inputs.into_iter().enumerate() {
            if i > 0 {
                col_inputs = col_inputs.push(separation().width(Length::Fill));
            }
            col_inputs = col_inputs.push(element);
        }
        let element: Element<_> = col_inputs.max_width(500).into();

        let mut footer = Column::new();
        if valid {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue")).padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
            )));
        }
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(
                                        button::transparent(
                                            &mut self.back_button,
                                            Container::new(text::simple("Go Back")).padding(10),
                                        )
                                        .on_press(Message::Previous),
                                    )
                                    .width(Length::Fill),
                            )
                            .push(
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
                        Container::new(element)
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}

pub fn manager_send_input_view<'a>(
    outpoint: &str,
    amount: &u64,
    selected: bool,
) -> Element<'a, InputMessage> {
    let checkbox =
        Checkbox::new(selected, &format!("{}", outpoint), InputMessage::Selected).text_size(15);
    let row = Row::new()
        .push(checkbox)
        .push(text::bold(text::simple(&format!(
            "{}",
            *amount as f64 / 100000000_f64
        ))))
        .spacing(20);
    Container::new(row).width(Length::Fill).into()
}

#[derive(Debug)]
pub struct ManagerSelectFeeView {
    scroll: scrollable::State,
    cancel_button: iced::button::State,
    next_button: iced::button::State,
    back_button: iced::button::State,
}

impl ManagerSelectFeeView {
    pub fn new() -> Self {
        ManagerSelectFeeView {
            cancel_button: iced::button::State::new(),
            next_button: iced::button::State::new(),
            back_button: iced::button::State::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(&'a mut self, valid: bool) -> Element<'a, Message> {
        let mut footer = Row::new().spacing(20);
        if valid {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue")).padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
            )));
        }
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(
                                        button::transparent(
                                            &mut self.back_button,
                                            Container::new(text::simple("Go Back")).padding(10),
                                        )
                                        .on_press(Message::Previous),
                                    )
                                    .width(Length::Fill),
                            )
                            .push(
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
                        Container::new(text::simple("Select fee"))
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
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

    pub fn view<'a>(&'a mut self, valid: bool) -> Element<'a, Message> {
        let mut footer = Row::new().spacing(20);
        if valid {
            footer = footer.push(Container::new(
                button::primary(
                    &mut self.next_button,
                    Container::new(text::simple("Continue")).padding(10),
                )
                .on_press(Message::Next),
            ));
        } else {
            footer = footer.push(Container::new(button::primary_disable(
                &mut self.next_button,
                Container::new(text::simple("Continue")).padding(10),
            )));
        }
        Container::new(
            Scrollable::new(&mut self.scroll).push(Container::new(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(
                                        button::transparent(
                                            &mut self.back_button,
                                            Container::new(text::simple("Go Back")).padding(10),
                                        )
                                        .on_press(Message::Previous),
                                    )
                                    .width(Length::Fill),
                            )
                            .push(
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
                        Container::new(text::simple("Sign tx"))
                            .width(Length::Fill)
                            .align_x(iced::Align::Center),
                    )
                    .push(
                        Column::new()
                            .push(footer)
                            .width(Length::Fill)
                            .align_items(iced::Align::Center),
                    )
                    .spacing(20),
            )),
        )
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
