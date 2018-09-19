use network::packet::*;
use network::types::*;

use std::any::Any;
use std::mem;

#[derive(Debug, Eq, PartialEq)]
pub enum ProtocolType {
    JavaEdition,
    BedrockEdition,
}

/// Packets in different states have different id counters.
/// When a state changes, the id counter resets.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum State {
    /// first connecting to the server
    JavaHandshake,
    /// for the client's server list
    JavaStatus,
    /// authentication
    JavaLogin,
    /// in game
    JavaPlay,

    /// raknet packets that are offline
    BedrockRakNetOffline,
    /// raknet packets that are datagrams
    BedrockRakNet,
    /// Minecraft protocol
    BedrockMinecraft,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Bound {
    /// Going to the Client
    Clientbound,
    /// Going to the Server
    Serverbound,
    /// Direction doesn't matter
    None,
}

pub trait Protocol: Send + Sync {
    fn protocol_type(&self) -> ProtocolType;
    fn protocol_version(&self) -> i32;
    fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Box<Packet>>;
    fn write(&self, packet: Box<Packet>, bound: Bound) -> Option<Vec<u8>>;
}

#[macro_export]
macro_rules! protocol {
    ($protocol_name:ident, $protocol_type:expr, $protocol_version:expr, $($id:expr, $state:pat, $bound:pat, $packet:ident),*) => {
        pub struct $protocol_name;

        impl Protocol for $protocol_name {
            fn protocol_type(&self) -> ProtocolType {
                $protocol_type
            }

            fn protocol_version(&self) -> i32 {
                $protocol_version
            }

            fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Box<Packet>> {
                let mut packet: Option<Box<Packet>> = match (id, state, bound) {
                    $(
                        ($id, $state, $bound) => Some(Box::new($packet::default())),
                    )*
                    _ => None
                };
                match packet {
                    Some(mut packet) => match packet.read(bytes) {
                        true => Some(packet),
                        false => None
                    },
                    None => None
                }
            }

            fn write(&self, packet: Box<Packet>, bound: Bound) -> Option<Vec<u8>> {
                let any: Box<Any> = packet.as_any();
                $(
                    if let $bound = bound {
                        if let Some(packet) = any.downcast_ref::<$packet>() {
                            let id = $id;
                            if $protocol_type == ProtocolType::JavaEdition {
                                let mut buf = VarInt(id).write();
                                buf.append(&mut packet.write());
                                // prepend length as a varint
                                let mut full = VarInt(buf.len() as i32).write();
                                full.append(&mut buf);
                                return Some(full);
                            } else if $protocol_type == ProtocolType::BedrockEdition {
                                let mut buf = vec![id as u8];
                                buf.append(&mut packet.write());
                                return Some(buf);
                            } else {
                                return None;
                            }
                        }
                    }
                )*
                None
            }
        }
    };
}

// These need to be defined after the macro
pub mod bedrock;
pub mod java;
