use iced::{
    scrollable,
    tooltip::{self, Tooltip},
    Align, Column, Container, Element, Length, QRCode, Row,
};

use crate::{
    app::{context::Context, error::Error, menu::Menu, message::Message},
    ui::{
        component::{
            button, card, scroll, separation, text, ContainerBackgroundStyle, TooltipStyle,
        },
        icon,
    },
};

#[derive(Debug)]
pub struct StakeholderCreateVaultsView {
    scroll: scrollable::State,
    qr_code: Option<iced::qr_code::State>,
    close_button: iced::button::State,
    copy_button: iced::button::State,
}

impl StakeholderCreateVaultsView {
    pub fn new() -> Self {
        StakeholderCreateVaultsView {
            qr_code: None,
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
            copy_button: iced::button::State::new(),
        }
    }

    // Address is loaded directly in the view in order to cache the created qrcode.
    pub fn load(&mut self, address: &bitcoin::Address) {
        self.qr_code = iced::qr_code::State::new(address.to_string()).ok();
    }

    pub fn view<'a>(
        &'a mut self,
        _ctx: &Context,
        deposits: Vec<Element<'a, Message>>,
        address: Option<&bitcoin::Address>,
    ) -> Element<'a, Message> {
        let mut content = Column::new()
            .max_width(800)
            .push(text::bold(text::simple("Create some vaults")).size(50))
            .spacing(20);

        if !deposits.is_empty() {
            content = content.push(Container::new(
                Column::new()
                    .push(text::simple(" Click on a deposit to create a vault:"))
                    .push(Column::with_children(deposits).spacing(5))
                    .spacing(20),
            ))
        } else {
            content = content.push(Container::new(text::simple("No deposits")).padding(5))
        }

        if let Some(qr_code) = self.qr_code.as_mut() {
            if let Some(addr) = address {
                content = content.push(separation().width(Length::Fill)).push(
                    card::white(Container::new(
                        Row::new()
                            .push(
                                Column::new()
                                    .push(text::simple(
                                        "Bitcoin deposits are needed in order to create a vault\n",
                                    ))
                                    .push(
                                        Column::new()
                                            .push(text::bold(text::simple(
                                                "Please, use this deposit address:",
                                            )))
                                            .push(
                                                Row::new()
                                                    .push(Container::new(text::bold(text::small(
                                                        &addr.to_string(),
                                                    ))))
                                                    .push(
                                                        button::clipboard(
                                                            &mut self.copy_button,
                                                            Message::Clipboard(addr.to_string()),
                                                        )
                                                        .width(Length::Shrink),
                                                    )
                                                    .align_items(Align::Center),
                                            ),
                                    )
                                    .spacing(30)
                                    .width(Length::Fill),
                            )
                            .push(
                                Container::new(QRCode::new(qr_code).cell_size(5))
                                    .width(Length::Shrink),
                            )
                            .spacing(10),
                    ))
                    .width(Length::Fill),
                );
            }
        }

        let col = Column::new()
            .push(
                Row::new().push(Container::new(
                            Tooltip::new(
                                Row::new()
                                    .push(icon::tooltip_icon().size(15))
                                    .push(text::small(" Help")),
                                "A vault is a deposit with revocation transactions\nsigned and shared between stakeholders",
                                tooltip::Position::Right,
                            )
                            .gap(5)
                            .size(15)
                            .padding(10)
                            .style(TooltipStyle),
                        )
                        .width(Length::Fill)).push(
                    Container::new(
                        button::close_button(
                            &mut self.close_button,
                        )
                        .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Shrink),
                ),
            ).push(content).align_items(Align::Center).spacing(50);

        Container::new(scroll(&mut self.scroll, Container::new(col)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}

#[derive(Debug)]
pub struct StakeholderDelegateFundsView {
    scroll: scrollable::State,
    close_button: iced::button::State,
}

impl StakeholderDelegateFundsView {
    pub fn new() -> Self {
        StakeholderDelegateFundsView {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        active_balance: &u64,
        activating_balance: &u64,
        vaults: Vec<Element<'a, Message>>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        let mut col = Column::new();
        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(text::simple(&format!(
                "{}",
                error
            )))))
        }

        col = col
            .push(
                Column::new()
                    .push(text::bold(text::simple("Delegate funds to your manager team")).size(50))
                    .spacing(5),
            )
            .push(
                Column::new()
                    .push(
                        Row::new()
                            .push(
                                text::bold(text::simple(
                                    &ctx.converter.converts(*active_balance).to_string(),
                                ))
                                .size(30),
                            )
                            .push(text::simple(&format!(
                                " {} are allocated to managers",
                                ctx.converter.unit
                            )))
                            .align_items(Align::Center),
                    )
                    .push(
                        Row::new()
                            .push(
                                text::bold(text::simple(&format!(
                                    "+ {}",
                                    ctx.converter.converts(*activating_balance)
                                )))
                                .size(20),
                            )
                            .push(text::simple(&format!(
                                " {} are waiting for other stakeholders' approval",
                                ctx.converter.unit
                            )))
                            .align_items(Align::Center),
                    ),
            );
        if !vaults.is_empty() {
            col = col.push(Container::new(
                Column::new()
                    .push(text::simple(" Click on the vaults to delegate:"))
                    .push(Column::with_children(vaults).spacing(5))
                    .spacing(20),
            ))
        } else {
            col = col.push(
                Container::new(text::simple("No more funds can be delegated to managers"))
                    .padding(5),
            )
        }

        let modal = Column::new()
            .push(
                Row::new()
                    .push(
                        Container::new(
                            Tooltip::new(
                                Row::new()
                                    .push(icon::tooltip_icon().size(15))
                                    .push(text::small(" Help")),
                                "By delegating you allow managers to spend the funds,\n but you can still revert any undesired transaction.",
                                tooltip::Position::Right,
                            )
                            .gap(5)
                            .size(15)
                            .padding(10)
                            .style(TooltipStyle),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            button::close_button(
                                &mut self.close_button,
                            )
                            .on_press(Message::Menu(Menu::Home)),
                        )
                        .width(Length::Shrink),
                    )
                    .align_items(Align::Center),
            )
            .push(
                Container::new(col.spacing(30).max_width(800))
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(50);

        Container::new(scroll(&mut self.scroll, Container::new(modal)))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}
