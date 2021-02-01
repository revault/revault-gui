use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

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
    original_psbt: Psbt,
    signed_psbt: Option<Psbt>,
    transaction_kind: TransactionKind,
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
    IndirectSignature { view: IndirectSignatureView },
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
            SignMessage::ChangeMethod => {
                if let SignMethod::DirectSignature { .. } = self.method {
                    self.method = SignMethod::IndirectSignature {
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
            SignMethod::IndirectSignature { view } => {
                view.view(ctx, &self.transaction_kind, &self.original_psbt)
            }
        }
    }
}
