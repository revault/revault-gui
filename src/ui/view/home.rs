use iced::{scrollable, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    component::{badge, button, card, navbar, separation, text},
    error::Error,
    menu::Menu,
    message::Message,
    view::{layout, sidebar::Sidebar, Context},
};

#[derive(Debug)]
pub struct ManagerHomeView {
    sidebar: Sidebar,
    scroll: scrollable::State,
    deposit_button: iced::button::State,
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        balance: &(u64, u64),
    ) -> Element<'a, Message> {
        let vaults_view = if vaults.is_empty() {
            Column::new().push(card::simple(Container::new(
                Row::new()
                    .push(
                        Container::new(text::simple(
                            "No vaults yet, start using Revault by making a deposit",
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        button::primary(
                            &mut self.deposit_button,
                            button::button_content(None, "Deposit"),
                        )
                        .on_press(Message::Menu(Menu::Deposit)),
                    )
                    .align_items(iced::Align::Center),
            )))
        } else {
            Column::with_children(vaults)
        };
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Column::new().width(Length::FillPortion(1)))
                                .push(balance_view(ctx, balance).width(Length::FillPortion(1))),
                        )
                        .push(vaults_view)
                        .spacing(20),
                )),
            )),
        )
        .into()
    }
}

#[derive(Debug)]
pub struct StakeholderHomeView {
    sidebar: Sidebar,
    scroll: scrollable::State,
    ack_fund_button: iced::button::State,
    deposit_button: iced::button::State,
}

impl StakeholderHomeView {
    pub fn new() -> Self {
        StakeholderHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
            ack_fund_button: iced::button::State::default(),
            deposit_button: iced::button::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        balance: &(u64, u64),
        unsecured_fund_balance: &u64,
    ) -> Element<'a, Message> {
        let vaults_view = if vaults.is_empty() {
            Column::new().push(card::simple(Container::new(
                Row::new()
                    .push(
                        Container::new(text::simple(
                            "No vaults yet, start using Revault by making a deposit",
                        ))
                        .width(Length::Fill),
                    )
                    .push(
                        button::primary(
                            &mut self.deposit_button,
                            button::button_content(None, "Deposit"),
                        )
                        .on_press(Message::Menu(Menu::Deposit)),
                    )
                    .align_items(iced::Align::Center),
            )))
        } else {
            Column::with_children(vaults)
        };
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(
                                    unsecured_fund_view(
                                        ctx,
                                        &mut self.ack_fund_button,
                                        &unsecured_fund_balance,
                                    )
                                    .max_width(400)
                                    .width(Length::Fill),
                                )
                                .push(balance_view(ctx, balance).width(Length::Fill))
                                .spacing(20),
                        )
                        .push(vaults_view)
                        .spacing(20),
                )),
            )),
        )
        .into()
    }
}

fn unsecured_fund_view<'a>(
    ctx: &Context,
    button_state: &'a mut iced::button::State,
    fund: &u64,
) -> Container<'a, Message> {
    card::simple(Container::new(
        Row::new()
            .align_items(iced::Align::Center)
            .push(badge::shield_notif())
            .push(
                Column::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(text::bold(text::simple(&format!(
                                    "{}",
                                    ctx.converter.converts(*fund),
                                ))))
                                .push(text::simple(&format!(
                                    "  {} received since last signing",
                                    ctx.converter.unit
                                ))),
                        )
                        .width(Length::Fill)
                        .align_x(iced::Align::End),
                    )
                    .push(
                        Container::new(
                            button::important(
                                button_state,
                                button::button_content(None, "Acknowledge funds"),
                            )
                            .on_press(Message::Menu(Menu::ACKFunds)),
                        )
                        .width(Length::Fill)
                        .align_x(iced::Align::End),
                    )
                    .spacing(20)
                    .width(Length::Fill),
            ),
    ))
}

/// render balance card from a tuple: (active, inactive)
fn balance_view<'a, T: 'a>(ctx: &Context, balance: &(u64, u64)) -> Container<'a, T> {
    let active_balance = ctx.converter.converts(balance.0);
    let inactive_balance = ctx.converter.converts(balance.1);
    let col = Column::new()
        .push(text::bold(text::simple("Balance:")))
        .push(
            Row::new()
                .padding(5)
                .push(Container::new(text::simple("active")).width(Length::Fill))
                .push(
                    Container::new(text::bold(text::simple(&active_balance.to_string())))
                        .width(Length::Shrink),
                )
                .push(text::simple(&format!(" {}", ctx.converter.unit))),
        )
        .push(separation().width(Length::Fill))
        .push(
            Row::new()
                .padding(5)
                .push(Container::new(text::simple("inactive")).width(Length::Fill))
                .push(
                    Container::new(text::bold(text::simple(&inactive_balance.to_string())))
                        .width(Length::Shrink),
                )
                .push(text::simple(&format!(" {}", ctx.converter.unit))),
        );

    card::simple(Container::new(col))
}
