use crate::protocol::BmpMessage;
use crate::protocol::Decoder;

use failure::Error;

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

    pub fn recv(&mut self) -> Result<BmpMessage, Error> {
        // THis is a horrible hack and I hate it
        let mut buf = vec![0u8, 1];
        match self.stream.peek(&mut buf) {
            Ok(0) => {
                log::info!("Peer closed the connection");
                std::process::exit(0);
            },
            _ => {}
        };
        
        self.decoder.decode(&mut self.stream)
    }
}
