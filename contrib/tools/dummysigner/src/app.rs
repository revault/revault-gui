use std::net::SocketAddr;

use iced::{executor, Application, Clipboard, Command, Element, Settings};
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
                            target,
                            signed,
                            ..
                        }) => {
                            self.signer
                                .sign_unvault_tx(derivation_path, target)
                                .unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignSpendTx {
                            derivation_paths,
                            target,
                            signed,
                            ..
                        }) => {
                            self.signer.sign_spend_tx(derivation_paths, target).unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignRevocationTxs {
                            derivation_path,
                            target,
                            signed,
                            ..
                        }) => {
                            self.signer
                                .sign_revocation_txs(derivation_path, target)
                                .unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        _ => {}
                    }
                }
                Command::none()
            }
            Message::View(view::ViewMessage::Cancel) => {
                if let AppStatus::Connected { method, writer, .. } = &mut self.status {
                    *method = None;
                    return Command::perform(
                        server::respond(writer.clone(), json!({"request_status": "refused"})),
                        server::ServerMessage::Responded,
                    )
                    .map(Message::Server);
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
                None => view::connected(addr).map(Message::View),
            },
        }
    }
}

pub enum Method {
    SignSpendTx {
        derivation_paths: Vec<DerivationPath>,
        target: sign::SpendTransaction,
        signed: bool,
        view: view::SignSpendTxView,
    },
    SignUnvaultTx {
        derivation_path: DerivationPath,
        target: sign::UnvaultTransaction,
        signed: bool,
        view: view::SignUnvaultTxView,
    },
    SignRevocationTxs {
        derivation_path: DerivationPath,
        target: sign::RevocationTransactions,
        signed: bool,
        view: view::SignRevocationTxsView,
    },
}

impl Method {
    pub fn new(request: sign::SignRequest) -> Method {
        match request {
            sign::SignRequest::SpendTransaction {
                derivation_paths,
                target,
            } => Method::SignSpendTx {
                derivation_paths,
                target,
                signed: false,
                view: view::SignSpendTxView::new(),
            },
            sign::SignRequest::UnvaultTransaction {
                derivation_path,
                target,
            } => Method::SignUnvaultTx {
                derivation_path,
                target,
                signed: false,
                view: view::SignUnvaultTxView::new(),
            },
            sign::SignRequest::RevocationTransactions {
                derivation_path,
                target,
            } => Method::SignRevocationTxs {
                derivation_path,
                target,
                signed: false,
                view: view::SignRevocationTxsView::new(),
            },
        }
    }
    pub fn render(&mut self) -> Element<view::ViewMessage> {
        match self {
            Self::SignSpendTx {
                view,
                target,
                signed,
                ..
            } => view.render(&target, *signed),
            Self::SignUnvaultTx {
                view,
                target,
                signed,
                ..
            } => view.render(&target, *signed),
            Self::SignRevocationTxs {
                view,
                target,
                signed,
                ..
            } => view.render(&target, *signed),
        }
    }
}
