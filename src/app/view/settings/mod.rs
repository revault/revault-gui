use iced::{scrollable, Column, Container, Element};

use crate::revault::Role;
use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::{layout, sidebar::Sidebar},
    },
    ui::component::{navbar, scroll},
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
        let boxes = SettingsBoxes::new(blockheight);
        let mut column = Column::new()
            .push(boxes.general.display(config))
            .push(boxes.bitcoin.display(config));

        match ctx.role {
            Role::Manager => {
                column = column.push(boxes.manager.display(config));
            }
            Role::Stakeholder => {
                column = column.push(boxes.stakeholder.display(config));
            }
        };

        column.spacing(20)
    }
}
