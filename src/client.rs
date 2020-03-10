use crate::error::*;

use bmp_protocol::{BmpMessage, Decoder};
// use failure::{Error, format_err};

use std::error::Error as StdError;
use std::net::TcpStream;

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
    decoder: Decoder,
    stream: TcpStream,
}

impl BmpClient {
    /// Instantiate a new client
    pub fn new(stream: TcpStream) -> Self {
        let decoder = Decoder::new();

        Self { decoder, stream }
    }

    /// Block on the TcpStream and wait for the next BMP message
    ///
    /// Returns an error if the client disconnects or if there is an error decoding the message
    pub fn recv(&mut self) -> Result<BmpMessage, Error> {
        let mut buf = vec![0u8, 1];
        match self.stream.peek(&mut buf) {
            Ok(0) => {
                return Err(Error::disconnected());
                // return Err(format_err!("Client disconnected"));
            },
            _ => {}
        };
        
        self.decoder.decode(&mut self.stream).map_err(|e| (Box::new(e) as Box<dyn StdError>).into())
    }
}
