use revault_gui::daemon::client::{Client, RevaultDError};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::Debug;

pub type MockedRequests = HashMap<String, Result<Value, RevaultDError>>;

#[derive(Debug, Clone)]
pub struct DaemonClient {
    requests: MockedRequests,
}

impl Client for DaemonClient {
    type Error = Error;
    fn request<S: Serialize + Debug, D: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Option<S>,
    ) -> Result<D, Self::Error> {
        let req = json!({"method": method, "params": params}).to_string();
        if let Some(res) = self.requests.get(&req) {
            res.clone()
                .map(|value| serde_json::from_value(value).unwrap())
                .map_err(|e| Error::MockedError(e))
        } else {
            Err(Error::ResponseNotFound(req))
        }
    }
}

impl DaemonClient {
    pub fn new(requests: MockedRequests) -> Self {
        Self { requests }
    }
}

#[derive(Debug)]
pub enum Error {
    MockedError(RevaultDError),
    ResponseNotFound(String),
}

impl From<Error> for RevaultDError {
    fn from(e: Error) -> RevaultDError {
        match e {
            Error::MockedError(error) => error,
            Error::ResponseNotFound(request) => RevaultDError::Unexpected(format!(
                "mocked response not found for request: {}",
                request
            )),
        }
    }
}
