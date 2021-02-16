use iced::{scrollable, Column, Container, Element, Scrollable};

use crate::ui::{
    component::navbar,
    error::Error,
    message::Message,
    view::{layout, sidebar::Sidebar, Context},
};

#[derive(Debug)]
pub struct HistoryView {
    scroll: scrollable::State,
    sidebar: Sidebar,
}

impl HistoryView {
    pub fn new() -> Self {
        HistoryView {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(
                Scrollable::new(&mut self.scroll).push(Container::new(
                    Column::new()
                        .push(Column::with_children(vaults))
                        .spacing(20),
                )),
            )),
        )
        .into()
    }
}
