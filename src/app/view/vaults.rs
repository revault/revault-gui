use iced::{pick_list, Alignment, Column, Container, Element, Length, Row};

use revault_ui::component::{text::Text, TransparentPickListStyle};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, VaultFilterMessage},
        view::layout,
    },
    daemon::model::{
        VaultStatus, CURRENT_VAULT_STATUSES, MOVED_VAULT_STATUSES, MOVING_VAULT_STATUSES,
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
        if statuses == MOVING_VAULT_STATUSES {
            VaultsFilter::Moving
        } else if statuses == MOVED_VAULT_STATUSES {
            VaultsFilter::Moved
        } else {
            VaultsFilter::Current
        }
    }

    pub fn statuses(&self) -> &'static [VaultStatus] {
        match self {
            Self::Current => &CURRENT_VAULT_STATUSES,
            Self::Moving => &MOVING_VAULT_STATUSES,
            Self::Moved => &MOVED_VAULT_STATUSES,
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
    dashboard: layout::Dashboard,
    pick_filter: pick_list::State<VaultsFilter>,
}

impl VaultsView {
    pub fn new() -> Self {
        VaultsView {
            dashboard: layout::Dashboard::default(),
            pick_filter: pick_list::State::default(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        warning: Option<&Error>,
        vaults: Vec<Element<'a, Message>>,
        vault_status_filter: &[VaultStatus],
    ) -> Element<'a, Message> {
        let col = Column::new()
            .push(
                Row::new()
                    .push(
                        Container::new(
                            Row::new()
                                .push(Text::new(&format!(" {}", vaults.len())).bold())
                                .push(Text::new(" vaults")),
                        )
                        .width(Length::Fill),
                    )
                    .push(
                        pick_list::PickList::new(
                            &mut self.pick_filter,
                            &VaultsFilter::ALL[..],
                            Some(VaultsFilter::new(vault_status_filter)),
                            |filter| {
                                Message::FilterVaults(VaultFilterMessage::Status(filter.statuses()))
                            },
                        )
                        .text_size(20)
                        .padding(10)
                        .width(Length::Units(200))
                        .style(TransparentPickListStyle),
                    )
                    .align_items(Alignment::Center),
            )
            .push(Column::with_children(vaults).spacing(5));

        self.dashboard.view(ctx, warning, col.spacing(20))
    }
}
