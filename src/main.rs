use structopt::StructOpt;

mod config;

use bmp_client::*;
use bmp_protocol::*;
use config::Config;

use std::collections::HashMap;
use std::error::Error;
use std::{fs, io};
use std::io::Write;
use std::net::TcpListener;
use std::path::PathBuf;
// use std::thread;

/// bmp-client
#[derive(StructOpt)]
#[structopt(name = "bmp-client")]
struct Args {
    /// Config file
    #[structopt(short = "c", long = "config")]
    config: PathBuf,

    /// Logging verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8
}

pub fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();

    // Check that the config file exists and then read in into our Config struct
    if !args.config.exists() || !args.config.is_file() {
        eprintln!("{} doesn't exist!", args.config.display());
        std::process::exit(1);
    }

    // Read and parse our config
    let yaml = fs::read_to_string(args.config)?;
    let config: Config = serde_yaml::from_str(&yaml)?;

    // Derive log verbosity from args
    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace
    };

    // Init logger
    fern::Dispatch::new()
        .level(log::LevelFilter::Warn)
        .level_for(module_path!(), log_level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}({}:{}) {}",
                record.level(),
                record.file().unwrap(),
                record.line().unwrap(),
                message
            ))
        })
        .chain({
            config.log_file.clone()
                .and_then(|f| {
                    match fs::File::create(&f) {
                        Ok(fh) => Some(Box::new(fh) as Box<Write + Send>),
                        Err(_) => None
                    }
                })
                .unwrap_or_else(|| Box::new(io::stdout()) as Box<Write + Send>)
        })
        .apply()?;
    
    // Init prometrics_exporter
    // config.prometheus.as_ref().map(|p| {
    //     thread::spawn({
    //         let listen = p.listen;

    //         move || { prometheus_exporter::PrometheusExporter::run(&listen).expect("Error starting Prometheus exporter"); }
    //     });
    // });

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
            Err(ref e) if e.kind == ErrorKind::PeerDisconnected => {
                eprintln!("Peer disconnected");
                std::process::exit(0);
            },
            Err(e) => {
                log::warn!("Error decoding message: {:?}", e);
                errors += 1;

                break;
            }
        };

        // if message.kind == MessageKind::PeerUp {
        //     eprintln!("{:?}", message);
        // }

        // if message.kind == MessageKind::RouteMonitoring { break; }

        total += 1;
        kinds.entry(message.kind)
            .and_modify(|v| *v += 1)
            .or_insert(1);

        if let MessageData::RouteMonitoring((hdr, _)) = message.message {
            post_policy.entry(hdr.peer_flags.V)
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
