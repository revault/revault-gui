use iced::{Column, Container, Element, Length};

use crate::{
    app::{message::SignMessage, view::Context},
    ui::component::{button, card, text},
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
            return card::success(
                Container::new(text::success(text::simple("Signed")))
                    .width(Length::Fill)
                    .align_x(iced::Align::Center)
                    .padding(50),
            )
            .into();
        }
        if connected {
            let mut sign_button =
                button::primary(&mut self.sign_button, button::button_content(None, " Sign"));
            if !processing {
                sign_button = sign_button.on_press(SignMessage::SelectSign);
            }
            Container::new(Column::new().push(sign_button).spacing(10)).into()
        } else {
            card::white(Container::new(
                Column::new()
                    .push(text::simple("waiting connection"))
                    .spacing(10),
            ))
            .into()
        }
    }
}
