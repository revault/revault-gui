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
}

impl ManagerHomeView {
    pub fn new() -> Self {
        ManagerHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        balance: &(u64, u64),
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(
                            Row::new()
                                .push(Column::new().width(Length::FillPortion(1)))
                                .push(balance_view(balance).width(Length::FillPortion(1))),
                        )
                        .push(Column::with_children(vaults))
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
}

impl StakeholderHomeView {
    pub fn new() -> Self {
        StakeholderHomeView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
            ack_fund_button: iced::button::State::default(),
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
                                        &mut self.ack_fund_button,
                                        &unsecured_fund_balance,
                                    )
                                    .max_width(400)
                                    .width(Length::Fill),
                                )
                                .push(balance_view(balance).width(Length::Fill))
                                .spacing(20),
                        )
                        .push(Column::with_children(vaults))
                        .spacing(20),
                )),
            )),
        )
        .into()
    }
}

fn unsecured_fund_view<'a>(
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
                                    *fund as f64 / 100000000_f64
                                ))))
                                .push(text::simple("  BTC received since last signing")),
                        )
                        .width(Length::Fill)
                        .align_x(iced::Align::End),
                    )
                    .push(
                        Container::new(button::important(
                            button_state,
                            button::button_content(None, "Acknowledge funds"),
                            Message::Menu(Menu::ACKFunds),
                        ))
                        .width(Length::Fill)
                        .align_x(iced::Align::End),
                    )
                    .spacing(20)
                    .width(Length::Fill),
            ),
    ))
}

/// render balance card from a tuple: (active, inactive)
fn balance_view<'a, T: 'a>(balance: &(u64, u64)) -> Container<'a, T> {
    let col = Column::new()
        .push(text::bold(text::simple("Balance:")))
        .push(
            Row::new()
                .padding(5)
                .push(Container::new(text::simple("active")).width(Length::Fill))
                .push(
                    Container::new(text::bold(text::simple(&format!(
                        "{}",
                        balance.0 as f64 / 100000000_f64
                    ))))
                    .width(Length::Shrink),
                )
                .push(text::simple(" BTC")),
        )
        .push(separation().width(Length::Fill))
        .push(
            Row::new()
                .padding(5)
                .push(Container::new(text::simple("inactive")).width(Length::Fill))
                .push(
                    Container::new(text::bold(text::simple(&format!(
                        "{}",
                        balance.1 as f64 / 100000000_f64
                    ))))
                    .width(Length::Shrink),
                )
                .push(text::simple(" BTC")),
        );

    card::simple(Container::new(col))
}
