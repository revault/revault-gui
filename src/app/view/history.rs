use chrono::NaiveDateTime;
use iced::{pick_list, Align, Column, Container, Element, Length, Row};

use revault_ui::component::{badge, button, card, text::Text, TransparentPickListStyle};

use crate::{
    app::{context::Context, error::Error, message::Message, view::layout},
    daemon::{
        client::Client,
        model::{HistoryEvent, HistoryEventKind},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryFilter {
    Cancel,
    Spend,
    Deposit,
    All,
}

impl HistoryFilter {
    pub const ALL: [HistoryFilter; 4] = [
        HistoryFilter::Cancel,
        HistoryFilter::Spend,
        HistoryFilter::Deposit,
        HistoryFilter::All,
    ];

    pub fn new(kind: &Option<HistoryEventKind>) -> HistoryFilter {
        if let Some(kind) = kind {
            match kind {
                HistoryEventKind::Deposit => HistoryFilter::Deposit,
                HistoryEventKind::Cancel => HistoryFilter::Cancel,
                HistoryEventKind::Spend => HistoryFilter::Spend,
            }
        } else {
            HistoryFilter::All
        }
    }
}

impl std::fmt::Display for HistoryFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Cancel => write!(f, "Cancel"),
            Self::Deposit => write!(f, "Deposit"),
            Self::Spend => write!(f, "Spend"),
            Self::All => write!(f, "All"),
        }
    }
}

/// HistoryView renders a list of vaults filtered by the status filter.
/// If the loading field is true, only the status pick_list component is displayed.
#[derive(Debug)]
pub struct HistoryView {
    dashboard: layout::Dashboard,
    pick_filter: pick_list::State<HistoryFilter>,
    next_button: iced::button::State,
}

impl HistoryView {
    pub fn new() -> Self {
        HistoryView {
            dashboard: layout::Dashboard::new(),
            pick_filter: pick_list::State::default(),
            next_button: iced::button::State::default(),
        }
    }

    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        warning: Option<&Error>,
        events: Vec<Element<'a, Message>>,
        event_kind_filter: &Option<HistoryEventKind>,
        has_next: bool,
    ) -> Element<'a, Message> {
        let mut col = Column::new().push(
            Row::new()
                .push(Column::new().width(Length::Fill))
                .push(
                    pick_list::PickList::new(
                        &mut self.pick_filter,
                        &HistoryFilter::ALL[..],
                        Some(HistoryFilter::new(event_kind_filter)),
                        |filter| {
                            Message::FilterHistoryEvents(match filter {
                                HistoryFilter::Cancel => Some(HistoryEventKind::Cancel),
                                HistoryFilter::Deposit => Some(HistoryEventKind::Deposit),
                                HistoryFilter::Spend => Some(HistoryEventKind::Spend),
                                HistoryFilter::All => None,
                            })
                        },
                    )
                    .text_size(20)
                    .padding(10)
                    .width(Length::Units(200))
                    .style(TransparentPickListStyle),
                )
                .align_items(Align::Center),
        );

        if !events.is_empty() {
            col = col.push(Column::with_children(events).spacing(5));
        }

        if has_next {
            col = col.push(
                button::white_card_button(
                    &mut self.next_button,
                    Container::new(Text::new("See more"))
                        .width(Length::Fill)
                        .align_x(Align::Center),
                )
                .width(Length::Fill)
                .on_press(Message::Next),
            )
        }

        self.dashboard.view(ctx, warning, col.spacing(25))
    }
}

#[derive(Debug)]
pub struct HistoryEventListItemView {}

impl HistoryEventListItemView {
    pub fn view<'a, C: Client>(
        &'a mut self,
        ctx: &Context<C>,
        event: &HistoryEvent,
    ) -> Element<'a, Message> {
        let date = NaiveDateTime::from_timestamp(event.date, 0);
        let mut row = Row::new()
            .push(event_badge(event))
            .push(
                Container::new(
                    Row::new()
                        .push(Text::new(&format!("{}", date)).small())
                        .push(Text::new(&format!("{}", event.kind)).small().bold())
                        .align_items(Align::Center)
                        .spacing(5),
                )
                .width(Length::FillPortion(2)),
            )
            .spacing(10)
            .align_items(Align::Center);

        if let Some(fee) = event.fee {
            row = row.push(
                Container::new(
                    Text::new(&format!("fee: -{}", ctx.converter.converts(fee))).small(),
                )
                .width(Length::FillPortion(1))
                .align_x(Align::End),
            );
        } else {
            row = row.push(
                Container::new(Column::new())
                    .width(Length::FillPortion(1))
                    .align_x(Align::End),
            );
        }
        if let Some(amount) = event.amount {
            let sign = match event.kind {
                HistoryEventKind::Deposit => "+",
                HistoryEventKind::Cancel => "+",
                HistoryEventKind::Spend => "-",
            };
            row = row.push(
                Container::new(Text::new(&format!(
                    "{}{} {}",
                    sign,
                    ctx.converter.converts(amount),
                    ctx.converter.unit,
                )))
                .width(Length::FillPortion(1))
                .align_x(Align::End),
            );
        } else {
            row = row.push(
                Container::new(Column::new())
                    .width(Length::FillPortion(1))
                    .align_x(Align::End),
            );
        }
        card::white(Column::new().push(row).spacing(5)).into()
    }
}

/// event_badge returns a badge headlining the event kind.
fn event_badge<'a, T: 'a>(event: &HistoryEvent) -> Container<'a, T> {
    match &event.kind {
        HistoryEventKind::Deposit => badge::tx_deposit(),
        HistoryEventKind::Cancel => badge::vault_canceled(),
        HistoryEventKind::Spend => badge::vault_spent(),
    }
}
