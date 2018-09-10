pub mod packet;
pub mod connection;
pub mod protocol;

use std::mem;
use std::net::SocketAddr;
use std::collections::HashMap;

use concurrent_hashmap::*;

pub struct ConnectionManager {
    pub connections: ConcHashMap<SocketAddr, connection::Connection>
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        ConnectionManager {
            connections: ConcHashMap::<SocketAddr, connection::Connection>::new()
        }
    }
}

pub fn read_varint(buf: &Vec<u8>, mut index: usize) -> Option<(i32, usize)> {
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
            return Some((result, bytes_used));
        }
    }

    println!("read_varint reached end of loop, which should not be possible");
    None
}

pub fn read_string(buf: &Vec<u8>, mut index: usize) -> Option<(String, usize)> {
    let length = match read_varint(buf, index) {
        Some((l, v)) => {
            index += v;
            l as usize
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

    Some((String::from_utf8((&buf[index..(index + length)]).to_vec()).unwrap(), length + 1))
}

pub fn read_ushort(buf: &Vec<u8>, mut index: usize) -> Option<(u16, usize)> {
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
