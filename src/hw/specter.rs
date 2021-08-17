use bitcoin::{
    base64,
    consensus::encode,
    util::{bip32::DerivationPath, psbt::PartiallySignedTransaction as Psbt},
};

use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub struct Specter<T> {
    transport: T,
}

impl<T: Unpin + AsyncWrite + AsyncRead> Specter<T> {
    pub async fn fingerprint(&mut self) -> Result<String, SpecterError> {
        self.request("\r\n\r\nfingerprint\r\n").await
    }

    pub async fn sign_psbt(&mut self, psbt: &Psbt) -> Result<Psbt, SpecterError> {
        self.request(&format!(
            "\r\n\r\nsign {}\r\n",
            base64::encode(&encode::serialize(&psbt))
        ))
        .await
        .and_then(|resp| base64::decode(&resp).map_err(|e| SpecterError(e.to_string())))
        .and_then(|bytes| encode::deserialize(&bytes).map_err(|e| SpecterError(e.to_string())))
    }

    async fn request(&mut self, req: &str) -> Result<String, SpecterError> {
        self.transport
            .write_all(req.as_bytes())
            .await
            .map_err(|e| SpecterError(e.to_string()))?;

        let reader = tokio::io::BufReader::new(&mut self.transport);
        let mut lines = reader.lines();
        if let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| SpecterError(e.to_string()))?
        {
            if line != "ACK" {
                return Err(SpecterError("Received an incorrect answer".to_string()));
            }
        }

        if let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| SpecterError(e.to_string()))?
        {
            return Ok(line);
        }
        Err(SpecterError("Unexpected".to_string()))
    }
}

impl Specter<TcpStream> {
    pub async fn try_connect<T: ToSocketAddrs + std::marker::Sized>(
        address: T,
    ) -> Result<Self, SpecterError> {
        let transport = TcpStream::connect(address)
            .await
            .map_err(|e| SpecterError(e.to_string()))?;
        Ok(Specter { transport })
    }
}

#[derive(Debug)]
pub struct SpecterError(String);

impl std::fmt::Display for SpecterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Specter error: {}", self.0)
    }
}
