// use crate::protocol::bgp::{
//     Open as BgpOpen,
// };
use crate::protocol::enums::*;
use super::{BmpMessage, MessageData};

use bgp_rs::Capabilities;
use byteorder::{BigEndian, ReadBytesExt};
use failure::{Error, format_err};
use hashbrown::HashMap;

use std::io::{Cursor, Read};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

// Stupid workaround to skip first malformed UPDATE message
use std::sync::atomic::{AtomicUsize, Ordering};
static SKIPPED: AtomicUsize = AtomicUsize::new(1);
static SKIP_MAX: usize = 2;

#[derive(Clone, Debug)]
pub struct Decoder {
    client_capabilities: HashMap<IpAddr, Capabilities>,
}

// Debug tool
impl Drop for Decoder {
    fn drop(&mut self) {
        // Convenience - free sorting
        use std::collections::BTreeSet;

        if std::thread::panicking() {
            log::error!("Decoder state");

            let keys: BTreeSet<IpAddr> = self.client_capabilities.keys().copied().collect();
            for k in &keys {
                println!("{} => {:#?}", k, self.client_capabilities.get(k).unwrap());
            }
        }
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self { client_capabilities: HashMap::new() }
    }

    pub fn decode(&mut self, input: &mut dyn Read) -> Result<BmpMessage, Error> {
        // Read BMP header
        let version = input.read_u8()?;
        let length = input.read_u32::<BigEndian>()?;
        let kind: MessageKind = input.read_u8()?.into();

        // The length we just read is the entire message, so calculate how much we have to go
        // and read it. Pulling the lot off the wire here is nice because then if the decoding
        // fails for whatever reason, we can keep going and *should* be at the right spot.
        let remaining = (length as usize) - 6;

        let mut buf = vec![0u8; remaining as usize];
        input.read_exact(&mut buf)?;

        // Create a Cursor over the Vec<u8> so we're not reliant on the TcpStream anymore. Help
        // prevent over/under reading if we error somewhere.
        let mut cur = Cursor::new(buf);

        // Now decode based on the MessageKind
        let juice = match kind {
            MessageKind::Initiation => {
                let buf_len = cur.get_ref().len() as u64;

                let mut tlv = vec![];
                while cur.position() < buf_len {
                    let kind = cur.read_u16::<BigEndian>()?;
                    cur.set_position( cur.position() - 2 );

                    let info = match kind {
                        x if x <= 2 => InformationTlv::decode(&mut cur)?,
                        _ => { break; }
                    };

                    tlv.push(info);
                }

                MessageData::Initiation(tlv)
            },
            MessageKind::PeerUp => {
                let peer_header = PeerHeader::decode(&mut cur)?;
                let message = PeerUp::decode(&peer_header.peer_flags, &mut cur)?;

                // Record the speaker capabilities, we'll use these later
                self.client_capabilities.entry(peer_header.peer_addr)
                    .or_insert_with(|| {
                        match (&message.sent_open, &message.recv_open) {
                            (Some(s), Some(r)) => Capabilities::common(s, r).expect("missing capabilities"),
                            _ => { log::warn!("Missing BGP OPENs"); Capabilities::default() }
                        }
                    });
                    // .or_insert_with(|| Capabilities::common(&message.sent_open, &message.recv_open).expect("missing capabilities"));

                MessageData::PeerUp((peer_header, message))
            },
            MessageKind::PeerDown => {
                // Make sure to clean up self.capabilities
                MessageData::Unimplemented
            },
            MessageKind::RouteMonitoring => {
                let peer_header = PeerHeader::decode(&mut cur)?;
                let capabilities = self.client_capabilities.get(&peer_header.peer_addr)
                    .ok_or_else(|| format_err!("No capabilities found for neighbor {}", peer_header.peer_addr))?;

                let header = bgp_rs::Header::parse(&mut cur)?;
                // let update = bgp_rs::Update::parse(&header, &mut cur, &capabilities)?;
                // DEBUG
                let update = bgp_rs::Update::parse(&header, &mut cur, &capabilities)
                    .map_err(|e| {
                        if SKIPPED.load(Ordering::SeqCst) < SKIP_MAX { SKIPPED.fetch_add(1, Ordering::SeqCst); return e; }
                        log::error!("Panicking after {} errors", SKIPPED.load(Ordering::SeqCst));
                        log::error!("message.kind: {}", kind);
                        panic!()
                    })?;

                MessageData::RouteMonitoring((peer_header, update))
                // MessageData::Unimplemented
            },
            _ => MessageData::Unimplemented
        };

        Ok(BmpMessage {
            version: version,
            kind: kind,

            // peer_header,
            message: juice
        })
    }
}

/// Per-Peer Header
///
/// The per-peer header follows the common header for most BMP messages.
/// The rest of the data in a BMP message is dependent on the MessageKind
/// field in the common header.
// #[derive(Copy, Clone, Debug)]
#[derive(Clone, Debug)]
pub struct PeerHeader {
    pub peer_type: PeerType,
    pub peer_flags: PeerFlags,
    pub peer_distinguisher: (u32, u32),        // depends on PeerType, see RFC7854 for details
    pub peer_addr: IpAddr,
    pub peer_asn: u32,
    pub peer_bgp_id: Ipv4Addr,
    pub timestamp: u32,
    pub timestamp_ms: u32,
}

