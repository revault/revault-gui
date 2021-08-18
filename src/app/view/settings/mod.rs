use iced::{scrollable, Column, Container, Element, Length, Row};

use crate::revault::Role;
use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::{layout, sidebar::Sidebar},
    },
    ui::{
        color,
        component::{badge, card, navbar, scroll, separation, text},
        icon::dot_icon,
    },
};

use crate::revaultd::config::Config;

mod boxes;
use boxes::*;

#[derive(Debug)]
pub struct SettingsView {
    scroll: scrollable::State,
    sidebar: Sidebar,
}

impl SettingsView {
    pub fn new() -> Self {
        SettingsView {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        blockheight: u64,
        config: &Config,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                scroll(
                    &mut self.scroll,
                    Container::new(SettingsView::display_boxes(&ctx, blockheight, &config)),
                )
                .spacing(8),
            )),
        )
        .into()
    }

    pub fn display_boxes<'a>(
        ctx: &Context,
        blockheight: u64,
        config: &Config,
    ) -> Column<'a, Message> {
        let boxes = SettingsBoxes::default();
        let mut column = Column::new()
            .push(boxes.general.display(config))
            .push(bitcoind(blockheight, config));

        match ctx.role {
            Role::Manager => {
                column = column.push(boxes.manager.display(config));
            }
            Role::Stakeholder => {
                column = column.push(boxes.stakeholder.display(config));
            }
        };

        column.push(boxes.scripts.display(config)).spacing(20)
    }
}

pub fn bitcoind<'a, T: 'a>(blockheight: u64, config: &Config) -> Container<'a, T> {
    let mut col = Column::new()
        .push(
            Row::new()
                .push(
                    Row::new()
                        .push(badge::bitcoin_core())
                        .push(text::bold(text::simple("Bitcoin Core")))
                        .spacing(10)
                        .align_items(iced::Align::Center)
                        .width(Length::Fill),
                )
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
        .spacing(20);

    col = col.push(separation().width(Length::Fill));

    if blockheight != 0 {
        col = col.push(
            Row::new()
                .push(
                    Row::new()
                        .push(badge::network())
                        .push(
                            Column::new()
                                .push(text::simple("Network:"))
                                .push(text::bold(text::simple(
                                    &config.bitcoind_config.network.to_string(),
                                ))),
                        )
                        .spacing(10)
                        .width(Length::FillPortion(1)),
                )
                .push(
                    Row::new()
                        .push(badge::block())
                        .push(
                            Column::new()
                                .push(text::simple("Block Height:"))
                                .push(text::bold(text::simple(&blockheight.to_string()))),
                        )
                        .spacing(10)
                        .width(Length::FillPortion(1)),
                ),
        );
    }

    let config = &config.bitcoind_config;
    let rows = vec![
        (
            "Cookie file path",
            config.cookie_path.to_str().unwrap().to_string(),
        ),
        ("Socket address", config.addr.to_string()),
        (
            "Poll interval",
            config
                .poll_interval_secs
                .map(|p| format!("{} seconds", p))
                .unwrap_or_else(|| "Not set".to_string()),
        ),
    ];

    let mut column = Column::new();
    for (k, v) in rows {
        column = column.push(
            Row::new()
                .push(Container::new(text::small(k)).width(Length::Fill))
                .push(text::small(&v)),
        );
    }

    card::simple(Container::new(col.push(column)))
}
