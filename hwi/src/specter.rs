use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction as Psbt};

use serialport::{available_ports, SerialPortType};
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
pub use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio_serial::SerialPortBuilderExt;
pub use tokio_serial::SerialStream;

use super::{HWIError, HWI};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Specter<T> {
    transport: T,
}

impl<T: Unpin + AsyncWrite + AsyncRead> Specter<T> {
    pub async fn fingerprint(&mut self) -> Result<String, SpecterError> {
        self.request("\r\n\r\nfingerprint\r\n").await
    }

    pub async fn sign(&mut self, psbt: &Psbt) -> Result<Psbt, SpecterError> {
        let mut new_psbt: Psbt = self
            .request(&format!(
                "\r\n\r\nsign {}\r\n",
                base64::encode(&encode::serialize(&psbt))
            ))
            .await
            .and_then(|resp| base64::decode(&resp).map_err(|e| SpecterError::Device(e.to_string())))
            .and_then(|bytes| {
                encode::deserialize(&bytes).map_err(|e| SpecterError::Device(e.to_string()))
            })?;

        // Psbt returned by specter wallet has all unnecessary fields removed,
        // only global transaction and partial signatures for all inputs remain in it.
        // In order to have the full Psbt, the partial_sigs are extracted and appended
        // to the original psbt.
        let mut psbt = psbt.clone();
        for i in 0..new_psbt.inputs.len() {
            psbt.inputs[i]
                .partial_sigs
                .append(&mut new_psbt.inputs[i].partial_sigs)
        }

        Ok(psbt)
    }

    async fn request(&mut self, req: &str) -> Result<String, SpecterError> {
        self.transport
            .write_all(req.as_bytes())
            .await
            .map_err(|e| SpecterError::Device(e.to_string()))?;

        let reader = tokio::io::BufReader::new(&mut self.transport);
        let mut lines = reader.lines();
        if let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| SpecterError::Device(e.to_string()))?
        {
            if line != "ACK" {
                return Err(SpecterError::Device(
                    "Received an incorrect answer".to_string(),
                ));
            }
        }

        if let Some(line) = lines
            .next_line()
            .await
            .map_err(|e| SpecterError::Device(e.to_string()))?
        {
            return Ok(line);
        }
        Err(SpecterError::Device("Unexpected".to_string()))
    }
}

pub const SPECTER_SIMULATOR_DEFAULT_ADDRESS: &str = "127.0.0.1:8789";

impl Specter<TcpStream> {
    pub async fn try_connect_simulator<T: ToSocketAddrs + std::marker::Sized>(
        address: T,
    ) -> Result<Self, SpecterError> {
        let transport = TcpStream::connect(address)
            .await
            .map_err(|e| SpecterError::Device(e.to_string()))?;
        Ok(Specter { transport })
    }
}
#[async_trait]
impl HWI for Specter<TcpStream> {
    async fn is_connected(&mut self) -> Result<(), HWIError> {
        self.fingerprint()
            .await
            .map_err(|_| HWIError::DeviceDisconnected)?;
        Ok(())
    }
    async fn sign_tx(&mut self, tx: &Psbt) -> Result<Psbt, HWIError> {
        self.sign(tx).await.map_err(|e| e.into())
    }
}

const SPECTER_VID: u16 = 61525;
const SPECTER_PID: u16 = 38914;

impl Specter<SerialStream> {
    pub fn get_serial_port() -> Result<String, SpecterError> {
        match available_ports() {
            Ok(ports) => ports
                .iter()
                .find_map(|p| {
                    if let SerialPortType::UsbPort(info) = &p.port_type {
                        if info.vid == SPECTER_VID && info.pid == SPECTER_PID {
                            Some(p.port_name.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .ok_or(SpecterError::DeviceNotFound),
            Err(e) => Err(SpecterError::Device(format!(
                "Error listing serial ports: {}",
                e
            ))),
        }
    }
    pub fn try_connect_serial() -> Result<Self, SpecterError> {
        let tty = Self::get_serial_port()?;
        let transport = tokio_serial::new(tty, 9600)
            .open_native_async()
            .map_err(|e| SpecterError::Device(e.to_string()))?;
        Ok(Specter { transport })
    }
}

#[async_trait]
impl HWI for Specter<SerialStream> {
    async fn is_connected(&mut self) -> Result<(), HWIError> {
        Self::get_serial_port().map_err(|_| HWIError::DeviceDisconnected)?;
        Ok(())
    }
    async fn sign_tx(&mut self, tx: &Psbt) -> Result<Psbt, HWIError> {
        self.sign(tx).await.map_err(|e| e.into())
    }
}

#[derive(Debug)]
pub enum SpecterError {
    DeviceNotFound,
    Device(String),
}

impl std::fmt::Display for SpecterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound => write!(f, "Specter not found"),
            Self::Device(e) => write!(f, "Specter error: {}", e),
        }
    }
}

impl From<SpecterError> for HWIError {
    fn from(e: SpecterError) -> HWIError {
        match e {
            SpecterError::DeviceNotFound => HWIError::DeviceNotFound,
            SpecterError::Device(e) => HWIError::Device(e),
        }
    }
}

#[cfg(feature = "revault")]
mod revault {
    use super::{SerialStream, Specter, TcpStream};
    use crate::app::revault::{NoRevaultApp, RevaultHWI};

    impl From<Specter<SerialStream>> for Box<dyn RevaultHWI + Send> {
        fn from(s: Specter<SerialStream>) -> Box<dyn RevaultHWI + Send> {
            Box::new(s)
        }
    }

    impl From<Specter<TcpStream>> for Box<dyn RevaultHWI + Send> {
        fn from(s: Specter<TcpStream>) -> Box<dyn RevaultHWI + Send> {
            Box::new(s)
        }
    }

    impl<T> NoRevaultApp for Specter<T> {}
}
