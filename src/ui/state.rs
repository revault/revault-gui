use std::sync::Arc;

use iced::{button, Align, Button, Column, Container, Element, Length, ProgressBar, Text};
use iced_futures::futures;

use crate::app::App;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncingProgress {
    Pending(f32),
    Finished,
    Errored,
}

#[derive(Debug)]
pub struct SyncingState {
    pub app: Arc<App>,
    pub progress: f32,
    pub button_skip: button::State,
}

impl SyncingState {
    pub fn new(app: Arc<App>) -> SyncingState {
        SyncingState {
            app: app,
            progress: 100.0,
            button_skip: button::State::new(),
        }
    }

    pub fn view(&mut self) -> Element<SyncingProgress> {
        let progress_bar = ProgressBar::new(0.0..=100.0, self.progress);
        let button = Button::new(&mut self.button_skip, Text::new("Skip"))
            .on_press(SyncingProgress::Finished);
        let content = Column::new()
            .spacing(10)
            .padding(10)
            .align_items(Align::Center)
            .push(progress_bar)
            .push(button);
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug)]
pub struct RunningState {
    pub app: Arc<App>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl RunningState {
    pub fn new(app: Arc<App>) -> RunningState {
        RunningState { app: app }
    }
    pub fn view(&mut self) -> Element<'static, Message> {
        Text::new("Running...").into()
    }
}
