use std::hash::{Hash, Hasher};
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::{net::TcpListener, sync::Mutex};
use tokio_serde::{formats::SymmetricalJson, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use iced::futures::TryStreamExt;
use iced_futures::futures;

use iced::futures::SinkExt;

#[derive(Debug)]
pub struct Error(String);

pub fn listen<T: ToString>(url: T) -> iced::Subscription<ServerMessage> {
    iced::Subscription::from_recipe(Server {
        url: url.to_string(),
    })
}

pub struct Server {
    url: String,
}

impl<H, I> iced_native::subscription::Recipe<H, I> for Server
where
    H: Hasher,
{
    type Output = ServerMessage;

    fn hash(&self, state: &mut H) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);
        self.url.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        Box::pin(futures::stream::unfold(
            ServerState::Ready(self.url),
            move |state| async move {
                match state {
                    ServerState::Ready(url) => match tokio::net::TcpListener::bind(url).await {
                        Ok(l) => Some((ServerMessage::Started, ServerState::Listening(l))),
                        Err(_) => Some((ServerMessage::Stopped, ServerState::Stopped)),
                    },
                    ServerState::Listening(listener) => match listener.accept().await {
                        Ok((socket, addr)) => {
                            let (read_half, write_half) = socket.into_split();
                            let reader = SymmetricallyFramed::new(
                                FramedRead::new(read_half, LengthDelimitedCodec::new()),
                                SymmetricalJson::default(),
                            );
                            let writer = SymmetricallyFramed::new(
                                FramedWrite::new(write_half, LengthDelimitedCodec::new()),
                                SymmetricalJson::default(),
                            );
                            Some((
                                ServerMessage::NewConnection(addr, writer),
                                ServerState::Connected { listener, reader },
                            ))
                        }
                        Err(_) => Some((ServerMessage::Stopped, ServerState::Stopped)),
                    },
                    ServerState::Connected {
                        listener,
                        mut reader,
                    } => match reader.try_next().await {
                        Ok(Some(req)) => Some((
                            ServerMessage::Request(req),
                            ServerState::Connected { listener, reader },
                        )),
                        _ => Some((
                            ServerMessage::ConnectionDropped,
                            ServerState::Listening(listener),
                        )),
                    },
                    ServerState::Stopped => None,
                }
            },
        ))
    }
}

pub type Reader = SymmetricallyFramed<
    FramedRead<tokio::net::tcp::OwnedReadHalf, LengthDelimitedCodec>,
    serde_json::Value,
    tokio_serde::formats::Json<serde_json::Value, serde_json::Value>,
>;

pub type Writer = SymmetricallyFramed<
    FramedWrite<tokio::net::tcp::OwnedWriteHalf, LengthDelimitedCodec>,
    serde_json::Value,
    tokio_serde::formats::Json<serde_json::Value, serde_json::Value>,
>;

#[derive(Debug)]
pub enum ServerMessage {
    Started,
    NewConnection(SocketAddr, Writer),
    Request(serde_json::Value),
    Responded(Result<(), Error>),
    ConnectionDropped,
    Stopped,
}

pub enum ServerState {
    Ready(String),
    Listening(TcpListener),
    Connected {
        listener: TcpListener,
        reader: Reader,
    },
    Stopped,
}

pub async fn respond(writer: Arc<Mutex<Writer>>, value: serde_json::Value) -> Result<(), Error> {
    let mut writer = writer.lock().await;
    writer.send(value).await.unwrap();
    Ok(())
}
