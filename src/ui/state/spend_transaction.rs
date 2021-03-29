use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{list_spend_txs, list_vaults},
    State,
};

use crate::revaultd::{model, RevaultD};

use crate::ui::{
    error::Error,
    message::{Message, SpendTxMessage},
    view::Context,
    view::{SpendTransactionListItemView, SpendTransactionView},
};

#[derive(Debug)]
pub struct SpendTransactionState {
    pub psbt: Psbt,

    revaultd: Arc<RevaultD>,
    deposit_outpoints: Vec<String>,
    deposits: Vec<model::Vault>,
    warning: Option<Error>,

    view: SpendTransactionView,
}

impl SpendTransactionState {
    pub fn new(revaultd: Arc<RevaultD>, psbt: Psbt) -> Self {
        Self {
            revaultd,
            psbt,
            deposit_outpoints: Vec::new(),
            deposits: Vec::new(),
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
            _ => {}
        };
        Command::none()
    }
    fn view(&mut self, ctx: &Context) -> Element<Message> {
        self.view
            .view(ctx, &self.psbt, &self.deposits, self.warning.as_ref())
    }

    fn load(&self) -> Command<Message> {
        Command::perform(list_spend_txs(self.revaultd.clone()), |res| {
            Message::SpendTx(SpendTxMessage::SpendTransactions(res))
        })
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
