use crate::protocol::BmpMessage;
use crate::protocol::Decoder;

use std::io::Result;
use std::net::TcpStream;

#[derive(Debug)]
pub struct BmpClient {
    decoder: Decoder,
    stream: TcpStream,
}

impl BmpClient {
    pub fn new(stream: TcpStream) -> Self {
        let decoder = Decoder::new();

        Self { decoder, stream }
    }

    pub fn recv(&mut self) -> Result<BmpMessage> {
        self.decoder.decode(&mut self.stream)
    }
}
