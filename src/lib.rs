use structopt::StructOpt;

mod client;
mod config;
mod handler;
mod protocol;
mod types;

use config::Config;

use std::error::Error;
use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use std::thread;

/// flowrider-blackhole
#[derive(StructOpt)]
#[structopt(name = "flowrider-blackhole")]
struct Args {
    /// Config file
    #[structopt(short = "c", long = "config")]
    config: PathBuf,

    /// Logging verbosity
    #[structopt(short = "v", parse(from_occurrences))]
    verbose: u8
}

pub fn run() -> Result<(), Box<dyn Error>> {
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

    handler::run(config)
}
