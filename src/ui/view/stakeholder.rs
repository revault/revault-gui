use iced::{
    scrollable,
    tooltip::{self, Tooltip},
    Align, Column, Container, Element, Length, Row,
};

use crate::ui::{
    component::{button, card, icon, scroll, text, ContainerBackgroundStyle, TooltipStyle},
    error::Error,
    menu::Menu,
    message::Message,
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
                    Container::new(
                        button::cancel(
                            &mut self.close_button,
                            Container::new(text::simple("X Close")).padding(10),
                        )
                        .on_press(Message::Menu(Menu::Home)),
                    )
                    .width(Length::Shrink),
                ),
            )
            .push(
                Container::new(element)
                    .width(Length::Fill)
                    .align_x(Align::Center),
            )
            .spacing(50);
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
        if vaults.len() > 0 {
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
                            button::cancel(
                                &mut self.close_button,
                                Container::new(text::simple("X Close")).padding(10),
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
