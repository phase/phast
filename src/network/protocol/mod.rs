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

//pub trait Protocol {
//    fn read(&self, id: i32, bytes: Vec<u8>) -> Box<Packet>;
//}

pub struct Protocol {
    pub protocol_type: ProtocolType,
    pub protocol_version: i32,
    pub lookup: Box<fn(i32, State, Bound, Vec<u8>) -> Option<Box<Packet>>>,
}

#[macro_export]
macro_rules! protocol {
    ($protocol_name:ident, $protocol_type:expr, $protocol_version:expr, $($id:expr, $state:pat, $bound:pat, $packet:ident),*) => {
        lazy_static! {
            pub static ref $protocol_name: Protocol = { Protocol {
                protocol_type: $protocol_type,
                protocol_version: $protocol_version,
                lookup: Box::new(|id, state, bound, bytes| {
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
                })
            }};
        }
    };
}

// These need to be defined after the macro
pub mod bedrock;
pub mod java;
