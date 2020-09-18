use std::sync::Arc;

use iced::{button, Align, Column, Container, Element, Length, ProgressBar, Text};
use iced_futures::futures;

use crate::app::App;

/// SyncingProgress specifies the application syncing progress.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncingProgress {
    Pending(f32),
    Finished,
    Errored,
}

pub enum SyncingState {
    Unknown,
    Pending { progress: f32 },
    Synced,
}

/// SyncingManager is the app syncing state.
#[derive(Debug)]
pub struct SyncingManager {
    pub app: Arc<App>,
    pub progress: f32,
    pub button_skip: button::State,
}

impl SyncingManager {
    pub fn new(app: Arc<App>) -> SyncingManager {
        SyncingManager {
            app: app,
            progress: 0.0,
            button_skip: button::State::new(),
        }
    }

    pub fn update(&mut self, message: SyncingProgress) {
        match message {
            SyncingProgress::Pending(progress) => self.progress = progress,
            _ => (),
        };
    }

    pub fn view(&mut self) -> Element<SyncingProgress> {
        let progress_bar = ProgressBar::new(0.0..=100.0, self.progress);
        let content = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(progress_bar);
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

pub struct SyncObserver {}

impl SyncObserver {
    pub fn new() -> SyncObserver {
        SyncObserver {}
    }
}

impl<H, I> iced_native::subscription::Recipe<H, I> for SyncObserver
where
    H: std::hash::Hasher,
{
    type Output = SyncingProgress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            SyncingState::Unknown,
            |state| async move {
                match state {
                    SyncingState::Unknown => Some((
                        SyncingProgress::Pending(00.0),
                        SyncingState::Pending { progress: 00.0 },
                    )),
                    SyncingState::Pending { mut progress } => {
                        if progress > 100.0 as f32 {
                            return Some((SyncingProgress::Finished, SyncingState::Synced));
                        }
                        std::thread::sleep(std::time::Duration::from_millis(2));
                        progress += 0.1;
                        Some((
                            SyncingProgress::Pending(progress),
                            SyncingState::Pending { progress },
                        ))
                    }
                    SyncingState::Synced => {
                        // We do not let the stream die, as it would start a
                        // new download repeatedly if the user is not careful
                        // in case of errors.
                        let _: () = iced::futures::future::pending().await;

                        None
                    }
                }
            },
        ))
    }
}

/// Runner is the app running state manager.
#[derive(Debug)]
pub struct Runner {
    pub app: Arc<App>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Runner {
    pub fn new(app: Arc<App>) -> Runner {
        Runner { app }
    }
    pub fn view(&mut self) -> Element<'static, Message> {
        Text::new("Running...").into()
    }
}
