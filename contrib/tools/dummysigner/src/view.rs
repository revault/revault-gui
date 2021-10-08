use iced::{
    button, container, Align, Button, Color, Column, Container, Element, Length, Row, Text,
};
use std::net::SocketAddr;

use crate::api;

#[derive(Debug, Clone)]
pub enum ViewMessage {
    Confirm,
    Cancel,
}

pub fn waiting_connection<'a>() -> Element<'a, ViewMessage> {
    Container::new(Text::new("waiting"))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
}

pub fn connected<'a>(addr: &SocketAddr) -> Element<'a, ViewMessage> {
    Container::new(Text::new(&format!("Connected to {}", addr)))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
}

pub struct SignSpendTxView {
    cancel_button: button::State,
    confirm_button: button::State,
}

impl SignSpendTxView {
    pub fn new() -> Self {
        Self {
            cancel_button: button::State::new(),
            confirm_button: button::State::new(),
        }
    }

    pub fn render(&mut self, _req: &api::SpendTransaction, signed: bool) -> Element<ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed spend transaction"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Align::Center)
                .align_y(Align::Center)
                .into();
        }
        Container::new(
            Column::new()
                .push(Text::new("Sign spend transaction"))
                .push(
                    Row::new()
                        .push(
                            Button::new(
                                &mut self.cancel_button,
                                Container::new(Text::new("Cancel"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Cancel),
                        )
                        .push(
                            Button::new(
                                &mut self.confirm_button,
                                Container::new(Text::new("Confirm"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Confirm),
                        )
                        .spacing(20),
                )
                .spacing(20)
                .align_items(Align::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
    }
}

pub struct SignUnvaultTxView {
    cancel_button: button::State,
    confirm_button: button::State,
}

impl SignUnvaultTxView {
    pub fn new() -> Self {
        Self {
            cancel_button: button::State::new(),
            confirm_button: button::State::new(),
        }
    }

    pub fn render(&mut self, _req: &api::UnvaultTransaction, signed: bool) -> Element<ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed unvault transaction"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Align::Center)
                .align_y(Align::Center)
                .into();
        }
        Container::new(
            Column::new()
                .push(Text::new("Sign unvault tx"))
                .push(
                    Row::new()
                        .push(
                            Button::new(
                                &mut self.cancel_button,
                                Container::new(Text::new("Cancel"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Cancel),
                        )
                        .push(
                            Button::new(
                                &mut self.confirm_button,
                                Container::new(Text::new("Confirm"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Confirm),
                        )
                        .spacing(20),
                )
                .spacing(20)
                .align_items(Align::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
    }
}

pub struct SignRevocationTxsView {
    cancel_button: button::State,
    confirm_button: button::State,
}

impl SignRevocationTxsView {
    pub fn new() -> Self {
        Self {
            cancel_button: button::State::new(),
            confirm_button: button::State::new(),
        }
    }

    pub fn render(
        &mut self,
        _req: &api::RevocationTransactions,
        signed: bool,
    ) -> Element<ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed revocation transactions"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Align::Center)
                .align_y(Align::Center)
                .into();
        }
        Container::new(
            Column::new()
                .push(Text::new("Sign revocation transactions"))
                .push(
                    Row::new()
                        .push(
                            Button::new(
                                &mut self.cancel_button,
                                Container::new(Text::new("Cancel"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Cancel),
                        )
                        .push(
                            Button::new(
                                &mut self.confirm_button,
                                Container::new(Text::new("Confirm"))
                                    .width(Length::Units(100))
                                    .align_x(Align::Center),
                            )
                            .on_press(ViewMessage::Confirm),
                        )
                        .spacing(20),
                )
                .spacing(20)
                .align_items(Align::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
    }
}

pub struct SuccessPageStyle;
impl container::StyleSheet for SuccessPageStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: SUCCESS.into(),
            text_color: Color::WHITE.into(),
            ..container::Style::default()
        }
    }
}

pub const SUCCESS: Color = Color::from_rgb(
    0x29 as f32 / 255.0,
    0xBC as f32 / 255.0,
    0x97 as f32 / 255.0,
);
