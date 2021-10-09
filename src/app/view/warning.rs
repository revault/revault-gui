use std::convert::From;

use iced::{Column, Container, Length};

use revault_ui::component::notification::warning;

use crate::{
    app::error::Error,
    daemon::{
        client::error::{ApiErrorCode, RpcErrorCode},
        client::RevaultDError,
        config::ConfigError,
    },
};

/// Simple warning message displayed to non technical user.
pub struct WarningMessage(String);

impl From<&Error> for WarningMessage {
    fn from(error: &Error) -> WarningMessage {
        match error {
            Error::ConfigError(e) => match e {
                ConfigError::NotFound => WarningMessage("Configuration file not fund".to_string()),
                ConfigError::ReadingFile(_) => {
                    WarningMessage("Failed to read configuration file".to_string())
                }
                ConfigError::Unexpected(_) => WarningMessage("Unknown error".to_string()),
            },
            Error::RevaultDError(e) => match e {
                RevaultDError::Rpc(code, _) => {
                    if *code == ApiErrorCode::COORDINATOR_SIG_STORE_ERROR as i32 {
                        WarningMessage("Coordinator could not store the signatures".to_string())
                    } else if *code == ApiErrorCode::COORDINATOR_SPEND_STORE_ERROR as i32 {
                        WarningMessage(
                            "Coordinator could not store the spend transaction".to_string(),
                        )
                    } else if *code == ApiErrorCode::TRANSPORT_ERROR as i32 {
                        WarningMessage("Failed to communicate with remote server".to_string())
                    } else if *code == ApiErrorCode::COSIGNER_INSANE_ERROR as i32 {
                        WarningMessage("The cosigner has an anormal behaviour, stop all operations and report to your security team".to_string())
                    } else if *code == ApiErrorCode::COSIGNER_ALREADY_SIGN_ERROR as i32 {
                        WarningMessage("The cosigner already signed the transaction".to_string())
                    } else if *code == RpcErrorCode::JSONRPC2_INVALID_PARAMS as i32 {
                        WarningMessage("Some fields are invalid".to_string())
                    } else {
                        WarningMessage("Internal error".to_string())
                    }
                }
                RevaultDError::Unexpected(_) => WarningMessage("Unknown error".to_string()),
                RevaultDError::NoAnswer | RevaultDError::Transport(..) => {
                    WarningMessage("Communication with Revault daemon failed".to_string())
                }
            },
            Error::UnexpectedError(_) => WarningMessage("Unknown error".to_string()),
        }
    }
}

impl std::fmt::Display for WarningMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn warn<'a, T: 'a>(error: Option<&Error>) -> Container<'a, T> {
    if let Some(w) = error {
        let message: WarningMessage = w.into();
        warning(&message.to_string(), &w.to_string()).width(Length::Fill)
    } else {
        Container::new(Column::new()).width(Length::Fill)
    }
}
