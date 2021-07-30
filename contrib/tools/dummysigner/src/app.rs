use std::net::SocketAddr;

use iced::{executor, Application, Clipboard, Command, Container, Element, Settings};
use serde_json::json;

use crate::server;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn run() -> iced::Result {
    let settings = Settings::default();
    App::run(settings)
}

pub enum App {
    Waiting,
    Connected {
        addr: SocketAddr,
        writer: Arc<Mutex<server::Writer>>,
        message: Option<serde_json::Value>,
    },
}

#[derive(Debug)]
pub enum Message {
    Server(server::ServerMessage),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        (App::Waiting {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Dummy signer - Revault")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Server(server::ServerMessage::NewConnection(addr, writer)) => {
                *self = Self::Connected {
                    addr,
                    writer: Arc::new(Mutex::new(writer)),
                    message: None,
                };
                Command::none()
            }
            Message::Server(server::ServerMessage::Request(msg)) => {
                if let Self::Connected {
                    message, writer, ..
                } = self
                {
                    *message = Some(msg);
                    return Command::perform(
                        server::respond(writer.clone(), json!({"hello": "edouard"})),
                        server::ServerMessage::Responded,
                    )
                    .map(Message::Server);
                }
                Command::none()
            }
            Message::Server(server::ServerMessage::ConnectionDropped) => {
                *self = Self::Waiting;
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![server::listen("0.0.0.0:8080").map(Message::Server)])
    }

    fn view(&mut self) -> Element<Message> {
        match self {
            Self::Waiting => Container::new(iced::Text::new("waiting"))
                .align_x(iced::Align::Center)
                .align_y(iced::Align::Center)
                .into(),
            Self::Connected { addr, message, .. } => {
                Container::new(iced::Text::new(format!("{} {:?}", addr, message)))
                    .align_x(iced::Align::Center)
                    .align_y(iced::Align::Center)
                    .into()
            }
        }
    }
}
