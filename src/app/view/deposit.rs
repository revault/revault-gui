use iced::{scrollable, Align, Column, Container, Element, Length, QRCode, Row};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::{layout, sidebar::Sidebar},
    },
    daemon::client::Client,
    ui::component::{button, card, navbar, scroll, text::Text},
};

/// DepositView is the view rendering the deposit panel.
/// this view is used by the Deposit State.
#[derive(Debug)]
pub struct DepositView {
    sidebar: Sidebar,
    qr_code: Option<iced::qr_code::State>,
    scroll: scrollable::State,
    copy_button: iced::button::State,
}

impl DepositView {
    pub fn new() -> Self {
        DepositView {
            qr_code: None,
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
            copy_button: iced::button::State::default(),
        }
    }

    // Address is loaded directly in the view in order to cache the created qrcode.
    pub fn load(&mut self, address: &bitcoin::Address) {
        self.qr_code = iced::qr_code::State::new(address.to_string()).ok();
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        address: Option<&bitcoin::Address>,
    ) -> Element<'a, Message> {
        let mut col = Column::new().align_items(Align::Center).spacing(20);
        if address.is_some() {
            col = col.push(Text::new("Please, use this deposit address:").bold())
        }
        if let Some(qr_code) = self.qr_code.as_mut() {
            col = col.push(Container::new(QRCode::new(qr_code).cell_size(5)));
        }
        if let Some(addr) = address {
            col = col.push(Container::new(
                Row::new()
                    .push(Container::new(Text::new(&addr.to_string()).small()))
                    .push(
                        button::clipboard(
                            &mut self.copy_button,
                            Message::Clipboard(addr.to_string()),
                        )
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            ));
        }
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                card::white(Container::new(col)),
            ))),
        )
        .into()
    }
}
