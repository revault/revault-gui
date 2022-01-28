use bitcoin::util::{
    bip32::{ExtendedPubKey, Fingerprint},
    psbt::PartiallySignedTransaction as Psbt,
};
use std::convert::From;
use std::str::FromStr;

use bitcoin::OutPoint;
use iced::{Command, Element, Subscription};
use revault_ui::component::form;
use revaultd::revault_tx::miniscript::DescriptorPublicKey;

use crate::{
    app::{
        context::Context,
        error::Error,
        message::{Message, SpendTxMessage},
        state::{
            cmd::{
                broadcast_spend_tx, delete_spend_tx, list_spend_txs, list_vaults, update_spend_tx,
            },
            sign::{Signer, SpendTransactionTarget},
            State,
        },
        view::spend_transaction::{
            SpendTransactionBroadcastView, SpendTransactionDeleteView,
            SpendTransactionListItemView, SpendTransactionSharePsbtView, SpendTransactionSignView,
            SpendTransactionView,
        },
    },
    daemon::model,
};

#[derive(Debug)]
pub struct SpendTransactionState {
    pub psbt: Psbt,
    cpfp_index: usize,
    change_index: Option<usize>,

    deposit_outpoints: Vec<OutPoint>,
    deposits: Vec<model::Vault>,
    warning: Option<Error>,

    action: SpendTransactionAction,

    view: SpendTransactionView,
}

impl SpendTransactionState {
    pub fn new(ctx: &Context, psbt: Psbt) -> Self {
        Self {
            cpfp_index: 0,
            change_index: None,
            action: SpendTransactionAction::new(
                ctx.managers_threshold,
                &ctx.config
                    .daemon
                    .manager_config
                    .as_ref()
                    .expect("User is a manager")
                    .xpub,
                &ctx.managers_xpubs(),
                &psbt,
            ),
            psbt,
            deposit_outpoints: Vec::new(),
            deposits: Vec::new(),
            warning: None,
            view: SpendTransactionView::new(),
        }
    }

    // TODO: remove it for subscription
    pub fn sub(&self) -> Subscription<Message> {
        if let SpendTransactionAction::Sign { signer, .. } = &self.action {
            signer
                .subscription()
                .map(|msg| Message::SpendTx(SpendTxMessage::Sign(msg)))
        } else {
            Subscription::none()
        }
    }
}

