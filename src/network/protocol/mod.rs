pub mod bedrock;
pub mod java;

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

pub trait Protocol {
    fn read(&self, id: i32, bytes: Vec<u8>) -> Box<Packet>;
}

/*
#[macro_export]
macro_rules! protocol {

}
*/
