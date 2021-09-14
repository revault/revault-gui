use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction as Psbt};

use serialport::{available_ports, SerialPortType};
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_serial::SerialPortBuilderExt;
use tokio_serial::SerialStream;

#[derive(Debug)]
pub struct Specter<T> {
    transport: T,
}

impl<T: Unpin + AsyncWrite + AsyncRead> Specter<T> {
    pub async fn fingerprint(&mut self) -> Result<String, SpecterError> {
        self.request("\r\n\r\nfingerprint\r\n").await
    }

    pub async fn sign_psbt(&mut self, psbt: &Psbt) -> Result<Psbt, SpecterError> {
        let mut new_psbt: Psbt = self
            .request(&format!(
                "\r\n\r\nsign {}\r\n",
                base64::encode(&encode::serialize(&psbt))
            ))
            .await
            .and_then(|resp| base64::decode(&resp).map_err(|e| SpecterError(e.to_string())))
            .and_then(|bytes| {
                encode::deserialize(&bytes).map_err(|e| SpecterError(e.to_string()))
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
    pub async fn try_connect_simulator<T: ToSocketAddrs + std::marker::Sized>(
        address: T,
    ) -> Result<Self, SpecterError> {
        let transport = TcpStream::connect(address)
            .await
            .map_err(|e| SpecterError(e.to_string()))?;
        Ok(Specter { transport })
    }
}

const SPECTER_VID: u16 = 61525;
const SPECTER_PID: u16 = 38914;

impl Specter<SerialStream> {
    pub async fn try_connect_serial() -> Result<Self, SpecterError> {
        let tty = match available_ports() {
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
                .ok_or_else(|| SpecterError("".into())),
            Err(e) => Err(SpecterError(format!("Error listing serial ports: {}", e))),
        }?;
        let transport = tokio_serial::new(tty, 9600)
            .open_native_async()
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
