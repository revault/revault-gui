use iced::{scrollable, Column, Container, Element, Length, Row};

use crate::ui::{
    color,
    component::{badge, card, navbar, scroll, text},
    error::Error,
    icon::dot_icon,
    message::Message,
    view::{layout, sidebar::Sidebar, Context},
};

#[derive(Debug)]
pub struct ManagerNetworkView {
    sidebar: Sidebar,
    scroll: scrollable::State,
}

impl ManagerNetworkView {
    pub fn new() -> Self {
        ManagerNetworkView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        blockheight: Option<&u64>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(
                    Column::new()
                        .push(bitcoin_core_card(blockheight))
                        .spacing(20),
                ),
            ))),
        )
        .into()
    }
}

#[derive(Debug)]
pub struct StakeholderNetworkView {
    sidebar: Sidebar,
    scroll: scrollable::State,
}

impl StakeholderNetworkView {
    pub fn new() -> Self {
        StakeholderNetworkView {
            scroll: scrollable::State::new(),
            sidebar: Sidebar::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        blockheight: Option<&u64>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(
                    Column::new()
                        .push(bitcoin_core_card(blockheight))
                        .spacing(20),
                ),
            ))),
        )
        .into()
    }
}

fn bitcoin_core_card<'a, T: 'a>(blockheight: Option<&u64>) -> Container<'a, T> {
    let mut col = Column::new()
        .push(
            Row::new()
                .push(Container::new(text::bold(text::simple("Bitcoin Core"))).width(Length::Fill))
                .push(
                    Container::new(
                        Row::new()
                            .push(dot_icon().size(5).color(color::SUCCESS))
                            .push(text::small("Running").color(color::SUCCESS))
                            .align_items(iced::Align::Center),
                    )
                    .width(Length::Shrink),
                ),
        )
        .spacing(10);
    if let Some(b) = blockheight {
        col = col.push(
            Row::new()
                .push(badge::block())
                .push(
                    Column::new()
                        .push(text::bold(text::simple("Block Height")))
                        .push(text::simple(&b.to_string())),
                )
                .spacing(10),
        );
    }
    card::simple(Container::new(col))
}
