use std::mem;

use network::packet::*;
use network::protocol::bedrock;

#[derive(Copy, Clone, Default, Debug)]
pub struct VarInt(pub i32);

/// Used by the Java protocol
#[derive(Clone, Default, Debug)]
pub struct VarIntLengthPrefixedString(pub String);

/// Used by the Bedrock protocol
#[derive(Clone, Default, Debug)]
pub struct ShortLengthPrefixedString(pub String);

#[derive(Clone, Default, Debug)]
pub struct RakNetMagic(pub [u8; 16]);
pub const RAKNET_MAGIC: RakNetMagic = RakNetMagic(bedrock::MAGIC);

impl ReadField for u8 {
    fn read(bytes: &Vec<u8>, index: usize) -> Option<(u8, usize)> {
        if bytes.len() <= index {
            None
        } else {
            Some((*bytes.get(index).unwrap(), 1))
        }
    }
}

impl WriteField for u8 {
    fn write(&self) -> Vec<u8> {
        vec![self.clone()]
    }
}

impl ReadField for VarInt {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(VarInt, usize)> {
        let mut result = 0;
        let mut bytes_used: usize = 0;

        let msb: u8 = 0b10000000;
        let mask: u8 = !msb;

        for i in 0..5 {
            let read = match buf.get(index) {
                Some(r) => r,
                None => {
                    println!("read_varint couldn't find byte {}/{}", index, buf.len());
                    return None;
                }
            };
            bytes_used += 1;
            index += 1;

            result |= ((read & mask) as i32) << (7 * i);

            /* The last (5th) byte is only allowed to have the 4 LSB set */
            if i == 4 && (read & 0xf0 != 0) {
                println!("read_varint is too long, last byte: {}", read);
                return None;
            }

            if (read & msb) == 0 {
                return Some((VarInt(result), bytes_used));
            }
        }

        println!("read_varint reached end of loop, which should not be possible");
        None
    }
}

impl WriteField for VarInt {
    fn write(&self) -> Vec<u8> {
        /* Define some helpful values for dealing with varints */
        let msb: u8 = 0b10000000;
        let mask: u32 = !(msb as u32);

        /* Make the value unsigned to avoid weird signed behavior when bit-shifting */
        let mut val = self.0 as u32;

        let mut vec: Vec<u8> = Vec::new();
        for _ in 0..5 {
            /* Get the last 7 bits and cast to an u8.
             * Also right-shift the value to advance further. */
            let mut tmp = (val & mask) as u8;
            val >>= 7;

            /* If there's still something to write, set the most significant bit and continue */
            if val != 0 {
                tmp |= msb;
                vec.push(tmp);
            } else {
                vec.push(tmp);
                break;
            }
        }

        vec
    }
}

impl ReadField for VarIntLengthPrefixedString {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(VarIntLengthPrefixedString, usize)> {
        let mut varint_size = 0;
        let length = match <VarInt as ReadField>::read(buf, index) {
            Some((l, v)) => {
                varint_size = v;
                index += v;
                l.0 as usize
            }
            None => return None
        };

        if length > (1 << 16) {
            println!("read_string refusing to read string due to its length");
            return None;
        }

        if buf.len() < index + length {
            println!("read_string expected a string with length {}", length);
            return None;
        }


        Some((
            VarIntLengthPrefixedString(
                String::from_utf8((&buf[index..(index + length)]).to_vec()).unwrap()
            ),
            length + varint_size
        ))
    }
}

impl WriteField for VarIntLengthPrefixedString {
    fn write(&self) -> Vec<u8> {
        let s = &self.0;
        let length = s.len();
        let mut buf = VarInt(length as i32).write();
        // TODO: Remove clone
        buf.append(&mut s.clone().into_bytes());
        buf
    }
}

impl ReadField for ShortLengthPrefixedString {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(ShortLengthPrefixedString, usize)> {
        let length = match <u16 as ReadField>::read(buf, index) {
            Some((l, v)) => {
                index += v;
                l as usize
            }
            None => return None
        };

        if buf.len() < index + length {
            // string isn't there!
            return None;
        }

        // plus 2 at the end for the size of the short
        Some((
            ShortLengthPrefixedString(
                String::from_utf8((&buf[index..(index + length)]).to_vec()).unwrap()
            ),
            length + 2
        ))
    }
}

impl WriteField for ShortLengthPrefixedString {
    fn write(&self) -> Vec<u8> {
        let s = &self.0;
        // TODO: Remove clone
        let length = s.clone().len() as u16;

        let mut buf = vec![
            ((length & 0xFF00) >> 8) as u8,
            (length & 0xFF) as u8
        ];

        buf.append(&mut s.clone().into_bytes());
        buf
    }
}

impl ReadField for u16 {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(u16, usize)> {
        if buf.len() < index + 2 {
            return None;
        }

        let mut us_bytes: [u8; 2] = Default::default();
        us_bytes.copy_from_slice(&buf[index..(index + 2)]);
        let mut s: u16 = 0;
        unsafe {
            // swap bytes
            s = mem::transmute([us_bytes[1], us_bytes[0]]);
        }

        Some((s, 2))
    }
}

impl WriteField for u16 {
    fn write(&self) -> Vec<u8> {
        vec![
            (self >> 8) as u8,
            (self & 0xFF) as u8,
        ]
    }
}

impl ReadField for u64 {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(u64, usize)> {
        if buf.len() < index + 2 {
            return None;
        }

        let mut b: [u8; 8] = Default::default();
        b.copy_from_slice(&buf[index..(index + 8)]);
        let mut s: u64 = 0;
        unsafe {
            // swap bytes
            s = mem::transmute([
                b[7], b[6], b[5], b[4],
                b[3], b[2], b[1], b[0]
            ]);
        }

        Some((s, 8))
    }
}

impl WriteField for u64 {
    fn write(&self) -> Vec<u8> {
        (unsafe { mem::transmute::<u64, [u8; 8]>(self.to_be()) })[..].to_vec()
    }
}

impl ReadField for RakNetMagic {
    fn read(buf: &Vec<u8>, mut index: usize) -> Option<(RakNetMagic, usize)> {
        // TODO: Validate
        if buf.len() < index + 16 {
            return None;
        }

        Some((RakNetMagic(bedrock::MAGIC), 16))
    }
}

impl WriteField for RakNetMagic {
    fn write(&self) -> Vec<u8> {
        bedrock::MAGIC.to_vec()
    }
}
