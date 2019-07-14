use serde_derive::Deserialize;

use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize)]
pub struct PrometheusConfig {
    // HTTP listen addr
    pub listen: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    // Log file
    pub log_file: Option<PathBuf>,
    // Listen addr
    pub listen: SocketAddr,
    // Prometheus config map
    pub prometheus: Option<PrometheusConfig>,
}