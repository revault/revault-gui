use iced::{Align, Column, Container, Element, Length, Row};

use revault_ui::{
    color,
    component::{button, card, text::Text},
    icon::warning_icon,
};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message, view::layout},
    daemon::client::Client,
};

#[derive(Debug)]
pub struct EmergencyView {
    modal: layout::Modal,
    emergency_button: iced::button::State,
}

impl EmergencyView {
    pub fn new() -> Self {
        EmergencyView {
            modal: layout::Modal::new(),
            emergency_button: iced::button::State::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        vaults_number: usize,
        funds_amount: u64,
        warning: Option<&Error>,
        processing: bool,
        success: bool,
    ) -> Element<'a, Message> {
        let mut emergency_button = button::primary(
            &mut self.emergency_button,
            button::button_content(None, "Emergency"),
        );

        if !processing {
            emergency_button = emergency_button.on_press(Message::Emergency);
        }

        let mut col = Column::new();

        if !success {
            col = col.push(
                card::border_primary(Container::new(
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
                                                ctx.converter.converts(funds_amount)
                                            ))
                                            .bold(),
                                        )
                                        .push(Text::new(&ctx.converter.unit.to_string()))
                                        .push(Text::new(" from"))
                                        .push(Text::new(&format!(" {} ", vaults_number)).bold())
                                        .push(Text::new("vaults")),
                                )
                                .push(Text::new("to the Emergency Deep Vault"))
                                .align_items(Align::Center),
                        )
                        .push(emergency_button)
                        .spacing(30)
                        .align_items(Align::Center),
                ))
                .padding(20)
                .align_x(Align::Center)
                .width(Length::Fill),
            );
        } else {
            col = col.push(
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
                                                ctx.converter.converts(funds_amount)
                                            ))
                                            .bold(),
                                        )
                                        .push(Text::new(&ctx.converter.unit.to_string()))
                                        .push(Text::new(" from"))
                                        .push(Text::new(&format!(" {} ", vaults_number)).bold())
                                        .push(Text::new("vaults")),
                                )
                                .push(Text::new("to the Emergency Deep Vault"))
                                .align_items(Align::Center),
                        )
                        .spacing(30)
                        .align_items(Align::Center),
                ))
                .padding(20)
                .align_x(Align::Center)
                .width(Length::Fill),
            );
        }

        self.modal.view(
            ctx,
            warning,
            card::white(col),
            None,
            Message::Menu(Menu::Home),
        )
    }
}
