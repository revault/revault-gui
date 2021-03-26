use iced::{scrollable, Column, Container, Element, Scrollable};

use crate::revault::Role;
use crate::ui::{
    component::navbar,
    error::Error,
    message::Message,
    view::{layout, sidebar::Sidebar, Context},
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
        config: Config,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll)
                    .push(SettingsView::display_boxes(&ctx, &config))
                    .spacing(8),
            )),
        )
        .into()
    }

    pub fn display_boxes<'a>(ctx: &Context, config: &Config) -> Column<'a, Message> {
        let boxes = SettingsBoxes::default();
        let mut column = Column::new()
            .push(boxes.general.display(config))
            .push(boxes.bitcoind.display(config));

        match ctx.role {
            Role::Manager => {
                column = column.push(boxes.manager.display(config));
            }
            Role::Stakeholder => {
                column = column.push(boxes.stakeholder.display(config));
            }
        };

        column
            .push(boxes.stakeholder_xpubs.display(config))
            .push(boxes.manager_xpubs.display(config))
            .push(boxes.cosigner_keys.display(config))
            .spacing(20)
    }
}
