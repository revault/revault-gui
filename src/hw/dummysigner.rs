use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction as Psbt};
use iced::futures::{SinkExt, TryStreamExt};
use serde_json::{json, Value};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream, ToSocketAddrs,
};
use tokio_serde::{
    formats::{Json, SymmetricalJson},
    SymmetricallyFramed,
};

use serde::{self, Deserialize, Deserializer};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::Error;

#[derive(Debug)]
pub struct DummySigner {
    sender: Sender,
    receiver: Receiver,
}

impl DummySigner {
    pub async fn try_connect<T: ToSocketAddrs + std::marker::Sized>(
        address: T,
    ) -> Result<DummySigner, Error> {
        let socket = TcpStream::connect(address)
            .await
            .map_err(|e| Error(e.to_string()))?;

        let (reader, writer) = socket.into_split();

        let sender = SymmetricallyFramed::new(
            FramedWrite::new(writer, LengthDelimitedCodec::new()),
            SymmetricalJson::default(),
        );

        let receiver = SymmetricallyFramed::new(
            FramedRead::new(reader, LengthDelimitedCodec::new()),
            SymmetricalJson::<serde_json::Value>::default(),
        );
        Ok(Self { sender, receiver })
    }

    pub async fn send(&mut self, request: Value) -> Result<Value, Error> {
        tracing::debug!("hw request: {:?}", request);
        self.sender
            .send(request)
            .await
            .map_err(|e| Error(e.to_string()))?;

        if let Some(msg) = self
            .receiver
            .try_next()
            .await
            .map_err(|e| Error(e.to_string()))?
        {
            tracing::debug!("hw responded: {:?}", msg);
            return Ok(msg);
        }
        Err(Error("No answer from dummysigner".to_string()))
    }

    pub async fn ping(&mut self) -> Result<(), Error> {
        self.send(json!({"request": "ping"})).await?;

        Ok(())
    }

    pub async fn sign_revocation_txs(
        &mut self,
        emergency_tx: Psbt,
        emergency_unvault_tx: Psbt,
        cancel_tx: Psbt,
    ) -> Result<Box<Vec<Psbt>>, Error> {
        let res = self
            .send(json!({
                "emergency_tx": base64::encode(&encode::serialize(&emergency_tx)),
                "emergency_unvault_tx": base64::encode(&encode::serialize(&emergency_unvault_tx)),
                "cancel_tx": base64::encode(&encode::serialize(&cancel_tx))
            }))
            .await?;

        let txs: RevocationTransactions =
            serde_json::from_value(res).map_err(|e| Error(e.to_string()))?;
        Ok(Box::new(vec![
            txs.emergency_tx,
            txs.emergency_unvault_tx,
            txs.cancel_tx,
        ]))
    }

    pub async fn sign_unvault_tx(&mut self, unvault_tx: Psbt) -> Result<Box<Vec<Psbt>>, Error> {
        let res = self
            .send(json!({
                "unvault_tx": base64::encode(&encode::serialize(&unvault_tx)),
            }))
            .await?;

        let tx: UnvaultTransaction =
            serde_json::from_value(res).map_err(|e| Error(e.to_string()))?;
        Ok(Box::new(vec![tx.unvault_tx]))
    }

    pub async fn sign_spend_tx(&mut self, spend_tx: Psbt) -> Result<Box<Vec<Psbt>>, Error> {
        let res = self
            .send(json!({
                "spend_tx": base64::encode(&encode::serialize(&spend_tx)),
            }))
            .await?;

        let tx: SpendTransaction = serde_json::from_value(res).map_err(|e| Error(e.to_string()))?;
        Ok(Box::new(vec![tx.spend_tx]))
    }
}

#[derive(Deserialize)]
pub struct RevocationTransactions {
    #[serde(deserialize_with = "deserialize_psbt")]
    pub cancel_tx: Psbt,

    #[serde(deserialize_with = "deserialize_psbt")]
    pub emergency_tx: Psbt,

    #[serde(deserialize_with = "deserialize_psbt")]
    pub emergency_unvault_tx: Psbt,
}

#[derive(Deserialize)]
pub struct UnvaultTransaction {
    #[serde(deserialize_with = "deserialize_psbt")]
    pub unvault_tx: Psbt,
}

#[derive(Deserialize)]
pub struct SpendTransaction {
    #[serde(deserialize_with = "deserialize_psbt")]
    pub spend_tx: Psbt,
}

pub fn deserialize_psbt<'de, D>(deserializer: D) -> Result<Psbt, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let bytes: Vec<u8> = base64::decode(&s).map_err(serde::de::Error::custom)?;
    encode::deserialize(&bytes).map_err(serde::de::Error::custom)
}

pub type Receiver =
    SymmetricallyFramed<FramedRead<OwnedReadHalf, LengthDelimitedCodec>, Value, Json<Value, Value>>;

pub type Sender = SymmetricallyFramed<
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    Value,
    Json<Value, Value>,
>;
