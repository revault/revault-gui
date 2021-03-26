use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;
use std::convert::From;
use std::sync::Arc;

use iced::{Command, Element};

use super::{
    cmd::{get_blockheight, get_spend_tx, list_spend_txs, list_vaults, update_spend_tx},
    vault::{Vault, VaultListItem},
    State,
};

use crate::revaultd::{
    model::{self, VaultStatus},
    RevaultD,
};

use crate::revault::TransactionKind;

use crate::ui::{
    error::Error,
    message::{InputMessage, Message, RecipientMessage, SignMessage, SpendTxMessage, VaultMessage},
    state::sign::SignState,
    view::Context,
    view::SpendTransactionView,
};

#[derive(Debug)]
pub struct SpendTransactionState {
    revaultd: Arc<RevaultD>,
    psbt: Psbt,
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
            Message::Vaults(res) => match res {
                Ok(vaults) => {
                    self.deposits = vaults
                        .into_iter()
                        .filter(|vault| self.deposit_outpoints.contains(&vault.outpoint()))
                        .collect();
                }
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SpendTransactions(res) => match res {
                Ok(txs) => {
                    for tx in txs {
                        if tx.psbt.global.unsigned_tx.txid() == self.psbt.global.unsigned_tx.txid()
                        {
                            self.deposit_outpoints = tx.deposit_outpoints;
                            self.psbt = tx.psbt;
                            return Command::perform(
                                list_vaults(self.revaultd.clone(), None),
                                Message::Vaults,
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
        Command::perform(
            list_spend_txs(self.revaultd.clone()),
            Message::SpendTransactions,
        )
    }
}
