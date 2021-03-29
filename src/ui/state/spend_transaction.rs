use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{broadcast_spend_tx, delete_spend_tx, list_spend_txs, list_vaults, update_spend_tx},
    State,
};

use crate::revaultd::{model, RevaultD};

use crate::ui::{
    error::Error,
    message::{Message, SpendTxMessage},
    view::spend_transaction::{
        SpendTransactionBroadcastView, SpendTransactionDeleteView, SpendTransactionListItemView,
        SpendTransactionSharePsbtView, SpendTransactionView,
    },
    view::Context,
};

#[derive(Debug)]
pub struct SpendTransactionState {
    pub psbt: Psbt,

    revaultd: Arc<RevaultD>,
    deposit_outpoints: Vec<String>,
    deposits: Vec<model::Vault>,
    warning: Option<Error>,

    action: SpendTransactionAction,

    view: SpendTransactionView,
}

impl SpendTransactionState {
    pub fn new(revaultd: Arc<RevaultD>, psbt: Psbt) -> Self {
        Self {
            revaultd,
            psbt,
            deposit_outpoints: Vec::new(),
            deposits: Vec::new(),
            action: SpendTransactionAction::new(),
            warning: None,
            view: SpendTransactionView::new(),
        }
    }
}

impl State for SpendTransactionState {
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Inputs(res)) => match res {
                Ok(vaults) => {
                    self.deposits = vaults
                        .into_iter()
                        .filter(|vault| self.deposit_outpoints.contains(&vault.outpoint()))
                        .collect();
                }
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SpendTx(SpendTxMessage::SpendTransactions(res)) => match res {
                Ok(txs) => {
                    for tx in txs {
                        if tx.psbt.global.unsigned_tx.txid() == self.psbt.global.unsigned_tx.txid()
                        {
                            self.deposit_outpoints = tx.deposit_outpoints;
                            self.psbt = tx.psbt;
                            return Command::perform(
                                list_vaults(self.revaultd.clone(), None),
                                |res| Message::SpendTx(SpendTxMessage::Inputs(res)),
                            );
                        }
                    }
                }
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SpendTx(msg) => {
                return self
                    .action
                    .update(self.revaultd.clone(), &mut self.psbt, msg)
                    .map(Message::SpendTx);
            }
            _ => {}
        };
        Command::none()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view.view(
            ctx,
            &self.psbt,
            &self.deposits,
            self.action.view(ctx, &self.psbt),
            self.warning.as_ref(),
        )
    }

    fn load(&self) -> Command<Message> {
        Command::perform(list_spend_txs(self.revaultd.clone()), |res| {
            Message::SpendTx(SpendTxMessage::SpendTransactions(res))
        })
    }
}

#[derive(Debug)]
pub enum SpendTransactionAction {
    SharePsbt {
        psbt_input: String,
        processing: bool,
        success: bool,
        warning: Option<Error>,
        view: SpendTransactionSharePsbtView,
    },
    Broadcast {
        processing: bool,
        success: bool,
        warning: Option<Error>,
        view: SpendTransactionBroadcastView,
    },
    Delete {
        processing: bool,
        success: bool,
        warning: Option<Error>,
        view: SpendTransactionDeleteView,
    },
}

