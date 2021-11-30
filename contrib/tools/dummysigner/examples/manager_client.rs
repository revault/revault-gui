use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio_serde::formats::*;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// run dummysigner:
/// cargo run -- xprv9yZtssy7SLcqLCQetFHe35ENDKT58vh87MNFYdNkMyphUAcfCXdxLzgG4enc7ZT8NXjBtivtLrtpjZAJzyiTEAKM6NKUeFerP97DZdctJPr

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

    sender
        .send(json!({
            "spend_tx": "cHNidP8BALQCAAAAAc1946BSKWX5trghNlBq/IIYScLPYqr9Bqs2LfqOYuqcAAAAAAAIAAAAA+BAAAAAAAAAIgAgCOQxrx6W/t0dSZikMBNYG2Yyam/3LIoVrAy6e8ZDUAyA8PoCAAAAACIAIMuwqNTx88KHHtIR0EeURzEu9pUmbnUxd22KzYKi25A2CBH6AgAAAAAiACB18mkXdMgWd4MYRrAoIgDiiLLFlxC1j3Qxg9SSVQfbxQAAAAAAAQEruFn1BQAAAAAiACBI6M9l6zams92tyCK/4gbWyNfJMJzgoOv34L0X7GTovAEDBAEAAAABBWEhAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmrFGHZHapFHKpXyKvmhuuuFL5qVJy+MIdmPJkiKxrdqkUtsmtuJyMk3Jsg+KhtdlHidd7lWGIrGyTUodnWLJoIgYCApM6sQN+rQqkp4WMbUnWcF4fs7xfZrAJGK97nwDs1SYIJR1gCQAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAA",
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;
}
