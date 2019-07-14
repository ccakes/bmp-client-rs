use std::fmt;

/*
    Type = 0: Route Monitoring
    Type = 1: Statistics Report
    Type = 2: Peer Down Notification
    Type = 3: Peer Up Notification
    Type = 4: Initiation Message
    Type = 5: Termination Message
    Type = 6: Route Mirroring Message
*/
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MessageKind {
    RouteMonitoring,
    StatisticsReport,
    PeerDown,
    PeerUp,
    Initiation,
    Termination,
    RouteMirroring,

    // __Invalid
}

impl From<u8> for MessageKind {
    fn from(value: u8) -> Self {
        match value {
            0 => MessageKind::RouteMonitoring,
            1 => MessageKind::StatisticsReport,
            2 => MessageKind::PeerDown,
            3 => MessageKind::PeerUp,
            4 => MessageKind::Initiation,
            5 => MessageKind::Termination,
            6 => MessageKind::RouteMirroring,

            _ => panic!("invalid value for BMP Message Type"),
        }
    }
}

impl fmt::Display for MessageKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessageKind::RouteMonitoring => write!(f, "route_monitoring"),
            MessageKind::StatisticsReport => write!(f, "statistics_report"),
            MessageKind::PeerUp => write!(f, "peer_up"),
            MessageKind::PeerDown => write!(f, "peer_down"),
            MessageKind::Initiation => write!(f, "initiation"),
            MessageKind::Termination => write!(f, "termination"),
            MessageKind::RouteMirroring => write!(f, "route_mirroring"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PeerType {
    GlobalInstance,
    RdInstance,
    LocalInstance,
}

impl From<u8> for PeerType {
    fn from(value: u8) -> Self {
        match value {
            0 => PeerType::GlobalInstance,
            1 => PeerType::RdInstance,
            2 => PeerType::LocalInstance,

            _ => panic!("invalid value for BMP Peer Type"),
        }
    }
}

impl fmt::Display for PeerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerType::GlobalInstance => write!(f, "global"),
            PeerType::RdInstance => write!(f, "rd"),
            PeerType::LocalInstance => write!(f, "local"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[allow(non_snake_case)]
pub struct PeerFlags {
    pub V: bool,
    pub L: bool,
    pub A: bool,
}

#[allow(non_snake_case)]
impl From<u8> for PeerFlags {
    fn from(value: u8) -> Self {
        let V = value & 0b10000000 == 0b10000000;
        let L = value & 0b01000000 == 0b01000000;
        let A = value & 0b00100000 == 0b00100000;

        Self { V, L, A}
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum InformationType {
    String,
    SysDescr,
    SysName,
}

impl From<u16> for InformationType {
    fn from(value: u16) -> Self {
        match value {
            0 => InformationType::String,
            1 => InformationType::SysDescr,
            2 => InformationType::SysName,

            _ => panic!("invalid value for BMP Information Type"),
        }
    }
}

impl fmt::Display for InformationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InformationType::String => write!(f, "string"),
            InformationType::SysDescr => write!(f, "sys_descr"),
            InformationType::SysName => write!(f, "sys_name"),
        }
    }
}
