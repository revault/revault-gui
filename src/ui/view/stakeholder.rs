use iced::{scrollable, Align, Column, Container, Element, Length, Row, Scrollable};

use bitcoin::util::psbt::PartiallySignedTransaction;

use crate::revaultd::model::Vault;

use crate::ui::{
    component::{badge, button, card, separation, text, ContainerBackgroundStyle},
    icon,
    menu::Menu,
    message::{DepositMessage, Message},
    view::Context,
};

#[derive(Debug)]
pub struct StakeholderACKFundsView {
    scroll: scrollable::State,
    close_button: iced::button::State,
}

impl StakeholderACKFundsView {
    pub fn new() -> Self {
        StakeholderACKFundsView {
            scroll: scrollable::State::new(),
            close_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        _ctx: &Context,
        deposits: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut col_deposits = Column::new();
        for element in deposits.into_iter() {
            col_deposits = col_deposits.push(element);
        }
        let element: Element<_> = col_deposits.spacing(20).max_width(1000).into();
        let col = Column::new()
            .push(
                Row::new().push(Column::new().width(Length::Fill)).push(
                    Container::new(button::cancel(
                        &mut self.close_button,
                        Container::new(text::simple("X Close")).padding(10),
                        Message::Menu(Menu::Home),
                    ))
                    .width(Length::Shrink),
                ),
            )
            .push(
                Container::new(element)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(50);
        Container::new(Scrollable::new(&mut self.scroll).push(col))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(ContainerBackgroundStyle)
            .padding(20)
            .into()
    }
}

pub fn stakeholder_deposit_signed<'a>(
    _ctx: &Context,
    deposit: &Vault,
) -> Element<'a, DepositMessage> {
    card::white(Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(badge::shield_success())
                        .push(
                            Container::new(text::success(text::bold(text::small(
                                &deposit.address,
                            ))))
                            .align_y(Align::Center),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(text::success(text::bold(text::simple(&format!(
                            "{}",
                            deposit.amount as f64 / 100000000_f64
                        )))))
                        .push(text::small(" BTC"))
                        .align_items(Align::Center),
                )
                .width(Length::Shrink),
            )
            .spacing(20)
            .align_items(Align::Center),
    ))
    .into()
}

pub fn stakeholder_deposit_pending<'a>(
    _ctx: &Context,
    deposit: &Vault,
) -> Element<'a, DepositMessage> {
    card::white(Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(badge::shield_notif())
                        .push(
                            Container::new(text::bold(text::small(&deposit.address)))
                                .align_y(Align::Center),
                        )
                        .spacing(20)
                        .align_items(Align::Center),
                )
                .width(Length::Fill),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(text::bold(text::simple(&format!(
                            "{}",
                            deposit.amount as f64 / 100000000_f64
                        ))))
                        .push(text::small(" BTC"))
                        .align_items(Align::Center),
                )
                .width(Length::Shrink),
            )
            .spacing(20)
            .align_items(Align::Center),
    ))
    .into()
}

#[derive(Debug)]
pub struct StakeholderACKDepositView {
    retry_button: iced::button::State,
}

impl StakeholderACKDepositView {
    pub fn new() -> Self {
        StakeholderACKDepositView {
            retry_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        _ctx: &Context,
        warning: Option<&String>,
        deposit: &Vault,
        emergency_tx: &(PartiallySignedTransaction, bool),
        emergency_unvault_tx: &(PartiallySignedTransaction, bool),
        cancel_tx: &(PartiallySignedTransaction, bool),
        signer: Element<'a, DepositMessage>,
    ) -> Element<'a, DepositMessage> {
        let mut row_transactions = Row::new();
        let (_, emergency_signed) = emergency_tx;
        if *emergency_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple("   Emergency TX")))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let (_, emergency_unvault_signed) = emergency_unvault_tx;
        if *emergency_unvault_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple(
                            "   Emergency unvault TX",
                        )))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else if *emergency_signed {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency Unvault TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::grey(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Emergency Unvault TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let (_, cancel_signed) = cancel_tx;
        if *cancel_signed {
            row_transactions = row_transactions.push(
                card::success(Container::new(
                    Row::new()
                        .push(text::success(icon::shield_check_icon()))
                        .push(text::success(text::bold(text::simple("   Cancel TX")))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else if *emergency_unvault_signed {
            row_transactions = row_transactions.push(
                card::border_black(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Cancel TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        } else {
            row_transactions = row_transactions.push(
                card::grey(Container::new(
                    Row::new()
                        .push(icon::shield_icon())
                        .push(text::bold(text::simple("   Cancel TX"))),
                ))
                .width(Length::FillPortion(1)),
            );
        };

        let mut col = Column::new()
            .push(Container::new(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(badge::shield())
                                .push(
                                    Container::new(text::bold(text::small(&deposit.address)))
                                        .align_y(Align::Center),
                                )
                                .spacing(20)
                                .align_items(Align::Center),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    deposit.amount as f64 / 100000000_f64
                                ))))
                                .push(text::small(" BTC"))
                                .align_items(Align::Center),
                        )
                        .width(Length::Shrink),
                    )
                    .spacing(20)
                    .align_items(Align::Center),
            ))
            .push(separation().width(Length::Fill))
            .push(row_transactions.spacing(10))
            .push(signer)
            .spacing(20)
            .push(Column::new());

        if let Some(error) = warning {
            col = col.push(card::alert_warning(Container::new(
                Column::new()
                    .push(Container::new(text::simple(&format!(
                        "Failed to connect to revaultd: {}",
                        error
                    ))))
                    .push(button::primary(
                        &mut self.retry_button,
                        button::button_content(None, "Retry"),
                        DepositMessage::Retry,
                    ))
                    .spacing(20),
            )))
        }

        card::white(Container::new(col)).into()
    }
}
