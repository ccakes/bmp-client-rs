// #![deny(missing_docs)]

//! Provides tools for receiving and decoding a stream of BMP messages
//!
//! BMP (BGP Monitoring Protocol) is a method for BGP-speakers, typically network routers
//! to provide telemetry relating to BGP state.

mod client;
mod error;
pub mod types {
    pub use bmp_protocol::types::*;
}

pub use self::client::BmpClient;
pub use self::error::*;