use std::net::SocketAddr;

use iced::{executor, Application, Clipboard, Command, Element, Settings};
use serde_json::json;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{api, config::Config, server, sign, view};

pub fn run(cfg: Config) -> iced::Result {
    let settings = Settings::with_flags(cfg);
    App::run(settings)
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
                signer: sign::Signer::new(
                    cfg.keys,
                    cfg.descriptors.map(|d| sign::Descriptors {
                        deposit_descriptor: d.deposit_descriptor,
                        unvault_descriptor: d.unvault_descriptor,
                        cpfp_descriptor: d.cpfp_descriptor,
                    }),
                    cfg.emergency_address,
                ),
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
                if let AppStatus::Connected { method, writer, .. } = &mut self.status {
                    match serde_json::from_value(msg) {
                        Ok(req) => {
                            if (matches!(req, api::Request::SecureBatch { .. })
                                && !(self.signer.has_descriptors()
                                    && self.signer.has_emergency_address()))
                                || (matches!(req, api::Request::DelegateBatch { .. })
                                    && !self.signer.has_descriptors())
                            {
                                return Command::perform(
                                    server::respond(
                                        writer.clone(),
                                        json!({"error": "batch unsupported"}),
                                    ),
                                    server::ServerMessage::Responded,
                                )
                                .map(Message::Server);
                            }
                            *method = Some(Method::new(req));
                        }
                        Err(_) => {
                            return Command::perform(
                                server::respond(
                                    writer.clone(),
                                    json!({"error": "request unknown"}),
                                ),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
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
                        Some(Method::SignUnvaultTx { target, signed, .. }) => {
                            self.signer.sign_psbt(&mut target.unvault_tx).unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignSpendTx { target, signed, .. }) => {
                            self.signer.sign_psbt(&mut target.spend_tx).unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignRevocationTxs { target, signed, .. }) => {
                            self.signer.sign_psbt(&mut target.emergency_tx).unwrap();
                            self.signer
                                .sign_psbt(&mut target.emergency_unvault_tx)
                                .unwrap();
                            self.signer.sign_psbt(&mut target.cancel_tx).unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SecureBatch { target, signed, .. }) => {
                            let mut response = Vec::new();
                            for deposit in target.deposits.iter().clone() {
                                let mut revocation_txs = self
                                    .signer
                                    .derive_revocation_txs(
                                        deposit.outpoint,
                                        deposit.amount,
                                        deposit.derivation_index,
                                    )
                                    .unwrap();

                                self.signer
                                    .sign_psbt(&mut revocation_txs.emergency_tx)
                                    .unwrap();

                                self.signer
                                    .sign_psbt(&mut revocation_txs.cancel_tx)
                                    .unwrap();

                                self.signer
                                    .sign_psbt(&mut revocation_txs.emergency_unvault_tx)
                                    .unwrap();

                                response.push(api::RevocationTransactions {
                                    cancel_tx: revocation_txs.cancel_tx,
                                    emergency_tx: revocation_txs.emergency_tx,
                                    emergency_unvault_tx: revocation_txs.emergency_unvault_tx,
                                });
                            }
                            *signed = true;
                            return Command::perform(
                                server::respond(
                                    writer.clone(),
                                    json!({ "transactions": response }),
                                ),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::DelegateBatch { target, signed, .. }) => {
                            let mut response = Vec::new();
                            for deposit in target.vaults.iter().clone() {
                                let mut unvault_tx = self
                                    .signer
                                    .derive_unvault_tx(
                                        deposit.outpoint,
                                        deposit.amount,
                                        deposit.derivation_index,
                                    )
                                    .unwrap();

                                self.signer.sign_psbt(&mut unvault_tx).unwrap();

                                response.push(api::UnvaultTransaction { unvault_tx });
                            }
                            *signed = true;
                            return Command::perform(
                                server::respond(
                                    writer.clone(),
                                    json!({ "transactions": response }),
                                ),
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
        target: api::SpendTransaction,
        signed: bool,
        view: view::SignSpendTxView,
    },
    SignUnvaultTx {
        target: api::UnvaultTransaction,
        signed: bool,
        view: view::SignUnvaultTxView,
    },
    SignRevocationTxs {
        target: api::RevocationTransactions,
        signed: bool,
        view: view::SignRevocationTxsView,
    },
    SecureBatch {
        target: api::SecureBatch,
        signed: bool,
        view: view::SecureBatchView,
    },
    DelegateBatch {
        target: api::DelegateBatch,
        signed: bool,
        view: view::DelegateBatchView,
    },
}

impl Method {
    pub fn new(request: api::Request) -> Method {
        match request {
            api::Request::SpendTransaction(target) => Method::SignSpendTx {
                target,
                signed: false,
                view: view::SignSpendTxView::new(),
            },
            api::Request::UnvaultTransaction(target) => Method::SignUnvaultTx {
                target,
                signed: false,
                view: view::SignUnvaultTxView::new(),
            },
            api::Request::RevocationTransactions(target) => Method::SignRevocationTxs {
                target,
                signed: false,
                view: view::SignRevocationTxsView::new(),
            },
            api::Request::SecureBatch(target) => Method::SecureBatch {
                target,
                signed: false,
                view: view::SecureBatchView::new(),
            },
            api::Request::DelegateBatch(target) => Method::DelegateBatch {
                target,
                signed: false,
                view: view::DelegateBatchView::new(),
            },
        }
    }
    pub fn render(&mut self) -> Element<view::ViewMessage> {
        match self {
            Self::SignSpendTx {
                view,
                target,
                signed,
            } => view.render(&target, *signed),
            Self::SignUnvaultTx {
                view,
                target,
                signed,
            } => view.render(&target, *signed),
            Self::SignRevocationTxs {
                view,
                target,
                signed,
            } => view.render(&target, *signed),
            Self::SecureBatch {
                view,
                target,
                signed,
            } => view.render(
                target
                    .deposits
                    .iter()
                    .map(|deposit| deposit.amount.as_sat())
                    .sum::<u64>(),
                target.deposits.len(),
                *signed,
            ),
            Self::DelegateBatch {
                view,
                target,
                signed,
            } => view.render(
                target
                    .vaults
                    .iter()
                    .map(|vault| vault.amount.as_sat())
                    .sum::<u64>(),
                target.vaults.len(),
                *signed,
            ),
        }
    }
}
