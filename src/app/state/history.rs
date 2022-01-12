use std::convert::From;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::{Command, Element};

use super::State;

use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::LoadingDashboard,
        view::{HistoryEventListItemView, HistoryView},
    },
    daemon::{
        client::Client,
        model::{HistoryEvent, HistoryEventKind},
    },
};

pub const HISTORY_EVENT_PAGE_SIZE: u64 = 20;

/// HistoryState displays history events.
#[derive(Debug)]
pub enum HistoryState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    Loaded {
        event_kind_filter: Option<HistoryEventKind>,
        events: Vec<HistoryEventListItemState>,
        has_next: bool,
        // Error in case of reload failure.
        warning: Option<Error>,

        view: HistoryView,
    },
}

impl HistoryState {
    pub fn new() -> Self {
        HistoryState::Loading {
            view: LoadingDashboard::new(),
            fail: None,
        }
    }
}

impl<C: Client + Send + Sync + 'static> State<C> for HistoryState {
    fn update(&mut self, ctx: &Context<C>, message: Message) -> Command<Message> {
        match self {
            Self::Loading { fail, .. } => {
                if let Message::HistoryEvents(res) = message {
                    match res {
                        Ok(events) => {
                            let has_next = events.len() as u64 >= HISTORY_EVENT_PAGE_SIZE;
                            *self = Self::Loaded {
                                has_next,
                                event_kind_filter: None,
                                events: events
                                    .into_iter()
                                    .map(|evt| HistoryEventListItemState::new(evt))
                                    .collect(),
                                warning: None,
                                view: HistoryView::new(),
                            };
                        }
                        Err(e) => *fail = Some(Error::RevaultDError(e)),
                    };
                }
            }
            Self::Loaded {
                events,
                warning,
                event_kind_filter,
                has_next,
                ..
            } => match message {
                Message::Reload => {
                    *events = Vec::new();
                    *event_kind_filter = None;
                    return self.load(ctx);
                }
                Message::FilterHistoryEvents(filter) => {
                    *events = Vec::new();
                    *event_kind_filter = filter;
                    let t1 = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let kind = event_kind_filter
                        .as_ref()
                        .map(|filter| vec![filter.clone()])
                        .unwrap_or(vec![
                            HistoryEventKind::Cancel,
                            HistoryEventKind::Deposit,
                            HistoryEventKind::Spend,
                        ]);
                    let revaultd = ctx.revaultd.clone();
                    return Command::perform(
                        async move {
                            revaultd
                                .get_history(kind.as_slice(), 0, t1, u32::MAX.into())
                                .map(|res| res.events)
                        },
                        Message::HistoryEvents,
                    );
                }
                Message::Next => {
                    if let Some(last) = events.last() {
                        let revaultd = ctx.revaultd.clone();
                        let last_event_date = last.event.date as u64;
                        let kind = event_kind_filter
                            .as_ref()
                            .map(|filter| vec![filter.clone()])
                            .unwrap_or(vec![
                                HistoryEventKind::Cancel,
                                HistoryEventKind::Deposit,
                                HistoryEventKind::Spend,
                            ]);
                        return Command::perform(
                            async move {
                                let mut limit = HISTORY_EVENT_PAGE_SIZE;
                                let mut events = revaultd
                                    .get_history(kind.as_slice(), 0, last_event_date, limit)
                                    .map(|res| res.events)?;

                                // because gethistory cursor is inclusive and use blocktime
                                // multiple events can occur in the same block.
                                // If there is more event in the same block that the
                                // HISTORY_EVENT_PAGE_SIZE they can not be retrieved by changing
                                // the cursor value (blocktime) but by increasing the limit.
                                //
                                // 1. Check if the events retrieved have all the same blocktime
                                let blocktime = if let Some(event) = events.first() {
                                    event.date
                                } else {
                                    return Ok(events);
                                };

                                // 2. Retrieve a larger batch of event with the same cursor but
                                //    a larger limit.
                                while !events.iter().any(|evt| evt.date != blocktime)
                                    && events.len() as u64 == limit
                                {
                                    // increments of the equivalent of one page more.
                                    limit += HISTORY_EVENT_PAGE_SIZE;
                                    events = revaultd
                                        .get_history(kind.as_slice(), 0, last_event_date, limit)
                                        .map(|res| res.events)?;
                                }
                                Ok(events)
                            },
                            Message::HistoryEvents,
                        );
                    }
                }
                Message::HistoryEvents(res) => match res {
                    Ok(evts) => {
                        *has_next =
                            !evts.is_empty() && evts.len() as u64 % HISTORY_EVENT_PAGE_SIZE == 0;
                        // gethistory cursor is inclusive and use blocktime.
                        // multiple events can occur in the same block and
                        // if they are included or not in the batch of events
                        // depends of the limit set.
                        let mut new_events: Vec<HistoryEventListItemState> = evts
                            .into_iter()
                            .filter_map(|evt| {
                                if !events.iter().any(|state| state.event == evt) {
                                    Some(HistoryEventListItemState::new(evt))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        events.append(&mut new_events);
                    }
                    Err(e) => *warning = Some(Error::RevaultDError(e)),
                },
                _ => {}
            },
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context<C>) -> Element<Message> {
        match self {
            Self::Loading { fail, view } => view.view(ctx, fail.as_ref()),
            Self::Loaded {
                events,
                warning,
                view,
                event_kind_filter,
                has_next,
            } => view.view(
                ctx,
                warning.as_ref(),
                events.iter_mut().map(|evt| evt.view(ctx)).collect(),
                &event_kind_filter,
                *has_next,
            ),
        }
    }

    // We retrieve the full history
    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        let t1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let revaultd = ctx.revaultd.clone();
        Command::perform(
            async move {
                revaultd
                    .get_history(&HistoryEventKind::ALL, 0, t1, HISTORY_EVENT_PAGE_SIZE)
                    .map(|res| res.events)
            },
            Message::HistoryEvents,
        )
    }
}

impl<C: Client + Send + Sync + 'static> From<HistoryState> for Box<dyn State<C>> {
    fn from(s: HistoryState) -> Box<dyn State<C>> {
        Box::new(s)
    }
}

#[derive(Debug)]
pub struct HistoryEventListItemState {
    event: HistoryEvent,
    view: HistoryEventListItemView,
}

impl HistoryEventListItemState {
    pub fn new(event: HistoryEvent) -> Self {
        Self {
            event,
            view: HistoryEventListItemView {},
        }
    }

    pub fn view<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
    ) -> Element<Message> {
        self.view.view(ctx, &self.event)
    }
}
