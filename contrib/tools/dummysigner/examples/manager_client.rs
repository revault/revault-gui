use futures::prelude::*;
use serde_json::json;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio_serde::formats::*;
use tokio_serde::SymmetricallyFramed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

/// run dummysigner:
/// cargo run -- xprv9zFeRZgUZaUZBEUq1vPFLpUavHPK5YZ6N2qeqCYe7GLxGVY9SRHuN5Uwd5YN56tMUKe2qPhmvP8fC1GBEAFRAwbJQi86swWvvGM5tXBpJt6

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
            "spend_tx": "cHNidP8BALQCAAAAAUUhXqr4rxtCZIqOCy2jWxtsPCTNQMXhggjjDlqMCLLMAAAAAAASAAAAA3BdAAAAAAAAIgAg4Phpb12z9E1dw2KEDZuzDVz5uaDrlq6HP/cOg88SDugAG7cAAAAAACIAIMuwqNTx88KHHtIR0EeURzEu9pUmbnUxd22KzYKi25A2sKPiOgAAAAAiACABGwegcOdMEymmRXmeInUOgESjX4C2LI1+8QF37PbkWgAAAAAAAQErtD2aOwAAAAAiACCwY6SJYlo46QW3LYhe+Q6qcX+B+x4wrlhUE3ZPrLL/WQEDBAEAAAABBf1HAVIhAgZzDAVlWp/QRUVpNZiVCbFDihMIiSP7ko6y0OgbW+A5IQJX2DNfJMEPSHrAUhRuZpF/SD09U8TcNHXqnH3H8ArP61KuZHapFG+hL2VAtyZw6eaG++u9JiA/EEbeiKxrdqkURa5gCgAqc2UHtm9zS+o2k4DBgxyIrGyTa3apFE+39NFE4Ei8xpEnmScITC31G7KriKxsk2t2qRTYJz8QqzEjcsFuZiHZplcj5xhESIisbJNUh2dUIQMPZLkiruL9WX8QS8bLO2cPHKLGxJsQcaGmwBBXXZT+WiECq+R1sZnsPWL6V2+u4WozT9uG/7JtznW+zrqu3zKKw/4hAxTz3DNZWw0Ba7Ui9v46Z2gHI9hCwbm4rmtZ/dirXMy0IQJeujMFvTyCnk4VUarHNY5BeIMsc55PxHKe/+Qo3gOYq1SvARKyaAABASUhAlg43AlKAoXb47H4rxKCu40jBgz7l1svOSFK+N+gIKOdrFGHAAABAYtUIQL7lssn6CH7Mq08DaEfDfNM0gXXyaxn3/g/VygUS+2hBSECQxaOYH36+LE6Tj73COOcK91bVQk4f8sP0ogiWgiI4lchAhRu46NRks9I7WnwCR5C2JhkV8BwP0XMwG2FRvR5F4HrIQLhbygd/ktb70qy+7i9U9wHyvDdHz3lbPf6qcQ5BLoCU1SuAA=="
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;
}
