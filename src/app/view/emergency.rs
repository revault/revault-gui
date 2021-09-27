use iced::{scrollable, Align, Column, Container, Element, Length, Row};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message},
    ui::{
        color,
        component::{button, card, scroll, text::Text, ContainerBackgroundStyle},
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
                        button::close_button(&mut self.close_button)
                            .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Shrink),
                ),
            )
            .spacing(50);

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(Text::new(&format!(
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
        }

        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}
