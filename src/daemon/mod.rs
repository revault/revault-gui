pub mod client;
pub mod embedded;
pub mod model;

use std::io::ErrorKind;

#[derive(Debug, Clone)]
pub enum RevaultDError {
    /// Something was wrong with the request.
    Rpc(i32, String),
    /// Something was wrong with the communication.
    Transport(Option<ErrorKind>, String),
    /// Something unexpected happened.
    Unexpected(String),
    /// No response.
    NoAnswer,
}

impl std::fmt::Display for RevaultDError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Rpc(code, e) => write!(f, "Revaultd error rpc call: [{:?}] {}", code, e),
            Self::NoAnswer => write!(f, "Revaultd returned no answer"),
            Self::Transport(kind, e) => write!(f, "Revaultd transport error: [{:?}] {}", kind, e),
            Self::Unexpected(e) => write!(f, "Revaultd unexpected error: {}", e),
        }
    }
}
