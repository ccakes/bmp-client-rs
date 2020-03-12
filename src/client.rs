use crate::error::*;

use bmp_protocol::{
    types::BmpMessage,
    BmpDecoder
};
// use failure::{Error, format_err};

use tokio::{
    net::TcpStream,
    stream::StreamExt,
};
use tokio_util::codec::FramedRead;

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
    inner: FramedRead<TcpStream, BmpDecoder>
}

impl BmpClient {
    /// Instantiate a new client
    pub fn new(stream: TcpStream) -> Self {
        let inner = FramedRead::new(stream, BmpDecoder::new());

        Self { inner }
    }

    /// Block on the TcpStream and wait for the next BMP message
    ///
    /// Returns an error if the client disconnects or if there is an error decoding the message
    pub async fn recv(&mut self) -> Option<Result<BmpMessage, Error>> {
        self.inner.next().await
            .map(|thing| thing.map_err(|e| e.into()))
    }
}
