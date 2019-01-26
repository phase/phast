use network::packet::*;
use network::types::*;

use std::any::Any;

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
    Any,
    /// Unknown
    None,
}

pub trait Protocol: Send + Sync {
    fn protocol_type(&self) -> ProtocolType;
    fn protocol_version(&self) -> i32;
    fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Packet>;
    fn write(&self, packet: Packet, bound: Bound) -> Option<Vec<u8>>;
}

#[macro_export]
macro_rules! protocol {
    ($protocol_name:ident, $protocol_type:expr, $protocol_version:expr, $($id:expr, $state:expr, $bound:expr, $packet:ident),*) => {
        #[allow(non_camel_case_types)]
        pub struct $protocol_name;

        #[allow(dead_code)]
        impl Protocol for $protocol_name {
            fn protocol_type(&self) -> ProtocolType {
                $protocol_type
            }

            fn protocol_version(&self) -> i32 {
                $protocol_version
            }

            fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Packet> {
                $(
                    if id == $id && state == $state && (bound == $bound || $bound == Bound::Any) {
                        let mut packet = $packet::default();
                        return match packet.read(bytes) {
                            true => Some(Packet::$packet(packet)),
                            false => None
                        }
                    }
                )*
                None
            }

            fn write(&self, packet: Packet, bound: Bound) -> Option<Vec<u8>> {
                $(
                    if $bound == bound || $bound == Bound::Any{
                        if let Packet::$packet(packet) = packet {
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

macro_rules! packet_registry {
    ($($package:ident $protocol:ident $packet_name:ident)*) => {
        #[derive(Debug)]
        pub enum Packet {
            $(
                $packet_name($package::$protocol::$packet_name),
            )*
        }

        impl PacketType for Packet {
            fn name(&self) -> &str {
                match self {
                    $(
                        Packet::$packet_name(packet) => packet.name(),
                    )*
                }
            }

            fn read(&mut self, bytes: Vec<u8>) -> bool {
                match self {
                    $(
                        Packet::$packet_name(packet) => packet.read(bytes),
                    )*
                }
            }

            fn write(&self) -> Vec<u8> {
                match self {
                    $(
                        Packet::$packet_name(packet) => packet.write(),
                    )*
                }
            }

            fn next_state(&self) -> Option<State> {
                match self {
                    $(
                        Packet::$packet_name(packet) => packet.next_state(),
                    )*
                }
            }
        }
    }
}

use protocol::bedrock::raknet::*;
use protocol::java::v1_12::*;

packet_registry! {
    // Bedrock Raknet
    bedrock raknet ConnectedPingPacket
    bedrock raknet UnconnectedPingPacket
    bedrock raknet UnconnectedPingOpenConnectionsPacket
    bedrock raknet ConnectedPongPacket
    bedrock raknet OpenConnectionRequest1Packet
    bedrock raknet OpenConnectionReply1Packet
    bedrock raknet OpenConnectionRequest2Packet
    bedrock raknet OpenConnectionReply2Packet
    bedrock raknet ConnectionRequestPacket
    bedrock raknet ConnectionRequestAcceptedPacket
    bedrock raknet NewIncomingConnectionPacket
    bedrock raknet NoFreeIncomingConnectionsPacket
    bedrock raknet DisconnectNotificationPacket
    bedrock raknet ConnectionBannedPacket
    bedrock raknet IncompatibleProtocolPacket
    bedrock raknet IpRecentlyConnectedPacket
    bedrock raknet UnconnectedPongPacket
    bedrock raknet NakPacket
    bedrock raknet AckPacket

    // 1.12
    // Handshake
    java v1_12 HandshakePacket
    // Status
    java v1_12 ResponsePacket
    java v1_12 PongPacket
    java v1_12 RequestPacket
    java v1_12 PingPacket
    // Login
    java v1_12 LoginStartPacket
    java v1_12 EncryptionResponsePacket
    java v1_12 DisconnectPacket
    java v1_12 EncryptionRequestPacket
    java v1_12 LoginSuccessPacket
    java v1_12 SetCompressionPacket
}
