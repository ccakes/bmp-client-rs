use byteorder::{BigEndian, ReadBytesExt};

use std::io::{Cursor, Read, Result};
use std::net::Ipv4Addr;

/// BGP OPEN message
#[derive(Clone, Debug)]
pub struct Open {
    pub version: u8,
    pub peer_asn: u16,
    pub hold_timer: u16,
    pub peer_rid: Ipv4Addr,
    pub parameters: Vec<OpenParameter>,
}

impl Open {
    pub fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<Self> {
        // Here we check to see if the cursor is at, or after the marker in the BGP message header
        let initial_pos = cur.position();
        let mut marker = [0u8; 16];
        cur.read_exact(&mut marker)?;

        if marker == [255u8; 16] {
            // Might as well check the type while we're here
            let _ = cur.read_u16::<BigEndian>()?; // length
            let _type = cur.read_u8()?;

            assert!(_type == 1);
        } else {
            // Reset the cursor position
            cur.set_position(initial_pos);
        }

        let version = cur.read_u8()?;
        let peer_asn = cur.read_u16::<BigEndian>()?;
        let hold_timer = cur.read_u16::<BigEndian>()?;
        let peer_rid = Ipv4Addr::from( cur.read_u32::<BigEndian>()? );

        let mut params_len = cur.read_u8()?;
        let mut parameters = vec![];

        while params_len > 0 {
            let (bytes_read, param) = OpenParameter::decode(cur)?;
            parameters.push(param);

            params_len -= bytes_read;
        }

        Ok(Open {
            version,
            peer_asn,
            hold_timer,
            peer_rid,
            parameters
        })
    }
}

#[derive(Clone, Debug)]
pub struct OpenParameter {
    pub param_type: u8,
    pub param_length: u8,
    pub value: Vec<u8>
}

impl OpenParameter {
    fn decode(cur: &mut Cursor<Vec<u8>>) -> Result<(u8, Self)> {
        let param_type = cur.read_u8()?;
        let param_length = cur.read_u8()?;

        let mut value = vec![0u8; param_length as usize];
        cur.read_exact(&mut value)?;

        Ok((2 + param_length, OpenParameter { param_type, param_length, value }))
    }
}