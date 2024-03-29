use bitcoin::Amount;
use iced::{Alignment, Column, Container, Element, Length, Row};

use revault_ui::{
    color,
    component::{button, card, text::Text},
    icon::warning_icon,
};

use crate::app::{context::Context, error::Error, menu::Menu, message::Message, view::layout};

#[derive(Debug)]
pub struct EmergencyView {
    modal: layout::Modal,
    emergency_button: iced::button::State,
}

impl EmergencyView {
    pub fn new() -> Self {
        EmergencyView {
            modal: layout::Modal::default(),
            emergency_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vaults_number: usize,
        funds_amount: u64,
        warning: Option<&Error>,
        processing: bool,
    ) -> Element<'a, Message> {
        let mut emergency_button = button::primary(
            &mut self.emergency_button,
            button::button_content(None, "Emergency"),
        );

        if !processing {
            emergency_button = emergency_button.on_press(Message::Emergency);
        }

        let content = if funds_amount != 0 {
            Column::new()
                .push(warning_icon().color(color::PRIMARY))
                .push(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Text::new("This action will send"))
                                .push(
                                    Text::new(&format!(
                                        " {} ",
                                        ctx.converter.converts(Amount::from_sat(funds_amount))
                                    ))
                                    .bold(),
                                )
                                .push(Text::new(&ctx.converter.unit.to_string()))
                                .push(Text::new(" from"))
                                .push(Text::new(&format!(" {} ", vaults_number)).bold())
                                .push(Text::new("vaults")),
                        )
                        .push(Text::new("to the Emergency Deep Vault"))
                        .align_items(Alignment::Center),
                )
                .push(emergency_button)
                .spacing(30)
                .align_items(Alignment::Center)
        } else {
            Column::new()
                .push(warning_icon().color(color::PRIMARY))
                .push(Text::new("No funds to send to the Emergency Deep Vault"))
                .spacing(30)
                .align_items(Alignment::Center)
        };

        self.modal.view(
            ctx,
            warning,
            card::border_primary(Container::new(content))
                .padding(20)
                .center_x()
                .width(Length::Fill),
            None,
            Message::Menu(Menu::Home),
        )
    }
}

#[derive(Debug)]
pub struct EmergencyTriggeredView {
    modal: layout::Modal,
}

impl EmergencyTriggeredView {
    pub fn new() -> Self {
        EmergencyTriggeredView {
            modal: layout::Modal::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vaults_number: usize,
        funds_amount: u64,
    ) -> Element<'a, Message> {
        self.modal.view(
            ctx,
            None,
            card::border_success(Container::new(
                Column::new()
                    .push(warning_icon().color(color::SUCCESS))
                    .push(
                        Column::new()
                            .push(
                                Row::new()
                                    .push(Text::new("Sending"))
                                    .push(
                                        Text::new(&format!(
                                            " {} ",
                                            ctx.converter.converts(Amount::from_sat(funds_amount))
                                        ))
                                        .bold(),
                                    )
                                    .push(Text::new(&ctx.converter.unit.to_string()))
                                    .push(Text::new(" from"))
                                    .push(Text::new(&format!(" {} ", vaults_number)).bold())
                                    .push(Text::new("vaults")),
                            )
                            .push(Text::new("to the Emergency Deep Vault"))
                            .align_items(Alignment::Center),
                    )
                    .spacing(30)
                    .align_items(Alignment::Center),
            ))
            .padding(20)
            .center_x()
            .width(Length::Fill),
            None,
            Message::Menu(Menu::Home),
        )
    }
}
