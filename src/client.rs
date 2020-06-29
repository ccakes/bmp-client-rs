use crate::error::*;

use bmp_protocol::{
    types::BmpMessage,
    BmpDecoder
};

use tokio::{
    net::TcpStream,
    stream::StreamExt,
};
use tokio_util::codec::FramedRead;

use std::time::{Duration, Instant};

/// ## BmpClient
///
/// Holds the `TcpStream` and Decoder state
///
/// ```
/// let client = BmpClient::new(tcp_stream)?;
/// let decoded_message = client.recv()?;
/// ```
#[derive(Debug)]
pub struct BmpClient {
    connected: Instant,
    inner: FramedRead<TcpStream, BmpDecoder>,
    messages: usize,
}

impl BmpClient {
    /// Instantiate a new client
    pub fn new(stream: TcpStream) -> Self {
        let inner = FramedRead::new(stream, BmpDecoder::new());

        Self {
            connected: Instant::now(),
            inner,
            messages: 0,
        }
    }

    /// Returns a Future that will resolve to the next message
    ///
    /// Returns an error if the client disconnects or if there is an error decoding the message
    pub async fn recv(&mut self) -> Option<Result<BmpMessage, Error>> {
        self.inner.next().await
            .and_then(|m| { self.messages += 1; Some(m) })
            .map(|thing| thing.map_err(|e| e.into()))
    }

    /// Return a Duration representing how long this client has been connected
    pub fn connected(&self) -> Duration {
        self.connected.elapsed()
    }

    /// Return the number of messages received from this client during the active session
    pub fn messages(&self) -> usize {
        self.messages
    }
}
