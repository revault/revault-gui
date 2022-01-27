use iced::{Column, Element};

use revaultd::config::Config;

use crate::{
    app::{context::Context, error::Error, message::Message, view::layout},
    daemon::model::ServersStatuses,
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

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        config: &Config,
        server_status: Option<ServersStatuses>,
    ) -> Element<'a, Message> {
        self.dashboard.view(
            ctx,
            warning,
            SettingsView::display_boxes(&ctx, server_status, &config).spacing(8),
        )
    }

    pub fn display_boxes<'a>(
        ctx: &Context,
        server_status: Option<ServersStatuses>,
        config: &Config,
    ) -> Column<'a, Message> {
        if let Some(server_status) = server_status {
            let boxes = SettingsBoxes::new(ctx.blockheight, server_status);
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
