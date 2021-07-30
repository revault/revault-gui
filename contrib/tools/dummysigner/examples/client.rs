use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_serde::formats::*;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let mut socket = TcpStream::connect("0.0.0.0:8080").await.unwrap();

    let (reader, writer) = socket.split();

    let mut sender = SymmetricallyFramed::new(
        FramedWrite::new(writer, LengthDelimitedCodec::new()),
        SymmetricalJson::default(),
    );

    let mut receiver = SymmetricallyFramed::new(
        FramedRead::new(reader, LengthDelimitedCodec::new()),
        SymmetricalJson::<serde_json::Value>::default(),
    );

    // Send the value
    sender
        .send(json!({
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }
}
