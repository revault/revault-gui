use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction as Psbt};
use serde::Deserialize;

use iced::{Command, Element};

use crate::{
    revault::TransactionKind,
    ui::{
        message::SignMessage,
        view::{
            sign::{DirectSignatureView, IndirectSignatureView},
            Context,
        },
    },
};

/// SignState is a general widget to handle the signature of a Psbt.
#[derive(Debug)]
pub struct SignState {
    pub original_psbt: Psbt,
    pub signed_psbt: Option<Psbt>,
    pub transaction_kind: TransactionKind,
    method: SignMethod,
}

/// SignMethod is the way the user will sign the PSBT.
#[derive(Debug)]
pub enum SignMethod {
    /// DirectSignature means that a hard module directly
    /// connect to the GUI and signs the given PSBT.
    DirectSignature { view: DirectSignatureView },
    /// IndirectSignature means that the PSBT is exported and
    /// then imported once signed on a air gapped device for example.
    IndirectSignature {
        processing: bool,
        warning: Option<String>,
        psbt_input: String,
        view: IndirectSignatureView,
    },
}

impl SignState {
    pub fn new(original_psbt: Psbt, transaction_kind: TransactionKind) -> Self {
        SignState {
            original_psbt,
            transaction_kind,
            signed_psbt: None,
            method: SignMethod::DirectSignature {
                view: DirectSignatureView::new(),
            },
        }
    }

    pub fn update(&mut self, message: SignMessage) -> Command<SignMessage> {
        match message {
            SignMessage::PsbtEdited(psbt) => {
                if let SignMethod::IndirectSignature {
                    psbt_input,
                    warning,
                    ..
                } = &mut self.method
                {
                    *warning = None;
                    *psbt_input = psbt;
                }
            }
            SignMessage::Sign => {
                if let SignMethod::IndirectSignature {
                    psbt_input,
                    warning,
                    processing,
                    ..
                } = &mut self.method
                {
                    if !psbt_input.is_empty() {
                        self.signed_psbt = base64::decode(&psbt_input)
                            .ok()
                            .and_then(|bytes| encode::deserialize(&bytes).ok());
                        if self.signed_psbt.is_some() {
                            *processing = true;
                        } else {
                            *warning = Some("Please enter valid PSBT".to_string());
                        }
                    }
                }
            }
            SignMessage::ChangeMethod => {
                if let SignMethod::DirectSignature { .. } = self.method {
                    self.method = SignMethod::IndirectSignature {
                        processing: false,
                        warning: None,
                        psbt_input: "".to_string(),
                        view: IndirectSignatureView::new(),
                    }
                } else {
                    self.method = SignMethod::DirectSignature {
                        view: DirectSignatureView::new(),
                    }
                }
            }
            _ => {}
        };
        Command::none()
    }

    pub fn view(&mut self, ctx: &Context) -> Element<SignMessage> {
        match &mut self.method {
            SignMethod::DirectSignature { view } => view.view(ctx, &self.transaction_kind),
            SignMethod::IndirectSignature {
                processing,
                psbt_input,
                view,
                warning,
            } => view.view(
                ctx,
                &processing,
                &self.transaction_kind,
                &self.original_psbt,
                &psbt_input,
                warning.as_ref(),
            ),
        }
    }
}
