use std::convert::From;

use iced::{Column, Container, Length};

use revault_ui::component::notification::warning;
use revaultd::commands::ErrorCode;

use crate::{
    app::error::Error,
    daemon::{client::error::RpcErrorCode, RevaultDError},
};

/// Simple warning message displayed to non technical user.
pub struct WarningMessage(String);

impl From<&Error> for WarningMessage {
    fn from(error: &Error) -> WarningMessage {
        match error {
            Error::Hardware(e) => match e {
                revault_hwi::HWIError::DeviceDidNotSign => {
                    WarningMessage("Device did not sign with user key".to_string())
                }
                _ => WarningMessage(e.to_string()),
            },
            Error::Config(e) => WarningMessage(e.to_owned()),
            // TODO: change when ConfigError is enum again.
            // Error::ConfigError(e) => match e {
            //     ConfigError::NotFound => WarningMessage("Configuration file not fund".to_string()),
            //     ConfigError::ReadingFile(_) => {
            //         WarningMessage("Failed to read configuration file".to_string())
            //     }
            //     ConfigError::Unexpected(_) => WarningMessage("Unknown error".to_string()),
            // },
            Error::Daemon(e) => match e {
                RevaultDError::Rpc(code, _) => {
                    if *code == ErrorCode::COORDINATOR_SIG_STORE_ERROR as i32 {
                        WarningMessage("Coordinator could not store the signatures".to_string())
                    } else if *code == ErrorCode::COORDINATOR_SPEND_STORE_ERROR as i32 {
                        WarningMessage(
                            "Coordinator could not store the spend transaction".to_string(),
                        )
                    } else if *code == ErrorCode::TRANSPORT_ERROR as i32 {
                        WarningMessage("Failed to communicate with remote server".to_string())
                    } else if *code == ErrorCode::COSIGNER_INSANE_ERROR as i32 {
                        WarningMessage("The cosigner has an anormal behaviour, stop all operations and report to your security team".to_string())
                    } else if *code == ErrorCode::COSIGNER_ALREADY_SIGN_ERROR as i32 {
                        WarningMessage("The cosigner already signed the transaction".to_string())
                    } else if *code == RpcErrorCode::JSONRPC2_INVALID_PARAMS as i32 {
                        WarningMessage("Some fields are invalid".to_string())
                    } else {
                        WarningMessage("Internal error".to_string())
                    }
                }
                RevaultDError::Unexpected(_) => WarningMessage("Unknown error".to_string()),
                RevaultDError::Start(_) => {
                    WarningMessage("Revault daemon failed to start".to_string())
                }
                RevaultDError::NoAnswer | RevaultDError::Transport(..) => {
                    WarningMessage("Communication with Revault daemon failed".to_string())
                }
            },
            Error::Unexpected(_) => WarningMessage("Unknown error".to_string()),
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
