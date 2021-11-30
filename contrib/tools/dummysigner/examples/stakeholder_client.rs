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
            "cancel_tx": "cHNidP8BAF4CAAAAATdzv51EXeeNc1fv6E852OhRxc67KNaWd+BrA3qN1a/1AAAAAAD9////ARRLJgcAAAAAIgAgdfJpF3TIFneDGEawKCIA4oiyxZcQtY90MYPUklUH28UAAAAAAAEBK7iGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwBAwSBAAAAAQVhIQICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJqxRh2R2qRRyqV8ir5obrrhS+alScvjCHZjyZIisa3apFLbJrbicjJNybIPiobXZR4nXe5VhiKxsk1KHZ1iyaCIGAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmCCUdYAkAAAAAIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACICA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAAA==",
            "emergency_tx": "cHNidP8BAF4CAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////ATheJgcAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBKwAOJwcAAAAAIgAgdfJpF3TIFneDGEawKCIA4oiyxZcQtY90MYPUklUH28UBAwSBAAAAAQVHUiECWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MhA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOUq4iBgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACIGA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAAAA=",
            "emergency_unvault_tx": "cHNidP8BAF4CAAAAATdzv51EXeeNc1fv6E852OhRxc67KNaWd+BrA3qN1a/1AAAAAAD9////AWa7JQcAAAAAIgAgy7Co1PHzwoce0hHQR5RHMS72lSZudTF3bYrNgqLbkDYAAAAAAAEBK7iGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwBAwSBAAAAAQVhIQICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJqxRh2R2qRRyqV8ir5obrrhS+alScvjCHZjyZIisa3apFLbJrbicjJNybIPiobXZR4nXe5VhiKxsk1KHZ1iyaCIGAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmCCUdYAkAAAAAIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAA"
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
            "unvault_tx": "cHNidP8BAIkCAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////AriGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwwdQAAAAAAACIAIAjkMa8elv7dHUmYpDATWBtmMmpv9yyKFawMunvGQ1AMAAAAAAABASsADicHAAAAACIAIHXyaRd0yBZ3gxhGsCgiAOKIssWXELWPdDGD1JJVB9vFAQMEAQAAAAEFR1IhAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDIQNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDlKuIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJgglHWAJAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAA"
        }))
        .await
        .unwrap();

    if let Some(msg) = receiver.try_next().await.unwrap() {
        println!("GOT: {:?}", msg);
    }

    sleep(Duration::from_secs(2)).await;
}
