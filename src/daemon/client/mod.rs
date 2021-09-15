// Rust JSON-RPC Library
// Written by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//     Wladimir J. van der Laan <laanwj@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//
//! Client support
//!
//! Support for connecting to JSONRPC servers over UNIX socets, sending requests,
//! and parsing responses
//!

pub mod error;
use error::Error;

#[cfg(windows)]
use uds_windows::UnixStream;

#[cfg(not(windows))]
use std::os::unix::net::UnixStream;

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{to_writer, Deserializer};

use tracing::debug;

/// A handle to a remote JSONRPC server
#[derive(Debug, Clone)]
pub struct Client {
    sockpath: PathBuf,
    timeout: Option<Duration>,
}

impl Client {
    /// Creates a new client
    pub fn new<P: AsRef<Path>>(sockpath: P) -> Client {
        Client {
            sockpath: sockpath.as_ref().to_path_buf(),
            timeout: None,
        }
    }

    /// Set an optional timeout for requests
    #[allow(dead_code)]
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Sends a request to a client
    pub fn send_request<S: Serialize + Debug, D: DeserializeOwned + Debug>(
        &self,
        method: &str,
        params: Option<S>,
    ) -> Result<Response<D>, Error> {
        // Setup connection
        let mut stream = UnixStream::connect(&self.sockpath)?;
        stream.set_read_timeout(self.timeout)?;
        stream.set_write_timeout(self.timeout)?;

        let request = Request {
            method,
            params,
            id: std::process::id(),
            jsonrpc: "2.0",
        };

        debug!("Sending to revaultd: {:#?}", request);

        to_writer(&mut stream, &request)?;

        let response: Response<D> = Deserializer::from_reader(&mut stream)
            .into_iter()
            .next()
            .map_or(Err(Error::NoErrorOrResult), |res| Ok(res?))?;
        if response
            .jsonrpc
            .as_ref()
            .map_or(false, |version| version != "2.0")
        {
            return Err(Error::VersionMismatch);
        }

        if response.id != request.id {
            return Err(Error::NonceMismatch);
        }

        debug!("Received from revaultd: {:#?}", response);

        Ok(response)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// A JSONRPC request object
pub struct Request<'f, T: Serialize> {
    /// The name of the RPC call
    pub method: &'f str,
    /// Parameters to the RPC call
    pub params: Option<T>,
    /// Identifier for this Request, which should appear in the response
    pub id: u32,
    /// jsonrpc field, MUST be "2.0"
    pub jsonrpc: &'f str,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
/// A JSONRPC response object
pub struct Response<T> {
    /// A result if there is one, or null
    pub result: Option<T>,
    /// An error if there is one, or null
    pub error: Option<error::RpcError>,
    /// Identifier for this Request, which should match that of the request
    pub id: u32,
    /// jsonrpc field, MUST be "2.0"
    pub jsonrpc: Option<String>,
}

impl<T> Response<T> {
    /// Extract the result from a response, consuming the response
    pub fn into_result(self) -> Result<T, Error> {
        if let Some(e) = self.error {
            return Err(Error::Rpc(e));
        }

        self.result.ok_or(Error::NoErrorOrResult)
    }

    /// Returns whether or not the `result` field is empty
    #[allow(dead_code)]
    pub fn is_none(&self) -> bool {
        self.result.is_none()
    }
}
