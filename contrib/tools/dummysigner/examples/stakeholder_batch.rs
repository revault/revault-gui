use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio_serde::formats::*;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// run dummysigner:
/// cargo run -- --conf examples/examples_cfg.toml

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

    // Secure deposits
    sender
        .send(json!({
            "deposits": [
                {
                    "outpoint": "899aecbc9a3b06feaf096fc576b35da352a1ca1aa0c34db23ccfa944f30fae47:1",
                    "amount": 120000000,
                    "derivation_index": 0,
                },
                {
                    "outpoint": "f72c10cfb1e915c18f35aa7f658bea0fe75188e93abfcc13dae3359bd311caed:0",
                    "amount": 43000000,
                    "derivation_index": 1,
                }
            ]
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;

    // Delegate vaults
    sender
        .send(json!({
            "vaults": [
                {
                    "outpoint": "899aecbc9a3b06feaf096fc576b35da352a1ca1aa0c34db23ccfa944f30fae47:1",
                    "amount": 120000000,
                    "derivation_index": 0,
                },
                {
                    "outpoint": "f72c10cfb1e915c18f35aa7f658bea0fe75188e93abfcc13dae3359bd311caed:0",
                    "amount": 43000000,
                    "derivation_index": 1,
                }
            ]
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;
}
