use iced::{scrollable, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    component::{card, navbar, separation, text},
    error::Error,
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
}

impl StakeholderHomeView {
    pub fn new() -> Self {
        StakeholderHomeView {
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

fn balance_view<'a, T: 'a>(balance: &(u64, u64)) -> Container<'a, T> {
    let col = Column::new()
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
