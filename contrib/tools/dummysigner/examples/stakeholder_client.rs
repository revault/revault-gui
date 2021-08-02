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
            "derivation_path": "m/3",
            "target": {
                "emergency_tx": "cHNidP8BAF4CAAAAAfqoxqY7yr3+nrH76bhDJP/nAFgAWYBBcgV3W6pwza6YAAAAAAD9////AQyEmjsAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBKwDKmjsAAAAAIgAgAG/6xQtRW31Y3LP2i2p7ANW8xZq+gK/FiHQufa/1s4UBAwSBAAAAAQWLVCEDf4+t53UwS4/4FZgFWXMSUbGLmWCIyreZ8YvU9mEfgQohAnfBC2P3QWPSNZAiGeRF+c5VukxYNOmF/HvoGTpRV7z1IQLQohhdvGJvO9Y8oLlNrB8BOby+ryEKIGQQjOE9J0CzPyECNrqEQUxCL0b3SyyFi0EeBx3a4xTLlQiRZQ+WtQcFN4dUrgAA",
                "emergency_unvault_tx": "cHNidP8BAF4CAAAAAXVHjUEMqSAxuHtxEaRO+5cI+Hj82I0W6eB6PdDtT0vKAAAAAAD9////AcramTsAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBK7Q9mjsAAAAAIgAg1pYVcYmkjg4tLILyT6qS0HahDjrHqOT05LffASl5Ez8BAwSBAAAAAQX9RwFSIQLKu0S06bTqM58ay1TmFWXs+3OwjYhNK8e07OGWSCCnNCEDYBvfdw9dYjhEWjiq8J8rWjDAyHgM50b2o8GMIpV8YFxSrmR2qRRDPLtPpqb1OgpbCG5WVtKYSmcwMoisa3apFCmNZawaARqOGBl46mYiMzscMrlDiKxsk2t2qRQO2nSPQ7YkMp/YMNN8EdXlMbxGY4isbJNrdqkU7ogZZkvgT+jBUsDQf4HJt0jS4+OIrGyTVIdnVCEDD2S5Iq7i/Vl/EEvGyztnDxyixsSbEHGhpsAQV12U/lohAqvkdbGZ7D1i+ldvruFqM0/bhv+ybc51vs66rt8yisP+IQMU89wzWVsNAWu1Ivb+OmdoByPYQsG5uK5rWf3Yq1zMtCECXrozBb08gp5OFVGqxzWOQXiDLHOeT8Rynv/kKN4DmKtUrwESsmgAAA==",
                "cancel_tx": "cHNidP8BAF4CAAAAAXVHjUEMqSAxuHtxEaRO+5cI+Hj82I0W6eB6PdDtT0vKAAAAAAD9////AcramTsAAAAAIgAgAG/6xQtRW31Y3LP2i2p7ANW8xZq+gK/FiHQufa/1s4UAAAAAAAEBK7Q9mjsAAAAAIgAg1pYVcYmkjg4tLILyT6qS0HahDjrHqOT05LffASl5Ez8BAwSBAAAAAQX9RwFSIQLKu0S06bTqM58ay1TmFWXs+3OwjYhNK8e07OGWSCCnNCEDYBvfdw9dYjhEWjiq8J8rWjDAyHgM50b2o8GMIpV8YFxSrmR2qRRDPLtPpqb1OgpbCG5WVtKYSmcwMoisa3apFCmNZawaARqOGBl46mYiMzscMrlDiKxsk2t2qRQO2nSPQ7YkMp/YMNN8EdXlMbxGY4isbJNrdqkU7ogZZkvgT+jBUsDQf4HJt0jS4+OIrGyTVIdnVCEDD2S5Iq7i/Vl/EEvGyztnDxyixsSbEHGhpsAQV12U/lohAqvkdbGZ7D1i+ldvruFqM0/bhv+ybc51vs66rt8yisP+IQMU89wzWVsNAWu1Ivb+OmdoByPYQsG5uK5rWf3Yq1zMtCECXrozBb08gp5OFVGqxzWOQXiDLHOeT8Rynv/kKN4DmKtUrwESsmgAAQGLVCEDf4+t53UwS4/4FZgFWXMSUbGLmWCIyreZ8YvU9mEfgQohAnfBC2P3QWPSNZAiGeRF+c5VukxYNOmF/HvoGTpRV7z1IQLQohhdvGJvO9Y8oLlNrB8BOby+ryEKIGQQjOE9J0CzPyECNrqEQUxCL0b3SyyFi0EeBx3a4xTLlQiRZQ+WtQcFN4dUrgA=",
            }
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;

    // Send the value
    sender
        .send(json!({
            "derivation_path": "m/3",
            "target": {
                "unvault_tx": "cHNidP8BAIkCAAAAAfqoxqY7yr3+nrH76bhDJP/nAFgAWYBBcgV3W6pwza6YAAAAAAD9////ArQ9mjsAAAAAIgAg1pYVcYmkjg4tLILyT6qS0HahDjrHqOT05LffASl5Ez8wdQAAAAAAACIAIBeqOVg1yhFzVZfNpl5L0FDH8jNoOdDu1llToc0cH0SxAAAAAAABASsAypo7AAAAACIAIABv+sULUVt9WNyz9otqewDVvMWavoCvxYh0Ln2v9bOFAQMEAQAAAAEFi1QhA3+Pred1MEuP+BWYBVlzElGxi5lgiMq3mfGL1PZhH4EKIQJ3wQtj90Fj0jWQIhnkRfnOVbpMWDTphfx76Bk6UVe89SEC0KIYXbxibzvWPKC5TawfATm8vq8hCiBkEIzhPSdAsz8hAja6hEFMQi9G90sshYtBHgcd2uMUy5UIkWUPlrUHBTeHVK4AAQH9RwFSIQLKu0S06bTqM58ay1TmFWXs+3OwjYhNK8e07OGWSCCnNCEDYBvfdw9dYjhEWjiq8J8rWjDAyHgM50b2o8GMIpV8YFxSrmR2qRRDPLtPpqb1OgpbCG5WVtKYSmcwMoisa3apFCmNZawaARqOGBl46mYiMzscMrlDiKxsk2t2qRQO2nSPQ7YkMp/YMNN8EdXlMbxGY4isbJNrdqkU7ogZZkvgT+jBUsDQf4HJt0jS4+OIrGyTVIdnVCEDD2S5Iq7i/Vl/EEvGyztnDxyixsSbEHGhpsAQV12U/lohAqvkdbGZ7D1i+ldvruFqM0/bhv+ybc51vs66rt8yisP+IQMU89wzWVsNAWu1Ivb+OmdoByPYQsG5uK5rWf3Yq1zMtCECXrozBb08gp5OFVGqxzWOQXiDLHOeT8Rynv/kKN4DmKtUrwESsmgAAQElIQNso3rwz3JXQOjnQy4WJ3kLVdbEjopUx74+rwZ5LGnWIaxRhwA="
            }
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;
}
