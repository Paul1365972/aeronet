//! Types for representing QUIC streams and their directions.
//!
//! This module lays out building blocks for the higher level stream APIs, which will depend on
//! whether you are on the client or server side. This is to ensure type safety, as you e.g. cannot
//! send along an S2C channel from the client side.
//! 
//! See [`StreamKind`] for an explanation of the underlying streams.

/// A side-agnostic type representing a kind of stream used for data transport.
///
/// WebTransport uses the QUIC protocol internally, which allows using multiple streams over the
/// same connection. This type represents which of these streams is used to transport some data.
///
/// There may be multiple streams of the same type open on a single connection, which is why the
/// extra [`StreamId`] field is used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamKind {
    /// Not really a stream, however we treat it as one for the sake of the API.
    ///
    /// Only one of these "streams" exists in a single connection.
    Datagram,
    /// A bidirectional stream, C2S and S2C.
    Bi(StreamId),
    /// A unidirectional stream in the C2S direction.
    C2S(StreamId),
    /// A unidirectional stream in the S2C direction.
    S2C(StreamId),
}

/// An identifier for a single instance of an opened stream.
///
/// Since you can open multiple streams of the same type on a connection, we need a way to index
/// which exact stream we're using when sending data. This struct provides an opaque way to
/// represent this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamId(pub(crate) usize);

impl StreamId {
    /// Creates a stream ID from a raw index.
    ///
    /// # Warning
    ///
    /// Sending data using a stream ID created from this function may lead to bugs or panics, as
    /// an invalid stream will be used to send data. Prefer the functions on [`Streams`] instead
    /// to obtain a stream for your current side.
    pub fn from_raw(raw: usize) -> Self {
        Self(raw)
    }

    /// Gets the raw stream ID from this value.
    pub fn into_raw(self) -> usize {
        self.0
    }
}

/// Defines which streams will be created when a client/server connection is established.
///
/// When a connection is started, the transport opens all required channels during the establish
/// step. Channels cannot be opened or closed afterwards. This struct defines which channels will
/// be created during this step.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Streams {
    pub(crate) bi: usize,
    pub(crate) c2s: usize,
    pub(crate) s2c: usize,
}

impl Streams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bi(&mut self) -> StreamKind {
        let i = self.bi;
        self.bi += 1;
        StreamKind::Bi(StreamId(i))
    }

    pub fn add_c2s(&mut self) -> StreamKind {
        let i = self.c2s;
        self.c2s += 1;
        StreamKind::C2S(StreamId(i))
    }

    pub fn add_s2c(&mut self) -> StreamKind {
        let i = self.s2c;
        self.s2c += 1;
        StreamKind::S2C(StreamId(i))
    }
}
