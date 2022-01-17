use bitcoin::{
    base64, blockdata::transaction::OutPoint, consensus::encode,
    util::psbt::PartiallySignedTransaction as Psbt, Amount,
};

use async_trait::async_trait;
use futures::{stream::TryStreamExt, SinkExt};
use serde_json::{json, Value};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream, ToSocketAddrs,
};
use tokio_serde::{
    formats::{Json, SymmetricalJson},
    SymmetricallyFramed,
};

use serde::{self, Deserialize, Deserializer, Serialize};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{HWIError, HWI};

pub const DUMMYSIGNER_DEFAULT_ADDRESS: &str = "127.0.0.1:8080";

#[derive(Debug)]
pub struct DummySigner {
    sender: Sender,
    receiver: Receiver,
}

impl DummySigner {
    pub async fn try_connect<T: ToSocketAddrs + std::marker::Sized>(
        address: T,
    ) -> Result<DummySigner, DummySignerError> {
        let socket = TcpStream::connect(address)
            .await
            .map_err(|e| DummySignerError::Device(e.to_string()))?;

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

    pub async fn send(&mut self, request: Value) -> Result<Value, DummySignerError> {
        log::debug!("hw request: {:?}", request);
        self.sender
            .send(request)
            .await
            .map_err(|e| DummySignerError::Device(e.to_string()))?;

        if let Some(msg) = self
            .receiver
            .try_next()
            .await
            .map_err(|e| DummySignerError::Device(e.to_string()))?
        {
            log::debug!("hw responded: {:?}", msg);
            return Ok(msg);
        }
        Err(DummySignerError::Device(
            "No answer from dummysigner".to_string(),
        ))
    }

    pub async fn ping(&mut self) -> Result<(), DummySignerError> {
        self.send(json!({"request": "ping"})).await?;

        Ok(())
    }

    pub async fn sign_revocation_txs(
        &mut self,
        emergency_tx: &Psbt,
        emergency_unvault_tx: &Psbt,
        cancel_tx: &Psbt,
    ) -> Result<(Psbt, Psbt, Psbt), DummySignerError> {
        let res = self
            .send(json!({
                "emergency_tx": base64::encode(&encode::serialize(&emergency_tx)),
                "emergency_unvault_tx": base64::encode(&encode::serialize(&emergency_unvault_tx)),
                "cancel_tx": base64::encode(&encode::serialize(&cancel_tx))
            }))
            .await?;

        let txs: RevocationTransactions =
            serde_json::from_value(res).map_err(|e| DummySignerError::Device(e.to_string()))?;
        Ok((txs.emergency_tx, txs.emergency_unvault_tx, txs.cancel_tx))
    }

    pub async fn sign_unvault_tx(&mut self, unvault_tx: &Psbt) -> Result<Psbt, DummySignerError> {
        let res = self
            .send(json!({
                "unvault_tx": base64::encode(&encode::serialize(&unvault_tx)),
            }))
            .await?;

        let tx: UnvaultTransaction =
            serde_json::from_value(res).map_err(|e| DummySignerError::Device(e.to_string()))?;
        Ok(tx.unvault_tx)
    }

    pub async fn sign_spend_tx(&mut self, spend_tx: &Psbt) -> Result<Psbt, DummySignerError> {
        let res = self
            .send(json!({
                "spend_tx": base64::encode(&encode::serialize(&spend_tx)),
            }))
            .await?;

        let tx: SpendTransaction =
            serde_json::from_value(res).map_err(|e| DummySignerError::Device(e.to_string()))?;

        let mut has_signed = false;
        for i in 0..tx.spend_tx.inputs.len() {
            if spend_tx.inputs[i].partial_sigs.len() < tx.spend_tx.inputs[i].partial_sigs.len() {
                has_signed = true;
            }
        }

        if !has_signed {
            return Err(DummySignerError::DeviceDidNotSign);
        }

        Ok(tx.spend_tx)
    }

    pub async fn create_vaults(
        &mut self,
        deposits: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<(Psbt, Psbt, Psbt)>, DummySignerError> {
        let utxos: Vec<Utxo> = deposits
            .into_iter()
            .map(|(outpoint, amount, derivation_index)| Utxo {
                outpoint: *outpoint,
                amount: *amount,
                derivation_index: *derivation_index,
            })
            .collect();
        let mut res = self
            .send(json!({
                "deposits": utxos,
            }))
            .await?;

        if res.get("error") == Some(&json!("batch unsupported")) {
            return Err(DummySignerError::UnimplementedMethod);
        }

        let txs: Vec<RevocationTransactions> =
            serde_json::from_value(res["transactions"].take())
                .map_err(|e| DummySignerError::Device(e.to_string()))?;
        Ok(txs
            .into_iter()
            .map(|txs| (txs.emergency_tx, txs.emergency_unvault_tx, txs.cancel_tx))
            .collect())
    }

    pub async fn delegate_vaults(
        &mut self,
        vaults: &[(OutPoint, Amount, u32)],
    ) -> Result<Vec<Psbt>, DummySignerError> {
        let utxos: Vec<Utxo> = vaults
            .into_iter()
            .map(|(outpoint, amount, derivation_index)| Utxo {
                outpoint: *outpoint,
                amount: *amount,
                derivation_index: *derivation_index,
            })
            .collect();
        let mut res = self
            .send(json!({
                "vaults": utxos,
            }))
            .await?;

        if res.get("error") == Some(&json!("batch unsupported")) {
            return Err(DummySignerError::UnimplementedMethod);
        }

        let txs: Vec<UnvaultTransaction> = serde_json::from_value(res["transactions"].take())
            .map_err(|e| DummySignerError::Device(e.to_string()))?;
        Ok(txs.into_iter().map(|txs| txs.unvault_tx).collect())
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

#[derive(Debug, Clone, Serialize)]
pub struct Utxo {
    #[serde(with = "bitcoin_outpoint")]
    pub outpoint: OutPoint,
    #[serde(with = "bitcoin_amount")]
    pub amount: Amount,
    pub derivation_index: u32,
}

mod bitcoin_outpoint {
    use bitcoin::blockdata::transaction::OutPoint;
    use serde::{self, Serializer};

    pub fn serialize<S>(outpoint: &OutPoint, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&outpoint.to_string())
    }
}

mod bitcoin_amount {
    use bitcoin::Amount;

    use serde::{self, Serializer};

    pub fn serialize<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(amount.as_sat())
    }
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

#[derive(Debug)]
pub enum DummySignerError {
    UnimplementedMethod,
    DeviceDidNotSign,
    Device(String),
}

impl std::fmt::Display for DummySignerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DeviceDidNotSign => write!(f, "DummySigner did not sign psbt"),
            Self::UnimplementedMethod => write!(f, "Unimplemented method for dummysigner device"),
            Self::Device(e) => write!(f, "DummySigner error: {}", e),
        }
    }
}

