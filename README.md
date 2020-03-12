# bmp-client

This is a simple BMP (BGP Monitoring Protocol) client for Rust. The heavy lifting is done within the bmp-protocol crate, this just provides a simple wrapper with some convenience functions.

### Usage

```toml
# Cargo.toml

[dependencies]
bmp-client = { git = "https://github.com/ccakes/bmp-client-rs" }
```

```rust
#[tokio::main]
async fn main() {
    let mut tcp = TcpListener::bind("0.0.0.0:1790").await.unwrap();

    loop {
        let (stream, peer) = tcp.accept().await.unwrap();
        println!("Client {} connected", peer);

        tokio::spawn(async move {
            let mut client = BmpClient::new(stream);

            while let Some(message) = client.recv().await {
                match message {
                    Ok(message) => println!("Received a {} message", message.kind),
                    Err(error) => {
                        eprintln!("{}", error);
                        std::process::exit(1);
                    }
                };
            }
        });
    }
}
```

## Contributing

Contributions are welcome, the library is still very barebones.