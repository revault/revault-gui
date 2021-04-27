use iced::{pick_list, scrollable, Align, Column, Container, Element, Length, Row};

use crate::{
    revaultd::model::VaultStatus,
    ui::{
        component::{navbar, scroll, text, TransparentPickListStyle},
        error::Error,
        message::{Message, VaultFilterMessage},
        view::{layout, sidebar::Sidebar, Context},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VaultsFilter {
    Current,
    Moving,
    Moved,
}

impl VaultsFilter {
    pub const ALL: [VaultsFilter; 3] = [
        VaultsFilter::Current,
        VaultsFilter::Moving,
        VaultsFilter::Moved,
    ];

    pub fn new(statuses: &[VaultStatus]) -> VaultsFilter {
        if statuses == VaultStatus::MOVING {
            return VaultsFilter::Moving;
        } else if statuses == VaultStatus::MOVED {
            return VaultsFilter::Moved;
        } else {
            return VaultsFilter::Current;
        }
    }

    pub fn statuses(&self) -> &'static [VaultStatus] {
        match self {
            Self::Current => &VaultStatus::CURRENT,
            Self::Moving => &VaultStatus::MOVING,
            Self::Moved => &VaultStatus::MOVED,
        }
    }
}

impl std::fmt::Display for VaultsFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Current => write!(f, "Current"),
            Self::Moving => write!(f, "Moving"),
            Self::Moved => write!(f, "Moved"),
        }
    }
}

/// VaultsView renders a list of vaults filtered by the status filter.
/// If the loading field is true, only the status pick_list component is displayed.
#[derive(Debug)]
pub struct VaultsView {
    scroll: scrollable::State,
    sidebar: Sidebar,
    pick_filter: pick_list::State<VaultsFilter>,
}

impl VaultsView {
    pub fn new() -> Self {
        VaultsView {
            sidebar: Sidebar::new(),
            scroll: scrollable::State::new(),
            pick_filter: pick_list::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        vault_status_filter: &[VaultStatus],
        loading: bool,
    ) -> Element<'a, Message> {
        let mut col = Column::new();

        if !loading {
            col = col
                .push(
                    Row::new()
                        .push(
                            Container::new(
                                Row::new()
                                    .push(text::bold(text::simple(&format!(" {}", vaults.len()))))
                                    .push(text::simple(" vaults")),
                            )
                            .width(Length::Fill),
                        )
                        .push(
                            pick_list::PickList::new(
                                &mut self.pick_filter,
                                &VaultsFilter::ALL[..],
                                Some(VaultsFilter::new(vault_status_filter)),
                                |filter| {
                                    Message::FilterVaults(VaultFilterMessage::Status(
                                        filter.statuses(),
                                    ))
                                },
                            )
                            .text_size(15)
                            .padding(10)
                            .width(Length::Units(200))
                            .style(TransparentPickListStyle),
                        )
                        .align_items(Align::Center),
                )
                .push(Column::with_children(vaults).spacing(5));
        } else {
            col = col.push(
                Row::new()
                    .push(Container::new(Row::new()).width(Length::Fill))
                    .push(
                        pick_list::PickList::new(
                            &mut self.pick_filter,
                            &VaultsFilter::ALL[..],
                            Some(VaultsFilter::new(vault_status_filter)),
                            |filter| {
                                Message::FilterVaults(VaultFilterMessage::Status(filter.statuses()))
                            },
                        )
                        .padding(10)
                        .width(Length::Units(200))
                        .style(TransparentPickListStyle),
                    )
                    .align_items(Align::Center),
            );
        }

        layout::dashboard(
            navbar(layout::navbar_warning(warning)),
            self.sidebar.view(ctx),
            layout::main_section(Container::new(scroll(
                &mut self.scroll,
                Container::new(col.spacing(20)),
            ))),
        )
        .into()
    }
}
