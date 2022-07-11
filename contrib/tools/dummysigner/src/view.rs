use iced::{
    alignment, button, container, Button, Checkbox, Color, Column, Container, Element, Length, Row,
    Text,
};
use std::net::SocketAddr;

use revault_tx::bitcoin::Amount;

use crate::api;

#[derive(Debug, Clone)]
pub enum ViewMessage {
    Confirm,
    Cancel,
    Key(usize, KeyMessage),
}

#[derive(Debug, Clone)]
pub enum KeyMessage {
    Selected(bool),
}

pub fn waiting_connection<'a>() -> Element<'a, ViewMessage> {
    Container::new(Text::new("waiting"))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
}

pub fn connected<'a>(addr: &SocketAddr) -> Element<'a, ViewMessage> {
    Container::new(Text::new(&format!("Connected to {}", addr)))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
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

    pub fn render<'a>(
        &'a mut self,
        _req: &api::SpendTransaction,
        signed: bool,
        keys: Vec<Element<'a, ViewMessage>>,
        can_confirm: bool,
    ) -> Element<ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed spend transaction"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into();
        }

        if keys.is_empty() {
            return error_no_keys(&mut self.cancel_button);
        }

        Container::new(
            Column::new()
                .push(Text::new("Send funds"))
                .push(Text::new("Select keys to sign the transaction with"))
                .push(Column::with_children(keys).spacing(10))
                .push(confirmation_footer(
                    &mut self.cancel_button,
                    &mut self.confirm_button,
                    can_confirm,
                ))
                .spacing(20)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
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

    pub fn render<'a>(
        &'a mut self,
        _req: &api::UnvaultTransaction,
        signed: bool,
        keys: Vec<Element<'a, ViewMessage>>,
        can_confirm: bool,
    ) -> Element<'a, ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed unvault transaction"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into();
        }

        if keys.is_empty() {
            return error_no_keys(&mut self.cancel_button);
        }

        Container::new(
            Column::new()
                .push(Text::new("Delegate vault"))
                .push(Text::new("Select keys to sign unvault transaction with"))
                .push(Column::with_children(keys).spacing(10))
                .push(confirmation_footer(
                    &mut self.cancel_button,
                    &mut self.confirm_button,
                    can_confirm,
                ))
                .spacing(20)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
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

    pub fn render<'a>(
        &'a mut self,
        _req: &api::RevocationTransactions,
        signed: bool,
        keys: Vec<Element<'a, ViewMessage>>,
        can_confirm: bool,
    ) -> Element<'a, ViewMessage> {
        if signed {
            return Container::new(Text::new("Signed revocation transactions"))
                .style(SuccessPageStyle)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .into();
        }

        if keys.is_empty() {
            return error_no_keys(&mut self.cancel_button);
        }

        Container::new(
            Column::new()
                .push(Text::new("Secure deposit"))
                .push(Text::new(
                    "Select keys to sign revocation transactions with",
                ))
                .push(Column::with_children(keys).spacing(10))
                .push(confirmation_footer(
                    &mut self.cancel_button,
                    &mut self.confirm_button,
                    can_confirm,
                ))
                .spacing(20)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    }
}

pub struct SecureBatchView {
    cancel_button: button::State,
    confirm_button: button::State,
}

impl SecureBatchView {
    pub fn new() -> Self {
        Self {
            cancel_button: button::State::new(),
            confirm_button: button::State::new(),
        }
    }

    pub fn render<'a>(
        &'a mut self,
        total_amount: u64,
        total_deposits: usize,
        signed: bool,
        keys: Vec<Element<'a, ViewMessage>>,
        can_confirm: bool,
    ) -> Element<ViewMessage> {
        if signed {
            return Container::new(Column::new().push(Text::new(format!(
                "Revocation transactions signed \n for {} deposits ({} BTC)",
                total_deposits,
                Amount::from_sat(total_amount).as_btc(),
            ))))
            .style(SuccessPageStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into();
        }

        if keys.is_empty() {
            return error_no_keys(&mut self.cancel_button);
        }

        Container::new(
            Column::new()
                .push(Text::new(format!(
                    "Vault {} deposits for a total of {} BTC",
                    total_deposits,
                    Amount::from_sat(total_amount).as_btc()
                )))
                .push(Text::new(
                    "Select keys to sign the revocation transactions with:",
                ))
                .push(Column::with_children(keys).spacing(10))
                .push(confirmation_footer(
                    &mut self.cancel_button,
                    &mut self.confirm_button,
                    can_confirm,
                ))
                .spacing(20)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    }
}

pub struct DelegateBatchView {
    cancel_button: button::State,
    confirm_button: button::State,
}

impl DelegateBatchView {
    pub fn new() -> Self {
        Self {
            cancel_button: button::State::new(),
            confirm_button: button::State::new(),
        }
    }

    pub fn render<'a>(
        &'a mut self,
        total_amount: u64,
        total_vaults: usize,
        signed: bool,
        keys: Vec<Element<'a, ViewMessage>>,
        can_confirm: bool,
    ) -> Element<ViewMessage> {
        if signed {
            return Container::new(Column::new().push(Text::new(format!(
                "Unvault transactions signed \n for {} deposits ({} BTC)",
                total_vaults,
                Amount::from_sat(total_amount).as_btc(),
            ))))
            .style(SuccessPageStyle)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .into();
        }

        if keys.is_empty() {
            return error_no_keys(&mut self.cancel_button);
        }

        Container::new(
            Column::new()
                .push(Text::new(format!(
                    "Delegate {} vaults for a total of {} BTC",
                    total_vaults,
                    Amount::from_sat(total_amount).as_btc(),
                )))
                .push(Text::new("Select keys to sign unvault transactions with"))
                .push(Column::with_children(keys).spacing(10))
                .push(confirmation_footer(
                    &mut self.cancel_button,
                    &mut self.confirm_button,
                    can_confirm,
                ))
                .spacing(20)
                .align_items(alignment::Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .into()
    }
}

pub fn key_view(name: &str, selected: bool) -> Element<'static, KeyMessage> {
    Container::new(Checkbox::new(selected, name, KeyMessage::Selected)).into()
}

pub fn confirmation_footer<'a>(
    cancel_button: &'a mut button::State,
    confirm_button: &'a mut button::State,
    can_confirm: bool,
) -> Element<'a, ViewMessage> {
    let mut confirm_button = Button::new(
        confirm_button,
        Container::new(Text::new("Sign"))
            .width(Length::Units(100))
            .align_x(alignment::Horizontal::Center),
    );

    if can_confirm {
        confirm_button = confirm_button.on_press(ViewMessage::Confirm);
    }

    Row::new()
        .push(
            Button::new(
                cancel_button,
                Container::new(Text::new("Cancel"))
                    .width(Length::Units(100))
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(ViewMessage::Cancel),
        )
        .push(confirm_button)
        .spacing(20)
        .into()
}

pub fn error_no_keys<'a>(cancel_button: &'a mut button::State) -> Element<'a, ViewMessage> {
    return Container::new(
        Column::new()
            .align_items(alignment::Alignment::Center)
            .spacing(20)
            .push(Text::new(
                "No keys matched the specified key fingerprints in the psbt",
            ))
            .push(
                Button::new(
                    cancel_button,
                    Container::new(Text::new("Cancel"))
                        .width(Length::Units(100))
                        .align_x(alignment::Horizontal::Center),
                )
                .on_press(ViewMessage::Cancel),
            ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into();
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