// Assist with debugging
// #[cfg(debug_assertions)]
impl Drop for PeerHeader {
    fn drop(&mut self) {
        if std::thread::panicking() {
            log::error!("Panicked processing this message: {:#?}", self);
        }
    }
}

impl PeerHeader {
    pub fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let peer_type: PeerType = cur.read_u8()?.into();
        let peer_flags: PeerFlags = cur.read_u8()?.into();
        let peer_distinguisher = (cur.read_u32::<BigEndian>()?, cur.read_u32::<BigEndian>()?);

        let peer_addr = match peer_flags.V {
            // IPv4
            false => {
                // Throw away 12 bytes
                cur.read_exact(&mut [0u8; 12])?;
                IpAddr::V4( Ipv4Addr::from(cur.read_u32::<BigEndian>()?) )
            },
            // IPv6
            true => {
                IpAddr::V6( Ipv6Addr::from(cur.read_u128::<BigEndian>()?) )
            }
        };

        let peer_asn = match peer_flags.A {
            // 2 byte ASNs
            true => {
                // Throw away 2 bytes
                cur.read_exact(&mut [0u8; 2])?;
                u32::from( cur.read_u16::<BigEndian>()? )
            },
            // 4 byte ASNs
            false => cur.read_u32::<BigEndian>()?
        };

        let peer_bgp_id = Ipv4Addr::from( cur.read_u32::<BigEndian>()? );

        let timestamp = cur.read_u32::<BigEndian>()?;
        let timestamp_ms = cur.read_u32::<BigEndian>()?;

        Ok(Self {
            peer_type,
            peer_flags,
            peer_distinguisher,
            peer_addr,
            peer_asn,
            peer_bgp_id,
            timestamp,
            timestamp_ms,
        })
    }
}

/// Information TLV
///
/// The Information TLV is used by the Initiation and Peer Up messages.
#[derive(Clone, Debug)]
pub struct InformationTlv {
    pub information_type: InformationType,
    pub value: String,
}

impl InformationTlv {
    pub fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let information_type = InformationType::from( cur.read_u16::<BigEndian>()? );
        let len = cur.read_u16::<BigEndian>()?;

        let mut val_buf = vec![0u8; len as usize];
        cur.read_exact(&mut val_buf)?;
        let value = String::from_utf8(val_buf).unwrap();

        Ok(Self { information_type, value })
    }
}

/// Peer Up Notification
///
/// The Peer Up message is used to indicate that a peering session has
/// come up (i.e., has transitioned into the Established state).
#[derive(Clone, Debug)]
pub struct PeerUp {
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_port: u16,
    // pub sent_open: BgpOpen,
    // pub recv_open: BgpOpen,
    pub sent_open: Option<bgp_rs::Open>,
    pub recv_open: Option<bgp_rs::Open>,
    pub information: Vec<InformationTlv>,
}

impl PeerUp {
    pub fn decode(peer_flags: &PeerFlags, cur: &mut Cursor<Vec<u8>>) -> Result<Self, Error> {
        let local_addr = match peer_flags.V {
            // IPv4
            false => {
                // Throw away 12 bytes
                cur.read_exact(&mut [0u8; 12])?;
                IpAddr::V4( Ipv4Addr::from(cur.read_u32::<BigEndian>()?) )
            },
            // IPv6
            true => {
                IpAddr::V6( Ipv6Addr::from(cur.read_u128::<BigEndian>()?) )
            }
        };

        let local_port = cur.read_u16::<BigEndian>()?;
        let remote_port = cur.read_u16::<BigEndian>()?;

        // For at least some routers (ie adm-b1) the PeerUp messages are missing the
        // OPENs. Short-circuit here until I can figure out whats going on
        if cur.position() == cur.get_ref().len() as u64 {
            return Ok(PeerUp {
                local_addr,
                local_port,
                remote_port,
                sent_open: None,
                recv_open: None,
                information: vec![]
            });
        }

        let sent_hdr = bgp_rs::Header::parse(cur)?;
        assert!(sent_hdr.record_type == 1);
        let sent_open = Some(bgp_rs::Open::parse(cur)?);

        let recv_hdr = bgp_rs::Header::parse(cur)?;
        assert!(recv_hdr.record_type == 1);
        let recv_open = Some(bgp_rs::Open::parse(cur)?);

        // Get the inner buffer length, then pull out TLVs until it's consumed
        let buf_len = cur.get_ref().len() as u64;

        let mut information = vec![];
        while cur.position() < buf_len {
            information.push( InformationTlv::decode(cur)? );
        }

        Ok(PeerUp {
            local_addr,
            local_port,
            remote_port,
            sent_open,
            recv_open,
            information
        })
    }
}
