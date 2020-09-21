use iced::{executor, Application, Command, Element, Subscription};
use std::sync::Arc;

pub mod view;
use crate::app::App;

/// UI is the main iced application.
#[derive(Debug)]
pub enum UI {
    // AppSyncing is the syncing state of the underlying application.
    AppSyncing {
        app: Arc<App>,
        progress: view::syncing::Progress,
    },
    // AppRunning is the running state of the underlying application.
    AppRunning {
        app: Arc<App>,
    },
}

impl UI {
    pub fn new_syncing(app: Arc<App>) -> UI {
        UI::AppSyncing {
            app,
            progress: view::syncing::Progress::Pending(0.0),
        }
    }
    pub fn set_progress(&mut self, p: view::syncing::Progress) {
        match self {
            UI::AppSyncing {
                ref mut progress, ..
            } => *progress = p,
            _ => (),
        }
    }
    /// start changes the UI application state from syncing to running.
    pub fn start(&mut self) {
        if let UI::AppSyncing { app, .. } = self {
            let app = app.clone();
            *self = UI::AppRunning { app };
        }
    }
}

/// Message is the UI application message.
#[derive(Debug, Clone)]
pub enum Message {
    Syncing(view::syncing::Progress),
    Running,
}

impl Application for UI {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (UI, Command<Message>) {
        let app = Arc::new(App::new());
        (UI::new_syncing(app), Command::none())
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            UI::AppSyncing { .. } => {
                Subscription::from_recipe(view::syncing::SyncObserver::new()).map(Message::Syncing)
            }
            _ => Subscription::none(),
        }
    }

    fn title(&self) -> String {
        String::from("Robalo")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Syncing(msg) => {
                if msg == view::syncing::Progress::Finished {
                    self.start();
                } else {
                    match self {
                        UI::AppSyncing { .. } => self.set_progress(msg),
                        UI::AppRunning { .. } => (),
                    }
                }
            }
            Message::Running => (),
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            UI::AppSyncing { progress, .. } => view::syncing::view(progress).map(Message::Syncing),
            UI::AppRunning { .. } => {
                use iced::Text;
                Text::new("Application running...").into()
            }
        }
    }
}
