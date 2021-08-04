use serde_json::Value;
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream, ToSocketAddrs,
};
use tokio_serde::{
    formats::{Json, SymmetricalJson},
    SymmetricallyFramed,
};
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
}

pub type Receiver =
    SymmetricallyFramed<FramedRead<OwnedReadHalf, LengthDelimitedCodec>, Value, Json<Value, Value>>;

pub type Sender = SymmetricallyFramed<
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    Value,
    Json<Value, Value>,
>;
