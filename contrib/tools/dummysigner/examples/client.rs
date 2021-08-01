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
            "derivation_path": "m/2",
            "target": {
                "unvault_tx": "cHNidP8BAIkCAAAAAfBWbcYqgFOgIg3ZhP6ywga8XHfyJIz7d/kOhMaOWFUKAAAAAAD9////ArQ9mjsAAAAAIgAgsGOkiWJaOOkFty2IXvkOqnF/gfseMK5YVBN2T6yy/1kwdQAAAAAAACIAIOD4aW9ds/RNXcNihA2bsw1c+bmg65auhz/3DoPPEg7oAAAAAAABASsAypo7AAAAACIAIAEbB6Bw50wTKaZFeZ4idQ6ARKNfgLYsjX7xAXfs9uRaAQMEAQAAAAEFi1QhAvuWyyfoIfsyrTwNoR8N80zSBdfJrGff+D9XKBRL7aEFIQJDFo5gffr4sTpOPvcI45wr3VtVCTh/yw/SiCJaCIjiVyECFG7jo1GSz0jtafAJHkLYmGRXwHA/RczAbYVG9HkXgeshAuFvKB3+S1vvSrL7uL1T3AfK8N0fPeVs9/qpxDkEugJTVK4AAQH9RwFSIQIGcwwFZVqf0EVFaTWYlQmxQ4oTCIkj+5KOstDoG1vgOSECV9gzXyTBD0h6wFIUbmaRf0g9PVPE3DR16px9x/AKz+tSrmR2qRRvoS9lQLcmcOnmhvvrvSYgPxBG3oisa3apFEWuYAoAKnNlB7Zvc0vqNpOAwYMciKxsk2t2qRRPt/TRROBIvMaRJ5knCEwt9Ruyq4isbJNrdqkU2Cc/EKsxI3LBbmYh2aZXI+cYREiIrGyTVIdnVCEDD2S5Iq7i/Vl/EEvGyztnDxyixsSbEHGhpsAQV12U/lohAqvkdbGZ7D1i+ldvruFqM0/bhv+ybc51vs66rt8yisP+IQMU89wzWVsNAWu1Ivb+OmdoByPYQsG5uK5rWf3Yq1zMtCECXrozBb08gp5OFVGqxzWOQXiDLHOeT8Rynv/kKN4DmKtUrwESsmgAAQElIQJYONwJSgKF2+Ox+K8SgruNIwYM+5dbLzkhSvjfoCCjnaxRhwA=",
            }
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
