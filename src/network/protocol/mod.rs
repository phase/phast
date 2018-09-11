use network::packet::*;

pub enum ProtocolType {
    JavaEdition,
    BedrockEdition,
}

/// Packets in different states have different id counters.
/// When a state changes, the id counter resets.
pub enum State {
    /// first connecting to the server
    JavaHandshake,
    /// for the client's server list
    JavaStatus,
    /// authentication
    JavaLogin,
    /// in game
    JavaPlay,

    /// raknet protocol
    BedrockRakNet,
    /// Minecraft protocol
    BedrockMinecraft,
}

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
                        ($id, $state, $bound) => Some(Box::new($packet::new())),
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
        }
    };
}

// These need to be defined after the macro
pub mod bedrock;
pub mod java;
