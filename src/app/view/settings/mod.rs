use iced::{scrollable, Column, Container, Element};

use revault_ui::component::{navbar, scroll};

use crate::revault::Role;
use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::{layout, sidebar::Sidebar},
    },
    daemon::client::Client,
};

use crate::daemon::{config::Config, model::ServersStatuses};

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

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        blockheight: u64,
        config: &Config,
        server_status: Option<ServersStatuses>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                scroll(
                    &mut self.scroll,
                    Container::new(SettingsView::display_boxes(
                        &ctx,
                        blockheight,
                        server_status,
                        &config,
                    )),
                )
                .spacing(8),
            )),
        )
        .into()
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
