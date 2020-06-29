use tokio::net::TcpListener;

use bmp_client::BmpClient;

#[tokio::main]
async fn main() {
    // Take the first incoming connection on tcp/1790
    let mut tcp = TcpListener::bind("0.0.0.0:1790").await.unwrap();
    println!("Listening on 0.0.0.0:1790");

    loop {
        let (stream, peer) = tcp.accept().await.unwrap();
        println!("Client {} connected", peer);

        tokio::spawn(async move {
            // Create a new client from the TcpStream
            let mut client = BmpClient::new(stream);

            let mut num = 0usize;
            while let Some(message) = client.recv().await {
                num += 1;
                match message {
                    Ok(message) => println!("[{}] Got {} message", num, message.kind),
                    Err(error) => {
                        eprintln!("{}", error);
                        std::process::exit(1);
                    }
                };
            }
        });
    }
}