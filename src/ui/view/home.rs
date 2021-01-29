use iced::{scrollable, Column, Container, Element, Length, Row, Scrollable};

use crate::ui::{
    component::{navbar, text},
    error::Error,
    message::{Context, Message},
    view::{layout, sidebar::Sidebar},
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
        balance: &u64,
    ) -> Element<'a, Message> {
        let mut vaults_col = Column::new();
        for vlt in vaults {
            vaults_col = vaults_col.push(vlt);
        }
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(balance_view(balance))
                        .push(vaults_col)
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
        balance: &u64,
    ) -> Element<'a, Message> {
        let mut vaults_col = Column::new();
        for vlt in vaults {
            vaults_col = vaults_col.push(vlt);
        }
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(balance_view(balance))
                        .push(vaults_col)
                        .spacing(20),
                )),
            )),
        )
        .into()
    }
}

fn balance_view<'a, T: 'a>(balance: &u64) -> Container<'a, T> {
    Container::new(
        Row::new().push(Column::new().width(Length::Fill)).push(
            Container::new(
                Row::new()
                    .push(text::large_title(&format!(
                        "{}",
                        *balance as f64 / 100000000_f64
                    )))
                    .push(text::simple(" BTC"))
                    .align_items(iced::Align::Center),
            )
            .width(Length::Shrink),
        ),
    )
    .width(Length::Fill)
}
