use std::net::SocketAddr;

use iced::{executor, Application, Clipboard, Command, Container, Element, Settings};
use revault_tx::bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use serde_json::json;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{server, sign, view};

pub fn run(cfg: Config) -> iced::Result {
    let settings = Settings::with_flags(cfg);
    App::run(settings)
}

pub struct Config {
    pub keys: Vec<ExtendedPrivKey>,
}

pub struct App {
    signer: sign::Signer,
    status: AppStatus,
}

pub enum AppStatus {
    Waiting,
    Connected {
        addr: SocketAddr,
        writer: Arc<Mutex<server::Writer>>,
        method: Option<Method>,
    },
}

#[derive(Debug)]
pub enum Message {
    Server(server::ServerMessage),
    View(view::ViewMessage),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = Config;

    fn new(cfg: Config) -> (App, Command<Message>) {
        (
            App {
                signer: sign::Signer::new(cfg.keys),
                status: AppStatus::Waiting,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Dummy signer - Revault")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Server(server::ServerMessage::NewConnection(addr, writer)) => {
                self.status = AppStatus::Connected {
                    addr,
                    writer: Arc::new(Mutex::new(writer)),
                    method: None,
                };
                Command::none()
            }
            Message::Server(server::ServerMessage::Request(msg)) => {
                if let AppStatus::Connected { method, .. } = &mut self.status {
                    match serde_json::from_value(msg) {
                        Ok(req) => {
                            *method = Some(Method::new(req));
                        }
                        Err(e) => {
                            println!("{}", e);
                        }
                    }
                }
                Command::none()
            }
            Message::Server(server::ServerMessage::ConnectionDropped) => {
                self.status = AppStatus::Waiting {};
                Command::none()
            }
            Message::View(view::ViewMessage::Confirm) => {
                if let AppStatus::Connected { method, writer, .. } = &mut self.status {
                    match method {
                        Some(Method::SignUnvaultTx {
                            derivation_path,
                            req,
                            ..
                        }) => {
                            self.signer.sign_unvault_tx(derivation_path, req).unwrap();
                            return Command::perform(
                                server::respond(writer.clone(), json!(req)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        _ => {}
                    }
                }
                Command::none()
            }
            _ => Command::none(),
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![server::listen("0.0.0.0:8080").map(Message::Server)])
    }

    fn view(&mut self) -> Element<Message> {
        match &mut self.status {
            AppStatus::Waiting => view::waiting_connection().map(Message::View),
            AppStatus::Connected { addr, method, .. } => match method {
                Some(m) => m.render().map(Message::View),
                None => Container::new(iced::Text::new(format!("Connected to {}", addr)))
                    .align_x(iced::Align::Center)
                    .align_y(iced::Align::Center)
                    .into(),
            },
        }
    }
}

pub enum Method {
    SignSpendTx {
        derivation_path: DerivationPath,
        req: sign::SpendTransaction,
        view: view::SignSpendTxView,
    },
    SignUnvaultTx {
        derivation_path: DerivationPath,
        req: sign::UnvaultTransaction,
        view: view::SignUnvaultTxView,
    },
    SignRevocationTxs {
        derivation_path: DerivationPath,
        req: sign::RevocationTransactions,
        view: view::SignRevocationTxsView,
    },
}

impl Method {
    pub fn new(request: sign::SignRequest) -> Method {
        let derivation_path = request.derivation_path;
        match request.target {
            sign::SignTarget::SpendTransaction(req) => Method::SignSpendTx {
                derivation_path,
                req,
                view: view::SignSpendTxView::new(),
            },
            sign::SignTarget::UnvaultTransaction(req) => Method::SignUnvaultTx {
                derivation_path,
                req,
                view: view::SignUnvaultTxView::new(),
            },
            sign::SignTarget::RevocationTransactions(req) => Method::SignRevocationTxs {
                derivation_path,
                req,
                view: view::SignRevocationTxsView::new(),
            },
        }
    }
    pub fn render(&mut self) -> Element<view::ViewMessage> {
        match self {
            Self::SignSpendTx { view, req, .. } => view.render(),
            Self::SignUnvaultTx { view, req, .. } => view.render(),
            Self::SignRevocationTxs { view, req, .. } => view.render(),
        }
    }
}
