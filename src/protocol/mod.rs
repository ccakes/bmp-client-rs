// pub mod bgp;
pub mod decoder;
pub mod enums;

pub use self::decoder::*;

use enums::*;

#[derive(Clone, Debug)]
pub struct BmpMessage {
    pub version: u8,
    pub kind: MessageKind,
    // pub peer_header: decoder::PeerHeader,

    pub message: MessageData,
}

#[derive(Clone, Debug)]
pub enum MessageData {
    Unimplemented,

    Initiation(Vec<decoder::InformationTlv>),
    PeerUp((PeerHeader, PeerUp)),
    RouteMonitoring((PeerHeader, bgp_rs::Update))
}
