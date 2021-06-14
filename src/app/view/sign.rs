use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use iced::{Align, Column, Container, Element, Length, Row, TextInput};

use crate::{
    app::{
        message::{SignMessage, SignatureSharingStatus},
        view::Context,
    },
    revault::TransactionKind,
    ui::component::{button, card, separation, text},
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
            TransactionKind::Emergency => text::bold(text::simple("Sign emergency transaction")),
            TransactionKind::EmergencyUnvault => {
                text::bold(text::simple("Sign emergency unvault transaction"))
            }
            TransactionKind::Cancel => text::bold(text::simple("Sign cancel transaction")),
            TransactionKind::Spend => text::bold(text::simple("Sign spend transaction")),
            TransactionKind::Unvault => text::bold(text::simple("Sign unvault transaction")),
        };

        let col = Column::new()
            .push(
                Row::new()
                    .push(Container::new(title).width(Length::Fill))
                    .push(
                        button::transparent(
                            &mut self.indirect_button,
                            button::button_content(None, "Use PSBT"),
                        )
                        .on_press(SignMessage::ChangeMethod)
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
                Container::new(
                    button::primary(
                        &mut self.sign_button,
                        button::button_content(None, " Sign transaction "),
                    )
                    .on_press(SignMessage::ChangeMethod),
                )
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
    copy_button: iced::button::State,
    psbt_input: iced::text_input::State,
}

impl IndirectSignatureView {
    pub fn new() -> Self {
        IndirectSignatureView {
            direct_button: iced::button::State::default(),
            sign_button: iced::button::State::default(),
            copy_button: iced::button::State::default(),
            psbt_input: iced::text_input::State::new(),
        }
    }

    pub fn view(
        &mut self,
        _ctx: &Context,
        sharing_status: &SignatureSharingStatus,
        transaction_kind: &TransactionKind,
        psbt: &Psbt,
        psbt_input: &str,
        warning: Option<&String>,
    ) -> Element<SignMessage> {
        let title = match transaction_kind {
            TransactionKind::Emergency => text::bold(text::simple("Sign emergency transaction")),
            TransactionKind::EmergencyUnvault => {
                text::bold(text::simple("Sign emergency unvault transaction"))
            }
            TransactionKind::Cancel => text::bold(text::simple("Sign cancel transaction")),
            TransactionKind::Spend => text::bold(text::simple("Sign spend transaction")),
            TransactionKind::Unvault => text::bold(text::simple("Sign unvault transaction")),
        };

        let psbt_str = bitcoin::base64::encode(&bitcoin::consensus::serialize(psbt));

        let mut col = Column::new()
            .push(
                Row::new()
                    .push(Container::new(title).width(Length::Fill))
                    .push(
                        button::transparent(
                            &mut self.direct_button,
                            button::button_content(None, "Use hardware module"),
                        )
                        .on_press(SignMessage::ChangeMethod)
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            )
            .push(separation().width(Length::Fill))
            .push(
                Container::new(
                    Row::new()
                        .push(Container::new(text::small(&psbt_str)).width(Length::Fill))
                        .push(
                            button::clipboard(
                                &mut self.copy_button,
                                SignMessage::Clipboard(psbt_str),
                            )
                            .width(Length::Shrink),
                        )
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            );
        if let Some(message) = warning {
            col = col.push(card::alert_warning(Container::new(text::simple(message))));
        }

        match sharing_status {
            SignatureSharingStatus::Success => {
                col = col
                    .push(Container::new(text::small(&psbt_input.to_string())))
                    .push(
                        Container::new(card::success(Container::new(text::simple("success"))))
                            .width(Length::Fill)
                            .align_x(Align::Center),
                    );
            }
            SignatureSharingStatus::Processing => {
                col = col
                    .push(Container::new(text::small(&psbt_input.to_string())))
                    .push(Container::new(
                        button::primary_disable(
                            &mut self.sign_button,
                            button::button_content(None, " Processing "),
                        )
                        .on_press(SignMessage::Sign),
                    ));
            }
            SignatureSharingStatus::Unshared => {
                col = col
                    .push(
                        TextInput::new(
                            &mut self.psbt_input,
                            "Signed PSBT",
                            &psbt_input,
                            SignMessage::PsbtEdited,
                        )
                        .size(15)
                        .width(Length::Fill)
                        .padding(10),
                    )
                    .push(
                        Container::new(
                            button::primary(
                                &mut self.sign_button,
                                button::button_content(None, " Sign transaction "),
                            )
                            .on_press(SignMessage::Sign),
                        )
                        .width(Length::Fill)
                        .align_x(Align::Center),
                    );
            }
        };

        Container::new(col.spacing(10)).into()
    }
}
