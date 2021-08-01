use iced::{button, Align, Button, Column, Container, Element, Length, Text};

#[derive(Debug, Clone)]
pub enum ViewMessage {
    Confirm,
}

pub fn waiting_connection<'a>() -> Element<'a, ViewMessage> {
    Container::new(Text::new("waiting"))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Align::Center)
        .align_y(Align::Center)
        .into()
}

pub struct SignSpendTxView {}

impl SignSpendTxView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self) -> Element<ViewMessage> {
        Container::new(Text::new("waiting"))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Align::Center)
            .align_y(Align::Center)
            .into()
    }
}

pub struct SignUnvaultTxView {
    confirm_button: button::State,
}

impl SignUnvaultTxView {
    pub fn new() -> Self {
        Self {
            confirm_button: button::State::new(),
        }
    }

    pub fn render(&mut self) -> Element<ViewMessage> {
        Container::new(
            Column::new()
                .push(Text::new("Sign unvault tx"))
                .push(
                    Button::new(
                        &mut self.confirm_button,
                        Container::new(Text::new("Confirm"))
                            .width(Length::Units(100))
                            .align_x(Align::Center),
                    )
                    .on_press(ViewMessage::Confirm),
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

pub struct SignRevocationTxsView {}

impl SignRevocationTxsView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&mut self) -> Element<ViewMessage> {
        Container::new(Text::new("sign revocation"))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Align::Center)
            .align_y(Align::Center)
            .into()
    }
}
