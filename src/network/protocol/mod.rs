use network::packet::*;
use network::types::*;

#[derive(Debug, Eq, PartialEq)]
pub enum ProtocolEdition {
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

pub trait ProtocolType: Send + Sync {
    fn name(&self) -> &str;
    fn protocol_type(&self) -> ProtocolEdition;
    fn protocol_version(&self) -> i32;
    fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Packet>;
    fn write(&self, packet: Packet, bound: Bound) -> Option<Vec<u8>>;
}

#[macro_export]
macro_rules! protocol {
    ($pretty_name:expr, $protocol_name:ident, $protocol_type:expr, $protocol_version:expr,
        $($id:expr, $state:ident, $bound:ident, $package:ident $protocol:ident $packet_name:ident),*) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Eq, PartialEq, Clone, Copy)]
        pub struct $protocol_name;
        paste::item! {
            pub const [<$protocol_name _Id>]: i32 = $protocol_version;
        }

        #[allow(dead_code)]
        impl ProtocolType for $protocol_name {
            fn name(&self) -> &str {
                $pretty_name
            }

            fn protocol_type(&self) -> ProtocolEdition {
                $protocol_type
            }

            fn protocol_version(&self) -> i32 {
                $protocol_version
            }

            fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Packet> {
                $(
                    if id == $id && state == State::$state && (bound == Bound::$bound || Bound::$bound == Bound::Any) {
                        let mut packet = $package::$protocol::$packet_name::default();
                        paste::item! {
                            return match packet.read(bytes) {
                                true =>  Some(Packet::[<$package _ $protocol _ $packet_name>](packet)),
                                false => None
                            }
                        }
                    }
                )*
                None
            }

            fn write(&self, packet: Packet, bound: Bound) -> Option<Vec<u8>> {
                $(
                    if Bound::$bound == bound || Bound::$bound == Bound::Any{
                        paste::item! {
                            if let Packet::[<$package _ $protocol _ $packet_name>](packet) = packet {
                                let id = $id;
                                if $protocol_type == ProtocolEdition::JavaEdition {
                                    let mut buf = VarInt(id).write();
                                    buf.append(&mut packet.write());
                                    // prepend length as a varint
                                    let mut full = VarInt(buf.len() as i32).write();
                                    full.append(&mut buf);
                                    return Some(full);
                                } else if $protocol_type == ProtocolEdition::BedrockEdition {
                                    let mut buf = vec![id as u8];
                                    buf.append(&mut packet.write());
                                    return Some(buf);
                                } else {
                                    return None;
                                }
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

macro_rules! protocol_registry {
    ($($package:ident $version:ident $protocol_name:ident)*) => {
        #[derive(Debug, Eq, PartialEq, Clone, Copy)]
        pub enum Protocol {
            $($protocol_name($package::$version::$protocol_name),)*
        }

        impl ProtocolType for Protocol {
            fn name(&self) -> &str {
                match self {
                    $(Protocol::$protocol_name(protocol) => protocol.name(),)*
                }
            }

            fn protocol_type(&self) -> ProtocolEdition {
                match self {
                    $(Protocol::$protocol_name(protocol) => protocol.protocol_type(),)*
                }
            }

            fn protocol_version(&self) -> i32 {
                match self {
                    $(Protocol::$protocol_name(protocol) => protocol.protocol_version(),)*
                }
            }

            fn read(&self, id: i32, state: State, bound: Bound, bytes: Vec<u8>) -> Option<Packet> {
                match self {
                    $(Protocol::$protocol_name(protocol) => protocol.read(id, state, bound, bytes),)*
                }
            }

            fn write(&self, packet: Packet, bound: Bound) -> Option<Vec<u8>> {
                match self {
                    $(Protocol::$protocol_name(protocol) => protocol.write(packet, bound),)*
                }
            }
        }

        pub fn get_protocol(version: i32) -> Option<Protocol> {
            $(
                let protocol_id = paste::expr! { $package::$version::[<$protocol_name _Id>] };
                if version == protocol_id {
                    return Some(Protocol::$protocol_name($package::$version::$protocol_name));
                }
            )*
            None
        }
    }
}

macro_rules! packet_registry {
    ($($package:ident $protocol:ident $packet_name:ident)*) => {
        paste::item! {
            #[derive(Debug)]
            #[allow(non_camel_case_types)]
            pub enum Packet {
                $([<$package _ $protocol _ $packet_name>]($package::$protocol::$packet_name),)*
            }
        }
        paste::item! {
            impl PacketType for Packet {
                fn name(&self) -> &str {
                    match self {
                        $(Packet::[<$package _ $protocol _ $packet_name>](packet) => packet.name(),)*
                    }
                }

                fn read(&mut self, bytes: Vec<u8>) -> bool {
                    match self {
                        $(Packet::[<$package _ $protocol _ $packet_name>](packet) => packet.read(bytes),)*
                    }
                }

                fn write(&self) -> Vec<u8> {
                    match self {
                        $(Packet::[<$package _ $protocol _ $packet_name>](packet) => packet.write(),)*
                    }
                }

                fn next_state(&self) -> Option<State> {
                    match self {
                        $(Packet::[<$package _ $protocol _ $packet_name>](packet) => packet.next_state(),)*
                    }
                }
            }
        }
    }
}

use protocol::bedrock::raknet::*;
use protocol::java::*;

packet_registry! {
    // BEDROCK PACKETS \\

    // Raknet
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

    // JAVA PACKETS \\

    // 1.7
    // Handshake
    java v1_7 HandshakePacket
    // Status
    java v1_7 ResponsePacket
    java v1_7 PongPacket
    java v1_7 RequestPacket
    java v1_7 PingPacket
    // Login
    java v1_7 LoginStartPacket
    java v1_7 EncryptionResponsePacket
    java v1_7 DisconnectPacket
    java v1_7 EncryptionRequestPacket
    java v1_7 LoginSuccessPacket
    java v1_7 SetCompressionPacket
    // Play
    java v1_7 KeepAlivePacket
    java v1_7 JoinGamePacket

    // 1.8
    java v1_8 JoinGamePacket

    // 1.9
    java v1_9 JoinGamePacket
}

protocol_registry!(
    bedrock raknet ProtocolBedrockRakNet
    java v1_7 ProtocolJava_1_7
    java v1_8 ProtocolJava_1_8
    java v1_9 ProtocolJava_1_9
    java v1_10 ProtocolJava_1_10
    java v1_11 ProtocolJava_1_11
    java v1_12 ProtocolJava_1_12
    java v1_13 ProtocolJava_1_13
    java v1_14 ProtocolJava_1_14
);
