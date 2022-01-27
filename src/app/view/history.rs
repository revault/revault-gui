use chrono::NaiveDateTime;
use iced::{pick_list, Align, Column, Container, Element, Length, Row};

use revault_ui::{
    component::{badge, button, card, separation, text::Text, TransparentPickListStyle},
    icon,
};

use crate::{
    app::{context::Context, error::Error, message::Message, view::layout},
    daemon::model::{HistoryEvent, HistoryEventKind, VaultTransactions},
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

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
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
pub struct HistoryEventListItemView {
    select_button: iced::button::State,
}

impl HistoryEventListItemView {
    pub fn new() -> Self {
        Self {
            select_button: iced::button::State::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        event: &HistoryEvent,
        index: usize,
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

        button::white_card_button(&mut self.select_button, Container::new(row))
            .on_press(Message::SelectHistoryEvent(index))
            .into()
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

#[derive(Debug)]
pub struct HistoryEventView {
    modal: layout::Modal,
}

impl HistoryEventView {
    pub fn new() -> Self {
        Self {
            modal: layout::Modal::new(),
        }
    }

    pub fn view<'a>(
        &'a mut self,
        ctx: &Context,
        event: &HistoryEvent,
        txs: &Vec<VaultTransactions>,
        warning: Option<&Error>,
    ) -> Element<'a, Message> {
        if txs.is_empty() {
            return self.modal.view(
                ctx,
                warning,
                Container::new(Column::new()),
                None,
                Message::Close,
            );
        }

        let content: Element<Message> = match event.kind {
            HistoryEventKind::Deposit => deposit(ctx, event),
            HistoryEventKind::Cancel => cancel(ctx, event),
            HistoryEventKind::Spend => spend(ctx, event, txs),
        };

        self.modal.view(
            ctx,
            warning,
            Container::new(content).padding(20).max_width(800),
            None,
            Message::Close,
        )
    }
}

fn date_and_blockheight<'a, T: 'a>(event: &HistoryEvent) -> Container<'a, T> {
    Container::new(
        Row::new()
            .push(
                Container::new(
                    Row::new()
                        .push(Container::new(
                            icon::calendar_icon().size(30).width(Length::Fill),
                        ))
                        .push(
                            Column::new()
                                .push(Text::new("Date:").bold())
                                .push(Text::new(&format!(
                                    "{}",
                                    NaiveDateTime::from_timestamp(event.date, 0)
                                ))),
                        )
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .align_x(Align::Center)
                .width(Length::FillPortion(1)),
            )
            .push(
                Container::new(
                    Row::new()
                        .push(Container::new(
                            icon::block_icon().size(30).width(Length::Fill),
                        ))
                        .push(
                            Column::new()
                                .push(Text::new("Block Height:").bold())
                                .push(Text::new(&event.blockheight.to_string())),
                        )
                        .align_items(Align::Center)
                        .spacing(20),
                )
                .align_x(Align::Center)
                .width(Length::FillPortion(1)),
            )
            .spacing(20),
    )
}

fn deposit<'a, T: 'a>(ctx: &Context, event: &HistoryEvent) -> Element<'a, T> {
    Column::new()
        .push(
            Row::new()
                .push(event_badge(event))
                .push(Text::new("Deposit").bold())
                .spacing(5)
                .align_items(Align::Center),
        )
        .push(
            Container::new(
                Text::new(&format!(
                    "+ {} {}",
                    ctx.converter
                        .converts(event.amount.expect("This is a deposit event")),
                    ctx.converter.unit,
                ))
                .bold()
                .size(50),
            )
            .padding(30),
        )
        .push(card::white(
            Column::new()
                .push(date_and_blockheight(&event))
                .push(separation().width(Length::Fill))
                .push(
                    Row::new()
                        .push(Text::new("Outpoint:").bold().width(Length::Fill))
                        .push(Text::new(&format!("{}", event.vaults[0])).small()),
                )
                .spacing(20),
        ))
        .align_items(Align::Center)
        .spacing(20)
        .into()
}

fn cancel<'a, T: 'a>(ctx: &Context, event: &HistoryEvent) -> Element<'a, T> {
    Column::new()
        .push(
            Row::new()
                .push(event_badge(event))
                .push(Text::new("Cancel").bold())
                .spacing(5)
                .align_items(Align::Center),
        )
        .push(Container::new(Text::new(&format!(
            "Fee: {} {}",
            ctx.converter.converts(event.fee.unwrap_or(0)),
            ctx.converter.unit,
        ))))
        .push(card::white(
            Column::new()
                .push(date_and_blockheight(&event))
                .push(separation().width(Length::Fill))
                .push(
                    Row::new()
                        .push(Text::new("Tx ID:").bold().width(Length::Fill))
                        .push(Text::new(&format!("{}", event.txid)).small()),
                )
                .push(
                    Row::new()
                        .push(Text::new("Vault:").bold().width(Length::Fill))
                        .push(Text::new(&format!("{}", event.vaults[0])).small()),
                )
                .spacing(20),
        ))
        .align_items(Align::Center)
        .spacing(20)
        .into()
}

fn spend<'a, T: 'a>(
    ctx: &Context,
    event: &HistoryEvent,
    txs: &Vec<VaultTransactions>,
) -> Element<'a, T> {
    let tx = &txs.first().as_ref().unwrap().spend.as_ref().unwrap().tx;
    let mut col_recipients = Column::new()
        .push(Text::new("Recipients:").bold())
        .spacing(10);
    for output in &tx.output {
        let addr = bitcoin::Address::from_script(&output.script_pubkey, ctx.network());
        let mut row = Row::new();
        if let Some(a) = addr {
            row = row.push(Text::new(&a.to_string()).small().width(Length::Fill))
        } else {
            row = row.push(
                Text::new(&output.script_pubkey.to_string())
                    .small()
                    .width(Length::Fill),
            )
        }
        col_recipients = col_recipients
            .push(
                card::simple(Container::new(
                    row.push(
                        Text::new(&ctx.converter.converts(output.value).to_string())
                            .bold()
                            .small()
                            .width(Length::Shrink),
                    ),
                ))
                .width(Length::Fill),
            )
            .width(Length::FillPortion(1));
    }

    Column::new()
        .push(
            Row::new()
                .push(event_badge(event))
                .push(Text::new("Spend").bold())
                .spacing(5)
                .align_items(Align::Center),
        )
        .push(
            Column::new()
                .push(
                    Text::new(&format!(
                        "- {} {}",
                        ctx.converter.converts(event.amount.unwrap_or(0)),
                        ctx.converter.unit,
                    ))
                    .bold()
                    .size(50),
                )
                .push(Container::new(Text::new(&format!(
                    "Fee: {} {}",
                    ctx.converter.converts(event.fee.unwrap_or(0)),
                    ctx.converter.unit,
                ))))
                .align_items(Align::Center),
        )
        .push(card::white(
            Column::new()
                .push(date_and_blockheight(&event))
                .push(separation().width(Length::Fill))
                .push(
                    Row::new()
                        .push(Text::new("Tx ID:").bold().width(Length::Fill))
                        .push(Text::new(&format!("{}", event.txid)).small()),
                )
                .spacing(20),
        ))
        .push(col_recipients)
        .align_items(Align::Center)
        .spacing(20)
        .into()
}
