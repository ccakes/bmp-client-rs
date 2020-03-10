use std::net::TcpListener;

use bmp_client::{BmpClient, ErrorKind};

fn main() {
    // Take the first incoming connection on tcp/1790
    let tcp = TcpListener::bind("0.0.0.0:11019").unwrap();
    let stream = tcp.incoming().next().unwrap().unwrap();

    println!("{} connected!", stream.peer_addr().unwrap());

    // Create a new client from the TcpStream
    let mut client = BmpClient::new(stream);

    let mut num = 1usize;
    loop {
        match client.recv() {
            Ok(message) => println!("[{}] Got {} message", num, message.kind),
            Err(ref e) if e.kind == ErrorKind::PeerDisconnected => {
                eprintln!("Peer disconnected");
                std::process::exit(0);
            },
            Err(e) => {
                eprintln!("[{}] Error decoding BMP message: {:?}", num, e);
                std::process::exit(1);
            }
        };

        num += 1;
    }
}