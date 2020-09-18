use iced::{executor, Application, Command, Element, Subscription};
use std::sync::Arc;

pub mod state;
use crate::app::App;

/// UI is the main iced application.
#[derive(Debug)]
pub enum UI {
    // AppSyncing is the syncing state of the underlying application.
    AppSyncing(state::SyncingState),
    // AppRunning is the running state of the underlying application.
    AppRunning(state::RunningState),
}

impl UI {
    /// start changes the UI application state from syncing to running.
    pub fn start(&mut self) {
        if let UI::AppSyncing(state) = self {
            let app = state.app.clone();
            *self = UI::AppRunning(state::RunningState::new(app));
        }
    }
}

/// Message is the UI application message.
#[derive(Debug, Clone)]
pub enum Message {
    Syncing(state::SyncingProgress),
    Running(state::Message),
}

impl Application for UI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (UI, Command<Message>) {
        let app = Arc::new(App::new());
        (
            UI::AppSyncing(state::SyncingState::new(app)),
            Command::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            UI::AppSyncing(_state) => Subscription::none(),
            _ => Subscription::none(),
        }
    }

    fn title(&self) -> String {
        String::from("Robalo")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Syncing(sync) => {
                if sync == state::SyncingProgress::Finished {
                    self.start();
                }
            }
            Message::Running(_) => (),
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            UI::AppSyncing(s) => s.view().map(Message::Syncing),
            UI::AppRunning(s) => s.view().map(Message::Running),
        }
    }
}
