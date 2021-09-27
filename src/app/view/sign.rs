use iced::{Column, Container, Element, Length};

use crate::{
    app::{context::Context, message::SignMessage},
    ui::{
        component::{button, card, text::Text},
        icon,
    },
};

#[derive(Debug)]
pub struct SignerView {
    sign_button: iced::button::State,
}

impl SignerView {
    pub fn new() -> Self {
        SignerView {
            sign_button: iced::button::State::default(),
        }
    }

    pub fn view(
        &mut self,
        _ctx: &Context,
        connected: bool,
        processing: bool,
        signed: bool,
    ) -> Element<SignMessage> {
        if signed {
            return card::success(Container::new(
                Column::new()
                    .align_items(iced::Align::Center)
                    .spacing(20)
                    .push(Text::from(icon::done_icon()).size(20).success())
                    .push(Text::new("Signed").success()),
            ))
            .padding(50)
            .width(Length::Fill)
            .align_x(iced::Align::Center)
            .into();
        }
        if connected {
            let mut sign_button = button::primary(
                &mut self.sign_button,
                button::button_content(None, " Sign ").width(Length::Units(200)),
            );
            if !processing {
                sign_button = sign_button.on_press(SignMessage::SelectSign);
            }
            card::white(Container::new(
                Column::new()
                    .align_items(iced::Align::Center)
                    .spacing(20)
                    .push(icon::connected_device_icon().size(20))
                    .push(sign_button),
            ))
            .padding(50)
            .width(Length::Fill)
            .align_x(iced::Align::Center)
            .into()
        } else {
            card::white(Container::new(
                Column::new()
                    .align_items(iced::Align::Center)
                    .spacing(20)
                    .push(icon::connect_device_icon().size(20))
                    .push(Text::new("Connect hardware wallet")),
            ))
            .padding(50)
            .width(Length::Fill)
            .align_x(iced::Align::Center)
            .into()
        }
    }
}
