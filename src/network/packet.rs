#![macro_escape]
use network::connection;
use network::types::*;

pub trait Packet {
    fn read(&mut self, bytes: Vec<u8>) -> bool;
    fn write(&self) -> Vec<u8>;
}

/// Read type from bytes
pub trait ReadField {
    /// returns the type & the length to increment the index by
    fn read(bytes: &Vec<u8>, index: usize) -> Option<(Self, usize)> where Self: Sized;
}

/// Write type to bytes
pub trait WriteField where Self: Sized {
    // TODO: Vec is probably going to kill performance
    fn write(&self) -> Vec<u8>;
}

#[macro_export]
macro_rules! packet {
    ($packet_name:ident, $($field:ident: $t:ty),*) => {
        pub struct $packet_name {
            $(
                $field: $t,
            )*
        }

        impl Packet for $packet_name {
            fn read(&mut self, bytes: Vec<u8>) -> bool {
                let mut index = 0usize;
                $(
                    match <$t as ReadField>::read(&bytes, index) {
                        Some((value, length)) => {
                            self.$field = value;
                            index += length;
                        },
                        None => return false
                    }
                )*
                true
            }

            fn write(&self) -> Vec<u8> {
                let mut buf = Vec::<u8>::new();
                $(
                    buf.append(&mut self.$field.write());
                )*
                buf
            }
        }
    };
}
