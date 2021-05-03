use iced::{scrollable, Align, Column, Container, Element, Length, Row};

use crate::{
    app::{error::Error, menu::Menu, message::Message, view::Context},
    ui::{
        color,
        component::{button, card, scroll, text, ContainerBackgroundStyle},
        icon::warning_icon,
    },
};

#[derive(Debug)]
pub struct EmergencyView {
    scroll: scrollable::State,
    close_button: iced::button::State,
    emergency_button: iced::button::State,
}

impl EmergencyView {
    pub fn new() -> Self {
        EmergencyView {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
            emergency_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        vaults_number: usize,
        funds_amount: u64,
        warning: Option<&Error>,
        loading: bool,
        processing: bool,
        success: bool,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .push(
                Row::new().push(Column::new().width(Length::Fill)).push(
                    Container::new(
                        button::cancel(
                            &mut self.close_button,
                            Container::new(text::simple("X Close")).padding(10),
                        )
                        .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Shrink),
                ),
            )
            .spacing(50);

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::simple(&format!(
                "{}",
                error
            )))))
        }

        if !loading {
            let mut emergency_button = button::primary(
                &mut self.emergency_button,
                button::button_content(None, "Emergency"),
            );

            if !processing {
                emergency_button = emergency_button.on_press(Message::Emergency);
            }

            if !success {
                col = col.push(
                    card::border_primary(Container::new(
                        Column::new()
                            .push(warning_icon().color(color::PRIMARY))
                            .push(
                                Column::new()
                                    .push(
                                        Row::new()
                                            .push(text::simple("This action will send"))
                                            .push(text::bold(text::simple(&format!(
                                                " {} ",
                                                ctx.converter.converts(funds_amount)
                                            ))))
                                            .push(text::simple(&ctx.converter.unit.to_string()))
                                            .push(text::simple(" from"))
                                            .push(text::bold(text::simple(&format!(
                                                " {} ",
                                                vaults_number
                                            ))))
                                            .push(text::simple("vaults")),
                                    )
                                    .push(text::simple("to the Emergency Deep Vault"))
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
                                            .push(text::simple("Sending"))
                                            .push(text::bold(text::simple(&format!(
                                                " {} ",
                                                ctx.converter.converts(funds_amount)
                                            ))))
                                            .push(text::simple(&ctx.converter.unit.to_string()))
                                            .push(text::simple(" from"))
                                            .push(text::bold(text::simple(&format!(
                                                " {} ",
                                                vaults_number
                                            ))))
                                            .push(text::simple("vaults")),
                                    )
                                    .push(text::simple("to the Emergency Deep Vault"))
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
        }

        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}
