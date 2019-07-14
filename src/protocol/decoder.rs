// use crate::protocol::bgp::{
//     Open as BgpOpen,
// };
use crate::protocol::enums::*;
use super::{BmpMessage, MessageData};

use bgp_rs::Capabilities;
use byteorder::{BigEndian, ReadBytesExt};
use hashbrown::HashMap;

use std::io::{Cursor, Error, ErrorKind, Read, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Clone, Debug)]
pub struct Decoder {
    client_capabilities: HashMap<Ipv4Addr, Capabilities>,
}

impl Decoder {
    pub fn new() -> Self {
        Self { client_capabilities: HashMap::new() }
    }

    pub fn decode(&mut self, input: &mut dyn Read) -> Result<BmpMessage> {
        // Read BMP header
        let version = input.read_u8()?;
        let length = input.read_u32::<BigEndian>()?;
        let kind: MessageKind = input.read_u8()?.into();

        // The length we just read is the entire message, so calculate how much we have to go
        // and read it
        let remaining = (length as usize) - 6;

        let mut buf = vec![0u8; remaining as usize];
        input.read_exact(&mut buf)?;

        // Create a Cursor over the Vec<u8> so we're not reliant on the TcpStream anymore. Help
        // prevent over/under reading if we error somewhere.
        let mut cur = Cursor::new(buf);

        let peer_header = PeerHeader::decode(&mut cur)?;
        
        // Now decode based on the MessageKind
        let juice = match kind {
            MessageKind::Initiation => {
                let buf_len = cur.get_ref().len() as u64;

                let mut tlv = vec![];
                while cur.position() < buf_len {
                    // Hack because bmp_dump broke the file
                    let kind = cur.read_u16::<BigEndian>()?;
                    cur.set_position( cur.position() - 2 );

                    let info = match kind {
                        x if x < 2 => InformationTlv::decode(&mut cur)?,
                        _ => { break; }
                    };

                    tlv.push(info);
                }

                MessageData::Initiation(tlv)
            },
            MessageKind::PeerUp => {
                let message = PeerUp::decode(&peer_header.peer_flags, &mut cur)?;

                // Record the speaker capabilities, we'll use these later
                self.client_capabilities.entry(peer_header.peer_bgp_id)
                    .or_insert_with(|| Capabilities::parse(&message.recv_open).expect("missing capabilities"));

                MessageData::PeerUp(message)
            },
            MessageKind::RouteMonitoring => {
                let capabilities = self.client_capabilities.get(&peer_header.peer_bgp_id)
                    .ok_or_else(|| Error::new(
                        ErrorKind::Other,
                        format!("No capabilities found for neighbor {}", peer_header.peer_bgp_id)
                    ))?;

                let header = bgp_rs::Header::parse(&mut cur)?;
                // let update = bgp_rs::Update::parse(&header, &mut cur, &bgp_rs::Capabilities::default())?;
                // DEBUG
                let update = bgp_rs::Update::parse(&header, &mut cur, &capabilities)
                    .map_err(|e| {
                        log::warn!("Error decoding UPDATE");
                        e
                    })?;

                MessageData::RouteMonitoring(update)
                // MessageData::Unimplemented
            }

            _ => MessageData::Unimplemented
        };

        Ok(BmpMessage {
            version: version,
            kind: kind,

            peer_header,
            message: juice
        })
    }
}

/// Per-Peer Header
///
/// The per-peer header follows the common header for most BMP messages.
/// The rest of the data in a BMP message is dependent on the MessageKind
/// field in the common header.
#[derive(Copy, Clone, Debug)]
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

impl PeerHeader {
    pub fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<Self> {
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
    pub fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<Self> {
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
    pub sent_open: bgp_rs::Open,
    pub recv_open: bgp_rs::Open,
    pub information: Vec<InformationTlv>,
}

impl PeerUp {
    pub fn decode(peer_flags: &PeerFlags, cur: &mut Cursor<Vec<u8>>) -> Result<Self> {
        // let mut cur = Cursor::new(buf);

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

        // Read the message header dumping the marker
        // TODO should make this more robust and check it confirms to the RFC
        // let sent_open = BgpOpen::decode(cur)?;

        // And now read the recv OPEN
        // let recv_open = BgpOpen::decode(cur)?;

        let sent_hdr = bgp_rs::Header::parse(cur)?;
        assert!(sent_hdr.record_type == 1);
        let sent_open = bgp_rs::Open::parse(cur)?;

        let recv_hdr = bgp_rs::Header::parse(cur)?;
        assert!(recv_hdr.record_type == 1);
        let recv_open = bgp_rs::Open::parse(cur)?;

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
