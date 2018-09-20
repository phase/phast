use packet::*;
use types::*;
use protocol::*;

// RakNet protocol
protocol!(ProtocolBedrockRakNet, ProtocolType::BedrockEdition, 9,
    0x00, State::BedrockRakNetOffline, Bound::Serverbound, ConnectedPingPacket,
    0x01, State::BedrockRakNetOffline, Bound::Serverbound, UnconnectedPingPacket,
    0x02, State::BedrockRakNetOffline, Bound::Serverbound, UnconnectedPingOpenConnectionsPacket,
    0x03, State::BedrockRakNetOffline, Bound::Clientbound, ConnectedPongPacket,
    0x05, State::BedrockRakNetOffline, Bound::Serverbound, OpenConnectionRequest1Packet,
    0x06, State::BedrockRakNetOffline, Bound::Clientbound, OpenConnectionReply1Packet,
    0x07, State::BedrockRakNetOffline, Bound::Serverbound, OpenConnectionRequest2Packet,
    0x08, State::BedrockRakNetOffline, Bound::Clientbound, OpenConnectionReply2Packet,
    0x09, State::BedrockRakNet, Bound::Serverbound, ConnectionRequestPacket,
    0x10, State::BedrockRakNet, Bound::Clientbound, ConnectionRequestAcceptedPacket,
    0x13, State::BedrockRakNet, Bound::Serverbound, NewIncomingConnectionPacket,
    0x14, State::BedrockRakNet, Bound::None, NoFreeIncomingConnectionsPacket,
    0x15, State::BedrockRakNet, Bound::None, DisconnectNotificationPacket,
    0x17, State::BedrockRakNet, Bound::None, ConnectionBannedPacket,
    0x19, State::BedrockRakNet, Bound::None, IncompatibleProtocolPacket,
    0x1a, State::BedrockRakNet, Bound::None, IpRecentlyConnectedPacket,
    0x1c, State::BedrockRakNetOffline, Bound::Clientbound, UnconnectedPongPacket,
    0xa0, State::BedrockRakNetOffline, Bound::Any, NakPacket,
    0xc0, State::BedrockRakNetOffline, Bound::Any, AckPacket
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
    mtu_data: FortySixZeros
);

packet!(OpenConnectionReply1Packet,
    magic: RakNetMagic,
    server_id: u64,
    security: u8,
    mtu_size: u16
);

packet!(OpenConnectionRequest2Packet,
    magic: RakNetMagic,
    server_address: Address,
    mtu_size: u16,
    client_id: u64
);

packet!(OpenConnectionReply2Packet,
    magic: RakNetMagic,
    server_id: u64,
    client_address: Address,
    mtu_size: u16,
    security: u8
);

packet!(ConnectionRequestPacket,
    client_guid: u64,
    timestamp: u64,
    security: u8
);

packet!(ConnectionRequestAcceptedPacket,
    system_address: Address,
    system_index: u16,
    system_addresses: Vec<Address>,
    incoming_timestamp: u64,
    system_timestamp: u64
);

packet!(NewIncomingConnectionPacket,
    client_address: Address,
    system_addresses: Vec<Address>,
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
    ids: IntRangeList
);

packet!(AckPacket,
    ids: IntRangeList
);
