use std::convert::From;
use std::time::{SystemTime, UNIX_EPOCH};

use iced::{Command, Element};

use super::{cmd::get_history, State};

use crate::{
    app::{
        context::Context,
        error::Error,
        message::Message,
        view::LoadingDashboard,
        view::{HistoryEventView, HistoryView},
    },
    daemon::{
        client::Client,
        model::{HistoryEvent, HistoryEventKind},
    },
};

/// HistoryState displays history events.
#[derive(Debug)]
pub enum HistoryState {
    Loading {
        fail: Option<Error>,
        view: LoadingDashboard,
    },
    Loaded {
        event_kind_filter: Option<HistoryEventKind>,
        events: Vec<HistoryEventState>,
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
                            *self = Self::Loaded {
                                event_kind_filter: None,
                                events: events
                                    .into_iter()
                                    .map(|evt| HistoryEventState::new(evt))
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
                ..
            } => match message {
                Message::Reload => return self.load(ctx),
                Message::FilterHistoryEvents(filter) => {
                    let t1 = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    *event_kind_filter = filter;
                    return Command::perform(
                        get_history(
                            ctx.revaultd.clone(),
                            event_kind_filter
                                .as_ref()
                                .map(|filter| vec![filter.clone()])
                                .unwrap_or(vec![
                                    HistoryEventKind::Deposit,
                                    HistoryEventKind::Cancel,
                                    HistoryEventKind::Spend,
                                ]),
                            0,
                            t1,
                            u32::MAX.into(),
                        ),
                        Message::HistoryEvents,
                    );
                }
                Message::HistoryEvents(res) => match res {
                    Ok(evts) => {
                        *events = evts
                            .into_iter()
                            .map(|evt| HistoryEventState::new(evt))
                            .collect();
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
            } => view.view(
                ctx,
                warning.as_ref(),
                events.iter_mut().map(|evt| evt.view(ctx)).collect(),
                &event_kind_filter,
            ),
        }
    }

    // We retrieve the full history
    fn load(&self, ctx: &Context<C>) -> Command<Message> {
        let t1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Command::perform(
            get_history(
                ctx.revaultd.clone(),
                vec![
                    HistoryEventKind::Cancel,
                    HistoryEventKind::Deposit,
                    HistoryEventKind::Spend,
                ],
                0,
                t1,
                u32::MAX.into(),
            ),
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
pub struct HistoryEventState {
    event: HistoryEvent,
    view: HistoryEventView,
}

impl HistoryEventState {
    pub fn new(event: HistoryEvent) -> Self {
        Self {
            event,
            view: HistoryEventView {},
        }
    }

    pub fn view<C: Client + Send + Sync + 'static>(
        &mut self,
        ctx: &Context<C>,
    ) -> Element<Message> {
        self.view.view(ctx, &self.event)
    }
}
