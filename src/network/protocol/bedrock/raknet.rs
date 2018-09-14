use packet::*;
use types::*;
use protocol::*;

// RakNet protocol
protocol!(ProtocolBedrockRakNet, ProtocolType::BedrockEdition, 9,
    0x00, State::BedrockRakNet, Bound::Serverbound, ConnectedPingPacket,
    0x01, State::BedrockRakNet, Bound::Serverbound, UnconnectedPingPacket,
    0x02, State::BedrockRakNet, Bound::Serverbound, UnconnectedPingOpenConnectionsPacket,
    0x03, State::BedrockRakNet, Bound::Clientbound, ConnectedPongPacket,
    0x05, State::BedrockRakNet, Bound::None, OpenConnectionRequest1Packet,
    0x06, State::BedrockRakNet, Bound::None, OpenConnectionReply1Packet,
    0x07, State::BedrockRakNet, Bound::None, OpenConnectionRequest2Packet,
    0x08, State::BedrockRakNet, Bound::None, OpenConnectionReply2Packet,
    0x09, State::BedrockRakNet, Bound::None, ConnectionRequestPacket,
    0x10, State::BedrockRakNet, Bound::None, ConnectionRequestAcceptedPacket,
    0x13, State::BedrockRakNet, Bound::None, NewIncomingConnectionPacket,
    0x14, State::BedrockRakNet, Bound::None, NoFreeIncomingConnectionsPacket,
    0x15, State::BedrockRakNet, Bound::None, DisconnectNotificationPacket,
    0x17, State::BedrockRakNet, Bound::None, ConnectionBannedPacket,
    0x19, State::BedrockRakNet, Bound::None, IncompatibleProtocolPacket,
    0x1a, State::BedrockRakNet, Bound::None, IpRecentlyConnectedPacket,
    0x1c, State::BedrockRakNet, Bound::Clientbound, UnconnectedPongPacket,
    0xa0, State::BedrockRakNet, Bound::None, NakPacket,
    0xc0, State::BedrockRakNet, Bound::None, AckPacket
);

packet!(ConnectedPingPacket,
    ping_time: u64
);

packet!(UnconnectedPingPacket,
    ping_time: u64,
    magic: RakNetMagic,
    guid: u64
);

packet!(UnconnectedPingOpenConnectionsPacket,
    ping_time: u64,
    magic: RakNetMagic,
    guid: u64
);

packet!(ConnectedPongPacket,
    ping_time: u64,
    pong_time: u64
);

packet!(OpenConnectionRequest1Packet,
    magic: RakNetMagic,
    protocol_version: u8,
    mtu: u16
);

packet!(OpenConnectionReply1Packet,
    magic: RakNetMagic,
    server_id: u64,
    security: u8,
    mtu_size: u16
);

packet!(OpenConnectionRequest2Packet,
    magic: RakNetMagic,
    // TODO: server_address: SockerAddr,
    mtu_size: u16,
    client_id: u64
);

packet!(OpenConnectionReply2Packet,
    magic: RakNetMagic,
    server_id: u64,
    // TODO: client_address: SocketAddr,
    mtu_size: u16,
    security: u8
);

packet!(ConnectionRequestPacket,
    client_guid: u64,
    timestamp: u64,
    security: u8
);

packet!(ConnectionRequestAcceptedPacket,
    // TODO: system_address: SocketAddr,
    system_index: u16,
    // TODO: system_addresses: Vec<SocketAddr>,?
    incoming_timestamp: u64,
    system_timestamp: u64
);

packet!(NewIncomingConnectionPacket,
    // TODO: client_address: system_address,
    // TODO: system_addresses: Vec<SocketAddr>,?
    client_timestamp: u64,
    server_timestamp: u64
);

packet!(NoFreeIncomingConnectionsPacket,
    magic: RakNetMagic,
    server_id: u64
);

packet!(DisconnectNotificationPacket,
    // None
);

packet!(ConnectionBannedPacket,
    magic: RakNetMagic,
    server_id: u64
);

packet!(IncompatibleProtocolPacket,
    raknet_version: u8,
    magic: RakNetMagic,
    server_id: u64
);

packet!(IpRecentlyConnectedPacket,
    // None
);

packet!(UnconnectedPongPacket,
    ping_time: u64,
    guid: u64,
    magic: RakNetMagic,
    // MCPE;motd;protocol version;version string (can be anything?);players online;max players;server guid;motd line two?;Survival (was in MiNet);
    motd: ShortLengthPrefixedString
);

packet!(NakPacket,
    // TODO: short prefix "int range"
);

packet!(AckPacket,
    // TODO: short prefix "int range"
);