impl From<DummySignerError> for HWIError {
    fn from(e: DummySignerError) -> HWIError {
        match e {
            DummySignerError::DeviceDidNotSign => HWIError::DeviceDidNotSign,
            DummySignerError::UnimplementedMethod => HWIError::UnimplementedMethod,
            DummySignerError::Device(e) => HWIError::Device(e),
        }
    }
}

#[async_trait]
impl HWI for DummySigner {
    async fn is_connected(&mut self) -> Result<(), HWIError> {
        self.ping().await.map_err(|_| HWIError::DeviceDisconnected)
    }
    async fn sign_tx(&mut self, tx: &Psbt) -> Result<Psbt, HWIError> {
        self.sign_spend_tx(tx).await.map_err(|e| e.into())
    }
}

#[cfg(feature = "revault")]
mod revault {
    use crate::{app::revault::RevaultHWI, HWIError};
    use async_trait::async_trait;
    use bitcoin::{
        blockdata::transaction::OutPoint, util::psbt::PartiallySignedTransaction as Psbt, Amount,
    };

    use super::DummySigner;

    #[async_trait]
    impl RevaultHWI for DummySigner {
        async fn has_revault_app(&mut self) -> bool {
            true
        }

        async fn sign_revocation_txs(
            &mut self,
            emergency_tx: &Psbt,
            emergency_unvault_tx: &Psbt,
            cancel_tx: &Psbt,
        ) -> Result<(Psbt, Psbt, Psbt), HWIError> {
            self.sign_revocation_txs(emergency_tx, emergency_unvault_tx, cancel_tx)
                .await
                .map_err(|e| e.into())
        }

        async fn sign_unvault_tx(&mut self, unvault_tx: &Psbt) -> Result<Psbt, HWIError> {
            self.sign_unvault_tx(unvault_tx).await.map_err(|e| e.into())
        }

        async fn create_vaults(
            &mut self,
            deposits: &[(OutPoint, Amount, u32)],
        ) -> Result<Vec<(Psbt, Psbt, Psbt)>, HWIError> {
            self.create_vaults(deposits).await.map_err(|e| e.into())
        }

        async fn delegate_vaults(
            &mut self,
            vaults: &[(OutPoint, Amount, u32)],
        ) -> Result<Vec<Psbt>, HWIError> {
            self.delegate_vaults(vaults).await.map_err(|e| e.into())
        }
    }

    impl From<DummySigner> for Box<dyn RevaultHWI + Send> {
        fn from(d: DummySigner) -> Box<dyn RevaultHWI + Send> {
            Box::new(d)
        }
    }
}
