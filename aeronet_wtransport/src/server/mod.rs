#![warn(clippy::future_not_send)]

mod back;
mod front;
#[cfg(feature = "bevy")]
pub mod plugin;

pub use back::Backend;
pub use front::Frontend;

use generational_arena::{Arena, Index};
use tokio::sync::{broadcast, mpsc};

use std::{
    collections::HashMap,
    fmt::Display,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use wtransport::{error::ConnectionError, Connection, ServerConfig};

use crate::{StreamId, StreamKind, Streams, TransportConfig};

pub(crate) const CHANNEL_BUF: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientId(pub(crate) Index);

impl Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (index, gen) = self.0.into_raw_parts();
        write!(f, "{}v{}", index, gen)
    }
}

impl ClientId {
    pub fn from_raw(raw: Index) -> Self {
        Self(raw)
    }

    pub fn into_raw(self) -> Index {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerStream {
    Datagram,
    Bi(StreamId),
    S2C(StreamId),
}

impl ServerStream {
    pub fn as_kind(self) -> StreamKind {
        match self {
            Self::Datagram => StreamKind::Datagram,
            Self::Bi(id) => StreamKind::Bi(id),
            Self::S2C(id) => StreamKind::S2C(id),
        }
    }
}

#[derive(Debug)]
pub enum Event<C2S> {
    Connecting {
        client: ClientId,
        authority: String,
        path: String,
        headers: HashMap<String, String>,
    },
    Connected {
        client: ClientId,
    },
    Recv {
        client: ClientId,
        msg: C2S,
    },
    Disconnected {
        client: ClientId,
        reason: SessionError,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum Request<S2C> {
    Send {
        client: ClientId,
        stream: ServerStream,
        msg: S2C,
    },
    Disconnect {
        client: ClientId,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("server closed")]
    ServerClosed,
    #[error("forced disconnect by server")]
    ForceDisconnect,
    #[error("failed to receive incoming session")]
    RecvSession(#[source] ConnectionError),
    #[error("failed to accept session")]
    AcceptSession(#[source] ConnectionError),
    #[error("stream {stream:?}")]
    Stream {
        stream: StreamKind,
        #[source]
        source: StreamError,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("failed to open stream")]
    Open(#[source] anyhow::Error),
    #[error("failed to receive data")]
    Recv(#[source] anyhow::Error),
    #[error("failed to send data")]
    Send(#[source] anyhow::Error),
    #[error("closed by client")]
    Closed,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientInfo {
    pub max_datagram_size: Option<usize>,
    pub remote_address: SocketAddr,
    pub rtt: Duration,
    pub stable_id: usize,
}

impl ClientInfo {
    pub fn from(conn: &Connection) -> Self {
        Self {
            max_datagram_size: conn.max_datagram_size(),
            remote_address: conn.remote_address(),
            rtt: conn.rtt(),
            stable_id: conn.stable_id(),
        }
    }
}

pub(crate) type SharedClients = Arc<Mutex<Arena<Option<ClientInfo>>>>;

pub fn create<C: TransportConfig>(
    config: ServerConfig,
    streams: Streams,
) -> (Frontend<C>, Backend<C>) {
    let (send_b2f, recv_b2f) = mpsc::channel::<Event<C::C2S>>(CHANNEL_BUF);
    let (send_f2b, _) = broadcast::channel::<Request<C::S2C>>(CHANNEL_BUF);
    let clients: SharedClients = Arc::new(Mutex::new(Arena::new()));

    let frontend = Frontend::<C> {
        send: send_f2b.clone(),
        recv: recv_b2f,
        clients: clients.clone(),
    };

    let backend = Backend::<C> {
        config,
        streams,
        send_b2f,
        send_f2b,
        clients,
    };

    (frontend, backend)
}
