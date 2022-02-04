use iced::{Alignment, Column, Container, Element, Length, QRCode, Row};

use revault_ui::component::{button, card, text::Text};

use crate::app::{context::Context, error::Error, message::Message, view::layout};

/// DepositView is the view rendering the deposit panel.
/// this view is used by the Deposit State.
#[derive(Debug)]
pub struct DepositView {
    dashboard: layout::Dashboard,
    qr_code: Option<iced::qr_code::State>,
    copy_button: iced::button::State,
}

impl DepositView {
    pub fn new() -> Self {
        DepositView {
            qr_code: None,
            dashboard: layout::Dashboard::new(),
            copy_button: iced::button::State::default(),
        }
    }

    // Address is loaded directly in the view in order to cache the created qrcode.
    pub fn load(&mut self, address: &bitcoin::Address) {
        self.qr_code = iced::qr_code::State::new(address.to_string()).ok();
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        address: &bitcoin::Address,
    ) -> Element<'a, Message> {
        let mut col = Column::new()
            .align_items(Alignment::Center)
            .spacing(20)
            .push(Text::new("Please, use this deposit address:").bold());

        if let Some(qr_code) = self.qr_code.as_mut() {
            col = col.push(Container::new(QRCode::new(qr_code).cell_size(5)));
        }

        let addr = address.to_string();
        col = col.push(Container::new(
            Row::new()
                .push(Container::new(Text::new(&addr).small()))
                .push(
                    button::clipboard(&mut self.copy_button, Message::Clipboard(addr))
                        .width(Length::Shrink),
                )
                .align_items(Alignment::Center),
        ));

        self.dashboard.view(ctx, warning, card::white(col))
    }
}
