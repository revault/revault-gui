use std::net::SocketAddr;

use iced::{executor, Application, Clipboard, Command, Element, Settings, Subscription};
use iced_native::{window, Event};
use revault_tx::bitcoin::util::bip32::ExtendedPrivKey;
use serde_json::json;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    api,
    config::{self, Config},
    server, sign, view,
};

pub fn run(cfg: Config) -> iced::Result {
    let mut settings = Settings::with_flags(cfg);
    settings.exit_on_close_request = false;
    App::run(settings)
}

pub struct App {
    exit: bool,
    keys: Vec<config::Key>,
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
    CtrlC,
    Event(Event),
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
                exit: false,
                signer: sign::Signer::new(
                    cfg.descriptors.map(|d| sign::Descriptors {
                        deposit_descriptor: d.deposit_descriptor,
                        unvault_descriptor: d.unvault_descriptor,
                        cpfp_descriptor: d.cpfp_descriptor,
                    }),
                    cfg.emergency_address,
                ),
                keys: cfg.keys,
                status: AppStatus::Waiting,
            },
            Command::perform(ctrl_c(), |_| Message::CtrlC),
        )
    }

    fn title(&self) -> String {
        String::from("Dummy signer - Revault")
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::CtrlC | Message::Event(Event::Window(window::Event::CloseRequested)) => {
                self.exit = true;
                self.status = AppStatus::Waiting {};
                Command::none()
            }
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
                            *method = Some(Method::new(&self.keys, &self.signer, req));
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
            Message::Server(server::ServerMessage::ConnectionDropped)
            | Message::Server(server::ServerMessage::Stopped) => {
                self.status = AppStatus::Waiting {};
                Command::none()
            }
            Message::View(view::ViewMessage::Key(i, view::KeyMessage::Selected(selected))) => {
                if let AppStatus::Connected { method, .. } = &mut self.status {
                    match method {
                        Some(Method::SignSpendTx { keys, .. }) => keys[i].selected = selected,
                        Some(Method::SignUnvaultTx { keys, .. }) => keys[i].selected = selected,
                        Some(Method::SignRevocationTxs { keys, .. }) => keys[i].selected = selected,
                        Some(Method::SecureBatch { keys, .. }) => keys[i].selected = selected,
                        Some(Method::DelegateBatch { keys, .. }) => keys[i].selected = selected,
                        _ => {}
                    }
                }
                Command::none()
            }
            Message::View(view::ViewMessage::Confirm) => {
                if let AppStatus::Connected { method, writer, .. } = &mut self.status {
                    match method {
                        Some(Method::SignUnvaultTx {
                            target,
                            signed,
                            keys,
                            ..
                        }) => {
                            let selected_keys = keys
                                .iter()
                                .filter_map(|k| if k.selected { Some(k.xpriv) } else { None })
                                .collect();
                            self.signer
                                .sign_psbt(&selected_keys, &mut target.unvault_tx)
                                .unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignSpendTx {
                            target,
                            signed,
                            keys,
                            ..
                        }) => {
                            let selected_keys = keys
                                .iter()
                                .filter_map(|k| if k.selected { Some(k.xpriv) } else { None })
                                .collect();
                            self.signer
                                .sign_psbt(&selected_keys, &mut target.spend_tx)
                                .unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SignRevocationTxs {
                            target,
                            signed,
                            keys,
                            ..
                        }) => {
                            let selected_keys = keys
                                .iter()
                                .filter_map(|k| if k.selected { Some(k.xpriv) } else { None })
                                .collect();
                            self.signer
                                .sign_psbt(&selected_keys, &mut target.emergency_tx)
                                .unwrap();
                            self.signer
                                .sign_psbt(&selected_keys, &mut target.emergency_unvault_tx)
                                .unwrap();
                            self.signer
                                .sign_psbt(&selected_keys, &mut target.cancel_tx)
                                .unwrap();
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!(target)),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::SecureBatch {
                            target,
                            signed,
                            keys,
                            ..
                        }) => {
                            let selected_keys = keys
                                .iter()
                                .filter_map(|k| if k.selected { Some(k.xpriv) } else { None })
                                .collect();
                            for revocation_txs in target.iter_mut() {
                                self.signer
                                    .sign_psbt(&selected_keys, &mut revocation_txs.emergency_tx)
                                    .unwrap();

                                self.signer
                                    .sign_psbt(&selected_keys, &mut revocation_txs.cancel_tx)
                                    .unwrap();

                                self.signer
                                    .sign_psbt(
                                        &selected_keys,
                                        &mut revocation_txs.emergency_unvault_tx,
                                    )
                                    .unwrap();
                            }
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!({ "transactions": target })),
                                server::ServerMessage::Responded,
                            )
                            .map(Message::Server);
                        }
                        Some(Method::DelegateBatch {
                            target,
                            signed,
                            keys,
                            ..
                        }) => {
                            let selected_keys = keys
                                .iter()
                                .filter_map(|k| if k.selected { Some(k.xpriv) } else { None })
                                .collect();
                            for tx in target.iter_mut() {
                                self.signer
                                    .sign_psbt(&selected_keys, &mut tx.unvault_tx)
                                    .unwrap();
                            }
                            *signed = true;
                            return Command::perform(
                                server::respond(writer.clone(), json!({ "transactions": target })),
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
        if !self.exit {
            Subscription::batch(vec![
                iced_native::subscription::events().map(Message::Event),
                server::listen("0.0.0.0:8080").map(Message::Server),
            ])
        } else {
            Subscription::none()
        }
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

async fn ctrl_c() -> Result<(), ()> {
    tokio::signal::ctrl_c().await.unwrap();
    Ok(())
}

pub struct Key {
    name: String,
    xpriv: ExtendedPrivKey,
    selected: bool,
}

impl Key {
    pub fn new(name: String, xpriv: ExtendedPrivKey) -> Self {
        Key {
            name,
            xpriv,
            selected: false,
        }
    }

    pub fn render(&self) -> Element<view::KeyMessage> {
        if self.name != "" {
            view::key_view(&self.name, self.selected)
        } else {
            view::key_view(&self.xpriv.to_string(), self.selected)
        }
    }
}

pub enum Method {
    SignSpendTx {
        keys: Vec<Key>,
        target: api::SpendTransaction,
        signed: bool,
        view: view::SignSpendTxView,
    },
    SignUnvaultTx {
        keys: Vec<Key>,
        target: api::UnvaultTransaction,
        signed: bool,
        view: view::SignUnvaultTxView,
    },
    SignRevocationTxs {
        keys: Vec<Key>,
        target: api::RevocationTransactions,
        signed: bool,
        view: view::SignRevocationTxsView,
    },
    SecureBatch {
        keys: Vec<Key>,
        target: Vec<api::RevocationTransactions>,
        total_amount: u64,
        signed: bool,
        view: view::SecureBatchView,
    },
    DelegateBatch {
        keys: Vec<Key>,
        target: Vec<api::UnvaultTransaction>,
        total_amount: u64,
        signed: bool,
        view: view::DelegateBatchView,
    },
}

impl Method {
    pub fn new(
        config_keys: &Vec<config::Key>,
        signer: &sign::Signer,
        request: api::Request,
    ) -> Method {
        match request {
            api::Request::SpendTransaction(target) => {
                let mut keys: Vec<Key> = config_keys
                    .iter()
                    .filter_map(|key| {
                        if signer.requires_key_for_psbt(&key.xpriv, &target.spend_tx) {
                            Some(Key::new(key.name.clone(), key.xpriv))
                        } else {
                            None
                        }
                    })
                    .collect();
                // if there is only one key, then it is automatically selected
                if keys.len() == 1 {
                    keys[0].selected = true;
                }

                Method::SignSpendTx {
                    keys,
                    target,
                    signed: false,
                    view: view::SignSpendTxView::new(),
                }
            }
            api::Request::UnvaultTransaction(target) => {
                let mut keys: Vec<Key> = config_keys
                    .iter()
                    .filter_map(|key| {
                        if signer.requires_key_for_psbt(&key.xpriv, &target.unvault_tx) {
                            Some(Key::new(key.name.clone(), key.xpriv))
                        } else {
                            None
                        }
                    })
                    .collect();

                // if there is only one key, then it is automatically selected
                if keys.len() == 1 {
                    keys[0].selected = true;
                }

                Method::SignUnvaultTx {
                    keys,
                    target,
                    signed: false,
                    view: view::SignUnvaultTxView::new(),
                }
            }
            api::Request::RevocationTransactions(target) => {
                let mut keys: Vec<Key> = config_keys
                    .iter()
                    .filter_map(|key| {
                        if signer.requires_key_for_psbt(&key.xpriv, &target.emergency_tx) {
                            Some(Key::new(key.name.clone(), key.xpriv))
                        } else {
                            None
                        }
                    })
                    .collect();

                // if there is only one key, then it is automatically selected
                if keys.len() == 1 {
                    keys[0].selected = true;
                }

                Method::SignRevocationTxs {
                    keys,
                    target,
                    signed: false,
                    view: view::SignRevocationTxsView::new(),
                }
            }
            api::Request::SecureBatch(target) => {
                let total_amount = target
                    .deposits
                    .iter()
                    .map(|deposit| deposit.amount.as_sat())
                    .sum::<u64>();

                let target: Vec<api::RevocationTransactions> = target
                    .deposits
                    .into_iter()
                    .map(|deposit| {
                        let txs = signer
                            .derive_revocation_txs(
                                deposit.outpoint,
                                deposit.amount,
                                deposit.derivation_index,
                            )
                            .unwrap();
                        api::RevocationTransactions {
                            cancel_tx: txs.cancel_tx,
                            emergency_unvault_tx: txs.emergency_unvault_tx,
                            emergency_tx: txs.emergency_tx,
                        }
                    })
                    .collect();

                let mut keys: Vec<Key> = config_keys
                    .iter()
                    .filter_map(|key| {
                        if signer.requires_key_for_psbt(&key.xpriv, &target[0].emergency_tx) {
                            Some(Key::new(key.name.clone(), key.xpriv))
                        } else {
                            None
                        }
                    })
                    .collect();

                // if there is only one key, then it is automatically selected
                if keys.len() == 1 {
                    keys[0].selected = true;
                }

                Method::SecureBatch {
                    total_amount,
                    keys,
                    target,
                    signed: false,
                    view: view::SecureBatchView::new(),
                }
            }
            api::Request::DelegateBatch(target) => {
                let total_amount = target
                    .vaults
                    .iter()
                    .map(|vault| vault.amount.as_sat())
                    .sum::<u64>();

                let target: Vec<api::UnvaultTransaction> = target
                    .vaults
                    .into_iter()
                    .map(|deposit| {
                        let tx = signer
                            .derive_unvault_tx(
                                deposit.outpoint,
                                deposit.amount,
                                deposit.derivation_index,
                            )
                            .unwrap();
                        api::UnvaultTransaction { unvault_tx: tx }
                    })
                    .collect();

                let mut keys: Vec<Key> = config_keys
                    .iter()
                    .filter_map(|key| {
                        if signer.requires_key_for_psbt(&key.xpriv, &target[0].unvault_tx) {
                            Some(Key::new(key.name.clone(), key.xpriv))
                        } else {
                            None
                        }
                    })
                    .collect();

                // if there is only one key, then it is automatically selected
                if keys.len() == 1 {
                    keys[0].selected = true;
                }

                Method::DelegateBatch {
                    total_amount,
                    keys,
                    target,
                    signed: false,
                    view: view::DelegateBatchView::new(),
                }
            }
        }
    }

    pub fn render(&mut self) -> Element<view::ViewMessage> {
        match self {
            Self::SignSpendTx {
                view,
                target,
                signed,
                keys,
            } => view.render(
                &target,
                *signed,
                keys.iter()
                    .enumerate()
                    .map(|(i, key)| key.render().map(move |msg| view::ViewMessage::Key(i, msg)))
                    .collect(),
                keys.iter().any(|key| key.selected),
            ),
            Self::SignUnvaultTx {
                view,
                target,
                signed,
                keys,
            } => view.render(
                &target,
                *signed,
                keys.iter()
                    .enumerate()
                    .map(|(i, key)| key.render().map(move |msg| view::ViewMessage::Key(i, msg)))
                    .collect(),
                keys.iter().any(|key| key.selected),
            ),
            Self::SignRevocationTxs {
                view,
                target,
                signed,
                keys,
            } => view.render(
                &target,
                *signed,
                keys.iter()
                    .enumerate()
                    .map(|(i, key)| key.render().map(move |msg| view::ViewMessage::Key(i, msg)))
                    .collect(),
                keys.iter().any(|key| key.selected),
            ),
            Self::SecureBatch {
                view,
                total_amount,
                target,
                signed,
                keys,
            } => view.render(
                *total_amount,
                target.len(),
                *signed,
                keys.iter()
                    .enumerate()
                    .map(|(i, key)| key.render().map(move |msg| view::ViewMessage::Key(i, msg)))
                    .collect(),
                keys.iter().any(|key| key.selected),
            ),
            Self::DelegateBatch {
                total_amount,
                view,
                target,
                signed,
                keys,
            } => view.render(
                *total_amount,
                target.len(),
                *signed,
                keys.iter()
                    .enumerate()
                    .map(|(i, key)| key.render().map(move |msg| view::ViewMessage::Key(i, msg)))
                    .collect(),
                keys.iter().any(|key| key.selected),
            ),
        }
    }
}
