use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use iced::{Align, Column, Container, Element, Length, Row};

use crate::{
    revault::TransactionKind,
    ui::{
        component::{badge, button, card, separation, text, ContainerBackgroundStyle},
        message::SignMessage,
        view::Context,
    },
};

#[derive(Debug)]
pub struct DirectSignatureView {
    indirect_button: iced::button::State,
    sign_button: iced::button::State,
}

impl DirectSignatureView {
    pub fn new() -> Self {
        DirectSignatureView {
            indirect_button: iced::button::State::default(),
            sign_button: iced::button::State::default(),
        }
    }

    pub fn view(
        &mut self,
        _ctx: &Context,
        transaction_kind: &TransactionKind,
    ) -> Element<SignMessage> {
        let title = match transaction_kind {
            TransactionKind::Emergency => {
                text::bold(text::simple("Sign emergency transaction").size(25))
            }
            TransactionKind::EmergencyUnvault => {
                text::bold(text::simple("Sign emergency unvault transaction").size(25))
            }
            TransactionKind::Cancel => text::bold(text::simple("Sign cancel transaction").size(25)),
            TransactionKind::Spend => text::bold(text::simple("Sign spend transaction").size(25)),
            TransactionKind::Unvault => {
                text::bold(text::simple("Sign unvault transaction").size(25))
            }
        };

        let col = Column::new()
            .push(
                Row::new()
                    .push(Container::new(title).width(Length::Fill))
                    .push(
                        button::transparent(
                            &mut self.indirect_button,
                            button::button_content(None, "Use Psbt"),
                            SignMessage::ChangeMethod,
                        )
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            )
            .push(separation().width(Length::Fill))
            .push(
                Container::new(text::simple("Connect device"))
                    .padding(20)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .push(
                Container::new(button::primary(
                    &mut self.sign_button,
                    button::button_content(None, " Sign transaction "),
                    SignMessage::ChangeMethod,
                ))
                .width(Length::Fill)
                .align_x(Align::Center),
            )
            .spacing(10);
        Container::new(col).into()
    }
}

#[derive(Debug)]
pub struct IndirectSignatureView {
    direct_button: iced::button::State,
    sign_button: iced::button::State,
}

impl IndirectSignatureView {
    pub fn new() -> Self {
        IndirectSignatureView {
            direct_button: iced::button::State::default(),
            sign_button: iced::button::State::default(),
        }
    }

    pub fn view(
        &mut self,
        _ctx: &Context,
        transaction_kind: &TransactionKind,
        psbt: &Psbt,
    ) -> Element<SignMessage> {
        let title = match transaction_kind {
            TransactionKind::Emergency => {
                text::bold(text::simple("Sign emergency transaction").size(25))
            }
            TransactionKind::EmergencyUnvault => {
                text::bold(text::simple("Sign emergency unvault transaction").size(25))
            }
            TransactionKind::Cancel => text::bold(text::simple("Sign cancel transaction").size(25)),
            TransactionKind::Spend => text::bold(text::simple("Sign spend transaction").size(25)),
            TransactionKind::Unvault => {
                text::bold(text::simple("Sign unvault transaction").size(25))
            }
        };

        let col = Column::new()
            .push(
                Row::new()
                    .push(Container::new(title).width(Length::Fill))
                    .push(
                        button::transparent(
                            &mut self.direct_button,
                            button::button_content(None, "Use hardware module"),
                            SignMessage::ChangeMethod,
                        )
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            )
            .push(separation().width(Length::Fill))
            .push(Container::new(text::simple(&format!("{:?}", *psbt))).width(Length::Fill))
            .push(
                Container::new(button::primary(
                    &mut self.sign_button,
                    button::button_content(None, " Sign transaction "),
                    SignMessage::Sign,
                ))
                .width(Length::Fill)
                .align_x(Align::Center),
            )
            .spacing(10);
        Container::new(col).into()
    }
}
