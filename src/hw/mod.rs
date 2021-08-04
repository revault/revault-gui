use bitcoin::{base64, consensus::encode, util::psbt::PartiallySignedTransaction as Psbt};
use std::fmt::Debug;

mod dummysigner;
use dummysigner::DummySigner;

#[derive(Debug)]
pub struct Channel {
    device: DummySigner,
}

impl Channel {
    pub async fn try_connect() -> Result<Channel, Error> {
        let device = DummySigner::try_connect("0.0.0.0:8080").await?;
        Ok(Channel { device })
    }
}

#[derive(Debug, Clone)]
pub struct Error(String);
