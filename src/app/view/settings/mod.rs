use iced::{Column, Element};

use crate::{
    app::{context::Context, error::Error, message::Message, view::layout},
    daemon::{client::Client, config::Config, model::ServersStatuses},
    revault::Role,
};

mod boxes;
use boxes::*;

#[derive(Debug)]
pub struct SettingsView {
    dashboard: layout::Dashboard,
}

impl SettingsView {
    pub fn new() -> Self {
        SettingsView {
            dashboard: layout::Dashboard::new(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        blockheight: u64,
        config: &Config,
        server_status: Option<ServersStatuses>,
    ) -> Element<'a, Message> {
        self.dashboard.view(
            ctx,
            warning,
            SettingsView::display_boxes(&ctx, blockheight, server_status, &config).spacing(8),
        )
    }

    pub fn display_boxes<'a, C: Client>(
        ctx: &Context<C>,
        blockheight: u64,
        server_status: Option<ServersStatuses>,
        config: &Config,
    ) -> Column<'a, Message> {
        if let Some(server_status) = server_status {
            let boxes = SettingsBoxes::new(blockheight, server_status);
            let mut column = Column::new()
                .push(boxes.bitcoin.display(config))
                .push(boxes.coordinator.display(config));

            match ctx.role {
                Role::Manager => {
                    for c in boxes.cosigners {
                        column = column.push(c.display(config));
                    }
                }
                Role::Stakeholder => {
                    for w in boxes.watchtowers {
                        column = column.push(w.display(config));
                    }
                }
            };

            column.push(boxes.advanced.display(config)).spacing(20)
        } else {
            Column::new()
        }
    }
}
