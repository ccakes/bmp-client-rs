use crate::client::BmpClient;
use crate::config::Config;
use crate::protocol::MessageData;
use crate::protocol::enums::MessageKind;

// use flowrider_metrics::metrics::register_int_counter_vec;
use lazy_static::lazy_static;
use prometheus::IntCounterVec;

use std::collections::HashMap;
use std::error::Error;
use std::net::TcpListener;

// lazy_static! {
//     static ref FLOW_STATS: IntCounterVec = register_int_counter_vec("flowrider_bmp_messages", "Number of messages processed", &["hostname", "peer_ip_src"]).unwrap();
// }

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // Testing environment!
    // For simplicity will only deal with the first inbound connection.

    let tcp = TcpListener::bind(&config.listen)?;

    // Grab our first client!
    // let stream = tcp.incoming().next().unwrap()?;
    let stream = loop {
        let stream = tcp.incoming().next().unwrap()?;
        let client = stream.peer_addr()?.ip();

        if config.allowed_clients.contains(&client) {
            break stream;
        } else {
            log::info!("Rejecting client {}", client);
        }
    };
    log::info!("{} connected!", stream.peer_addr()?);

    let mut client = BmpClient::new(stream);

    let mut errors = 0usize;
    let mut total = 0usize;
    let mut kinds: HashMap<MessageKind, usize> = HashMap::new();
    let mut post_policy: HashMap<bool, usize> = HashMap::new();
    loop {
        let message = match client.recv() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Error decoding message: {}", e);

                if errors > 17 { log::warn!("{}", e.backtrace()); }
                errors += 1;

                continue;
            }
        };

        if message.kind == MessageKind::PeerUp {
            eprintln!("{:?}", message);
        }

        if message.kind == MessageKind::RouteMonitoring { break; }

        total += 1;
        kinds.entry(message.kind)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        if let MessageData::RouteMonitoring((hdr, _)) = message.message {
            post_policy.entry(hdr.peer_flags.L)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        }

        if total % 1000 == 0 {
            log::info!("Kinds: {:#?}", kinds);
            log::info!("Post-policy: {:?}", post_policy);
            log::info!("Errors: {}", errors);
        }
    }


    Ok(())
}
