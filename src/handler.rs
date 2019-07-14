use crate::client::BmpClient;
use crate::config::Config;
use crate::protocol::enums::MessageKind;

// use flowrider_metrics::metrics::register_int_counter_vec;
use lazy_static::lazy_static;
use prometheus::IntCounterVec;
use rand::prelude::*;

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
    let stream = tcp.incoming().next().unwrap()?;
    log::info!("{} connected!", stream.peer_addr()?);

    // 
    let mut rng = rand::thread_rng();
    let mut client = BmpClient::new(stream);

    let mut msg_num = 1usize;
    let mut logged = 0usize;
    let mut errors = 0usize;
    loop {
        let message = match client.recv() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Error decoding message: {}", e);
                errors += 1;

                if errors >= 3 { break; }
                continue;
            }
        };

        log::info!("{} - {:?}", msg_num, message.kind);

        // Only log a random sample of messages
        // if logged <= 20 && rng.gen::<f64>() > 0.80f64 && message.kind == MessageKind::RouteMonitoring {
        //     log::info!("{:#?}", message);
        //     logged += 1;
        // }
        msg_num += 1;

        // if logged >= 20 { break; }
    }

    log::info!("Logged {} out of {} seen messages",  logged, msg_num);

    Ok(())
}