impl SpendTransactionAction {
    fn new() -> Self {
        Self::SharePsbt {
            psbt_input: "".to_string(),
            processing: false,
            success: false,
            warning: None,
            view: SpendTransactionSharePsbtView::new(),
        }
    }
    fn update(
        &mut self,
        revaultd: Arc<RevaultD>,
        psbt: &mut Psbt,
        message: SpendTxMessage,
    ) -> Command<SpendTxMessage> {
        match message {
            SpendTxMessage::Delete => {
                if let Self::Delete { processing, .. } = self {
                    *processing = true;
                    return Command::perform(
                        delete_spend_tx(revaultd, psbt.global.unsigned_tx.txid().to_string()),
                        SpendTxMessage::Deleted,
                    );
                }
            }
            SpendTxMessage::Deleted(res) => {
                if let Self::Delete {
                    processing,
                    success,
                    warning,
                    ..
                } = self
                {
                    *processing = false;
                    match res {
                        Ok(()) => *success = true,
                        Err(e) => *warning = Error::from(e).into(),
                    };
                }
            }
            SpendTxMessage::SelectShare => {
                *self = Self::new();
            }
            SpendTxMessage::SelectDelete => {
                *self = Self::Delete {
                    processing: false,
                    success: false,
                    warning: None,
                    view: SpendTransactionDeleteView::new(),
                };
            }
            SpendTxMessage::SelectBroadcast => {
                *self = Self::Broadcast {
                    processing: false,
                    success: false,
                    warning: None,
                    view: SpendTransactionBroadcastView::new(),
                };
            }
            SpendTxMessage::Broadcast => {
                if let Self::Broadcast { processing, .. } = self {
                    *processing = true;
                    return Command::perform(
                        broadcast_spend_tx(revaultd, psbt.global.unsigned_tx.txid().to_string()),
                        SpendTxMessage::Broadcasted,
                    );
                }
            }
            SpendTxMessage::Broadcasted(res) => {
                if let Self::Broadcast {
                    processing,
                    success,
                    warning,
                    ..
                } = self
                {
                    *processing = false;
                    match res {
                        Ok(()) => *success = true,
                        Err(e) => *warning = Error::from(e).into(),
                    };
                }
            }
            SpendTxMessage::PsbtEdited(input) => {
                if let Self::SharePsbt {
                    psbt_input,
                    processing,
                    success,
                    ..
                } = self
                {
                    *success = false;
                    if !*processing {
                        *psbt_input = input;
                    }
                }
            }
            SpendTxMessage::Update => {
                if let Self::SharePsbt {
                    psbt_input,
                    processing,
                    warning,
                    ..
                } = self
                {
                    let p: Option<Psbt> = bitcoin::base64::decode(&psbt_input)
                        .ok()
                        .and_then(|bytes| bitcoin::consensus::encode::deserialize(&bytes).ok());
                    if let Some(p) = p {
                        if p.global.unsigned_tx.txid() != psbt.global.unsigned_tx.txid() {
                            *warning =
                                Error::UnexpectedError("psbt has not the same".to_string()).into();
                        } else {
                            *processing = true;
                            return Command::perform(
                                update_spend_tx(revaultd.clone(), p),
                                SpendTxMessage::Updated,
                            );
                        }
                    } else {
                        *warning =
                            Error::UnexpectedError("Please enter a valid psbt".to_string()).into();
                    }
                }
            }
            SpendTxMessage::Updated(res) => {
                if let Self::SharePsbt {
                    psbt_input,
                    processing,
                    success,
                    warning,
                    ..
                } = self
                {
                    match res {
                        Ok(()) => {
                            *success = true;
                            *psbt = bitcoin::consensus::encode::deserialize(
                                &bitcoin::base64::decode(&psbt_input)
                                    .expect("psbt was successfully updated with the given input"),
                            )
                            .expect("psbt was successfully updated with the given input");
                            *psbt_input = "".to_string();
                        }
                        Err(e) => *warning = Error::from(e).into(),
                    };
                    *processing = false;
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context, psbt: &Psbt) -> Element<Message> {
        match self {
            Self::SharePsbt {
                view,
                psbt_input,
                processing,
                success,
                warning,
                ..
            } => view.view(&psbt_input, &processing, &success, psbt, warning.as_ref()),
            Self::Broadcast {
                view,
                processing,
                success,
                warning,
            } => view.view(&processing, &success, warning.as_ref()),
            Self::Delete {
                view,
                processing,
                success,
                warning,
            } => view.view(&processing, &success, warning.as_ref()),
        }
    }
}

#[derive(Debug)]
pub struct SpendTransactionListItem {
    pub tx: model::SpendTx,
    view: SpendTransactionListItemView,
}

impl SpendTransactionListItem {
    pub fn new(tx: model::SpendTx) -> Self {
        Self {
            tx,
            view: SpendTransactionListItemView::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<SpendTxMessage> {
        self.view.view(ctx, &self.tx)
    }
}