impl State for SpendTransactionState {
    fn update(&mut self, ctx: &Context, message: Message) -> Command<Message> {
        match message {
            Message::SpendTx(SpendTxMessage::Inputs(res)) => match res {
                Ok(vaults) => {
                    self.deposits = Vec::new();
                    // we keep the order of the deposit_outpoints.
                    for deposit in vaults.into_iter() {
                        for outpoint in &self.deposit_outpoints {
                            if deposit.outpoint() == *outpoint {
                                self.deposits.push(deposit.clone());
                            }
                        }
                    }
                }
                Err(e) => self.warning = Error::from(e).into(),
            },
            Message::SpendTx(SpendTxMessage::SpendTransactions(res)) => match res {
                Ok(txs) => {
                    for tx in txs {
                        if tx.psbt.global.unsigned_tx.txid() == self.psbt.global.unsigned_tx.txid()
                        {
                            self.deposit_outpoints = tx
                                .deposit_outpoints
                                .iter()
                                .map(|s| OutPoint::from_str(s).unwrap())
                                .collect();
                            self.psbt = tx.psbt;
                            self.cpfp_index = tx.cpfp_index;
                            self.change_index = tx.change_index;
                            return Command::perform(
                                list_vaults(
                                    ctx.revaultd.clone(),
                                    None,
                                    Some(self.deposit_outpoints.clone()),
                                ),
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
                    .update(ctx, &mut self.psbt, msg)
                    .map(Message::SpendTx);
            }
            _ => {}
        };
        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.sub()
    }

    fn view(&mut self, ctx: &Context) -> Element<Message> {
        let show_delete_button = !matches!(self.action, SpendTransactionAction::Delete { .. });
        self.view.view(
            ctx,
            &self.psbt,
            self.cpfp_index,
            self.change_index,
            &self.deposits,
            self.action.view(ctx, &self.psbt),
            self.warning.as_ref(),
            show_delete_button,
        )
    }

    fn load(&self, ctx: &Context) -> Command<Message> {
        Command::perform(list_spend_txs(ctx.revaultd.clone(), None), |res| {
            Message::SpendTx(SpendTxMessage::SpendTransactions(res))
        })
    }
}

#[derive(Debug)]
pub enum SpendTransactionAction {
    SharePsbt {
        psbt_input: form::Value<String>,
        processing: bool,
        success: bool,
        warning: Option<Error>,
        view: SpendTransactionSharePsbtView,
    },
    Sign {
        warning: Option<Error>,
        processing: bool,
        signer: Signer<SpendTransactionTarget>,
        view: SpendTransactionSignView,
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
    fn new(
        managers_threshold: usize,
        user_manager_xpub: &ExtendedPubKey,
        managers_xpubs: &Vec<DescriptorPublicKey>,
        psbt: &Psbt,
    ) -> Self {
        if let Some(input) = psbt.inputs.first() {
            if input.partial_sigs.len() >= managers_threshold {
                return Self::Broadcast {
                    processing: false,
                    success: false,
                    warning: None,
                    view: SpendTransactionBroadcastView::new(),
                };
            } else if input.partial_sigs.keys().any(|key| {
                input
                    .bip32_derivation
                    .get(key)
                    .map(|(fingerprint, _)| user_manager_xpub.fingerprint() == *fingerprint)
                    .unwrap_or(false)
            }) {
                return Self::SharePsbt {
                    psbt_input: form::Value::default(),
                    processing: false,
                    success: false,
                    warning: None,
                    view: SpendTransactionSharePsbtView::new(),
                };
            } else {
                return Self::Sign {
                    processing: false,
                    warning: None,
                    signer: Signer::new(SpendTransactionTarget::new(
                        &managers_xpubs
                            .iter()
                            .map(|xpub| xpub.master_fingerprint())
                            .collect(),
                        psbt.clone(),
                    )),
                    view: SpendTransactionSignView::new(),
                };
            }
        }
        Self::SharePsbt {
            psbt_input: form::Value::default(),
            processing: false,
            success: false,
            warning: None,
            view: SpendTransactionSharePsbtView::new(),
        }
    }
    fn update(
        &mut self,
        ctx: &Context,
        psbt: &mut Psbt,
        message: SpendTxMessage,
    ) -> Command<SpendTxMessage> {
        match message {
            SpendTxMessage::Delete => {
                if let Self::Delete { processing, .. } = self {
                    *processing = true;
                    return Command::perform(
                        delete_spend_tx(
                            ctx.revaultd.clone(),
                            psbt.global.unsigned_tx.txid().to_string(),
                        ),
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
            SpendTxMessage::SelectDelete => {
                *self = Self::Delete {
                    processing: false,
                    success: false,
                    warning: None,
                    view: SpendTransactionDeleteView::new(),
                };
            }
            SpendTxMessage::UnselectDelete => {
                *self = Self::new(
                    ctx.managers_threshold,
                    &ctx.config
                        .daemon
                        .manager_config
                        .as_ref()
                        .expect("User is a manager")
                        .xpub,
                    &ctx.managers_xpubs(),
                    psbt,
                );
            }
            SpendTxMessage::Sign(msg) => {
                if let Self::Sign {
                    signer, processing, ..
                } = self
                {
                    let cmd = signer.update(ctx, msg);
                    if signer.signed() && !*processing {
                        *psbt = signer.target.spend_tx.clone();
                        *processing = true;
                        return Command::perform(
                            update_spend_tx(ctx.revaultd.clone(), signer.target.spend_tx.clone()),
                            SpendTxMessage::Signed,
                        );
                    }
                    return cmd.map(SpendTxMessage::Sign);
                }
            }
            SpendTxMessage::Signed(res) => {
                if let Self::Sign {
                    warning,
                    signer,
                    processing,
                    ..
                } = self
                {
                    *processing = false;
                    match res {
                        Ok(_) => {
                            // During this step state has a generated psbt
                            // and signer has a signed psbt.
                            *psbt = signer.target.spend_tx.clone();
                            *self = Self::new(
                                ctx.managers_threshold,
                                &ctx.config
                                    .daemon
                                    .manager_config
                                    .as_ref()
                                    .expect("User is a manager")
                                    .xpub,
                                &ctx.managers_xpubs(),
                                psbt,
                            );
                        }

                        Err(e) => *warning = Some(Error::RevaultDError(e)),
                    }
                }
            }
            SpendTxMessage::Broadcast => {
                if let Self::Broadcast { processing, .. } = self {
                    *processing = true;
                    return Command::perform(
                        broadcast_spend_tx(
                            ctx.revaultd.clone(),
                            psbt.global.unsigned_tx.txid().to_string(),
                        ),
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
                        psbt_input.value = input;
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
                    let p: Option<Psbt> = bitcoin::base64::decode(&psbt_input.value)
                        .ok()
                        .and_then(|bytes| bitcoin::consensus::encode::deserialize(&bytes).ok());
                    if let Some(p) = p {
                        if p.global.unsigned_tx.txid() != psbt.global.unsigned_tx.txid() {
                            psbt_input.valid = false;
                        } else if is_unknown_sig(
                            &ctx.managers_xpubs()
                                .into_iter()
                                .map(|xpub| xpub.master_fingerprint())
                                .collect(),
                            &p,
                        ) {
                            psbt_input.valid = false;
                        } else {
                            *processing = true;
                            *warning = None;
                            psbt_input.valid = true;
                            return Command::perform(
                                update_spend_tx(ctx.revaultd.clone(), p),
                                SpendTxMessage::Updated,
                            );
                        }
                    } else {
                        psbt_input.valid = false;
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
                            *processing = false;
                            *psbt = bitcoin::consensus::encode::deserialize(
                                &bitcoin::base64::decode(&psbt_input.value)
                                    .expect("psbt was successfully updated with the given input"),
                            )
                            .expect("psbt was successfully updated with the given input");
                            if let Some(input) = psbt.inputs.first() {
                                if input.partial_sigs.len() == ctx.managers_threshold {
                                    *self = Self::Broadcast {
                                        processing: false,
                                        success: false,
                                        warning: None,
                                        view: SpendTransactionBroadcastView::new(),
                                    };
                                } else if !input.partial_sigs.keys().any(|key| {
                                    input
                                        .bip32_derivation
                                        .get(key)
                                        .map(|(fingerprint, _)| {
                                            ctx.config
                                                .daemon
                                                .manager_config
                                                .as_ref()
                                                .expect("User is a manager")
                                                .xpub
                                                .fingerprint()
                                                == *fingerprint
                                        })
                                        .unwrap_or(false)
                                }) {
                                    *self = Self::Sign {
                                        processing: false,
                                        warning: None,
                                        signer: Signer::new(SpendTransactionTarget::new(
                                            &ctx.managers_xpubs()
                                                .iter()
                                                .map(|xpub| xpub.master_fingerprint())
                                                .collect(),
                                            psbt.clone(),
                                        )),
                                        view: SpendTransactionSignView::new(),
                                    };
                                }
                            }
                        }
                        Err(e) => {
                            *processing = false;
                            *warning = Error::from(e).into();
                        }
                    };
                }
            }
            _ => {}
        }
        Command::none()
    }

    fn view(&mut self, ctx: &Context, psbt: &Psbt) -> Element<Message> {
        match self {
            Self::Sign {
                signer,
                warning,
                view,
                ..
            } => view.view(
                signer
                    .view(ctx)
                    .map(|msg| Message::SpendTx(SpendTxMessage::Sign(msg))),
                warning.as_ref(),
            ),
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

/// Returns true if the psbt has a signature from a key with a master fingerprint
/// not contained in the given list of fingerprints
pub fn is_unknown_sig(fingerprints: &Vec<Fingerprint>, psbt: &Psbt) -> bool {
    psbt.inputs.iter().any(|input| {
        input.partial_sigs.keys().any(|key| {
            if let Some((fingerprint, _)) = input.bip32_derivation.get(key) {
                !fingerprints.contains(fingerprint)
            } else {
                true
            }
        })
    })
}

#[derive(Debug)]
pub struct SpendTransactionListItem {
    pub tx: model::SpendTx,

    spend_amount: u64,
    fees: u64,

    view: SpendTransactionListItemView,
}

impl SpendTransactionListItem {
    pub fn new(tx: model::SpendTx, vaults_amount: u64) -> Self {
        let spend_amount = tx
            .psbt
            .global
            .unsigned_tx
            .output
            .iter()
            .enumerate()
            .filter(|(i, _)| Some(i) != tx.change_index.as_ref() && i != &tx.cpfp_index)
            .fold(0, |acc, (_, output)| acc + output.value);
        let change_amount = if let Some(i) = tx.change_index {
            tx.psbt.global.unsigned_tx.output[i].value
        } else {
            0
        };

        let fees = if vaults_amount == 0 {
            // Vaults are still loading
            0
        } else {
            vaults_amount - spend_amount - change_amount
        };
        Self {
            tx,
            spend_amount,
            fees,
            view: SpendTransactionListItemView::new(),
        }
    }

    pub fn view(&mut self, ctx: &Context) -> Element<SpendTxMessage> {
        self.view.view(ctx, &self.tx, self.spend_amount, self.fees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::util::{
        bip32::ExtendedPubKey, ecdsa::PublicKey, psbt::PartiallySignedTransaction as Psbt,
    };
    use std::str::FromStr;

    #[test]
    fn test_new_spend_transaction_action() {
        let mut psbt = Psbt::from_str("cHNidP8BALQCAAAAAc1946BSKWX5trghNlBq/IIYScLPYqr9Bqs2LfqOYuqcAAAAAAAIAAAAA+BAAAAAAAAAIgAgCOQxrx6W/t0dSZikMBNYG2Yyam/3LIoVrAy6e8ZDUAyA8PoCAAAAACIAIMuwqNTx88KHHtIR0EeURzEu9pUmbnUxd22KzYKi25A2CBH6AgAAAAAiACB18mkXdMgWd4MYRrAoIgDiiLLFlxC1j3Qxg9SSVQfbxQAAAAAAAQEruFn1BQAAAAAiACBI6M9l6zams92tyCK/4gbWyNfJMJzgoOv34L0X7GTovAEDBAEAAAABBWEhAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmrFGHZHapFHKpXyKvmhuuuFL5qVJy+MIdmPJkiKxrdqkUtsmtuJyMk3Jsg+KhtdlHidd7lWGIrGyTUodnWLJoIgYCApM6sQN+rQqkp4WMbUnWcF4fs7xfZrAJGK97nwDs1SYIJR1gCQAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAA").unwrap();
        let user_manager_xpub = ExtendedPubKey::from_str("xpub6CZFHPW1GiB8YgV7zGpeQDB6mMHZYPQyUaHrM1nMvKMgLxwok4xCtnzjuxQ3p1LHJUkz5i1Y7bRy5fmGrdg8UBVb39XdXNtWWd2wTsNd7T9").unwrap();

        let action = SpendTransactionAction::new(2, &user_manager_xpub, &Vec::new(), &psbt);
        assert!(matches!(action, SpendTransactionAction::Sign { .. }));

        psbt.inputs[0].partial_sigs.insert(
            PublicKey::from_str(
                "0202933ab1037ead0aa4a7858c6d49d6705e1fb3bc5f66b00918af7b9f00ecd526",
            )
            .unwrap(),
            "304402202f5eec50f34929e4bd8f6b7e81426795b0cd3608a4dad53ffab3e7af38ab627a02204ff61d9df2432ff3272c17d9baee1ec6b6dcb72b198be7f4ef843d5d47010a0401".as_bytes().to_vec(),
        );

        let action = SpendTransactionAction::new(2, &user_manager_xpub, &Vec::new(), &psbt);
        assert!(matches!(action, SpendTransactionAction::SharePsbt { .. }));

        let action = SpendTransactionAction::new(1, &user_manager_xpub, &Vec::new(), &psbt);
        assert!(matches!(action, SpendTransactionAction::Broadcast { .. }));

        let action = SpendTransactionAction::new(0, &user_manager_xpub, &Vec::new(), &psbt);
        assert!(matches!(action, SpendTransactionAction::Broadcast { .. }));
    }
}
