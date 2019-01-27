use packet::*;
use protocol::*;

// RakNet protocol
protocol!("RakNet", ProtocolBedrockRakNet, ProtocolEdition::BedrockEdition, 9,
    0x00, BedrockRakNetOffline, Serverbound, bedrock raknet ConnectedPingPacket,
    0x01, BedrockRakNetOffline, Serverbound, bedrock raknet UnconnectedPingPacket,
    0x02, BedrockRakNetOffline, Serverbound, bedrock raknet UnconnectedPingOpenConnectionsPacket,
    0x03, BedrockRakNetOffline, Clientbound, bedrock raknet ConnectedPongPacket,
    0x05, BedrockRakNetOffline, Serverbound, bedrock raknet OpenConnectionRequest1Packet,
    0x06, BedrockRakNetOffline, Clientbound, bedrock raknet OpenConnectionReply1Packet,
    0x07, BedrockRakNetOffline, Serverbound, bedrock raknet OpenConnectionRequest2Packet,
    0x08, BedrockRakNetOffline, Clientbound, bedrock raknet OpenConnectionReply2Packet,
    0x09, BedrockRakNet, Serverbound, bedrock raknet ConnectionRequestPacket,
    0x10, BedrockRakNet, Clientbound, bedrock raknet ConnectionRequestAcceptedPacket,
    0x13, BedrockRakNet, Serverbound, bedrock raknet NewIncomingConnectionPacket,
    0x14, BedrockRakNet, None, bedrock raknet NoFreeIncomingConnectionsPacket,
    0x15, BedrockRakNet, None, bedrock raknet DisconnectNotificationPacket,
    0x17, BedrockRakNet, None, bedrock raknet ConnectionBannedPacket,
    0x19, BedrockRakNet, None, bedrock raknet IncompatibleProtocolPacket,
    0x1a, BedrockRakNet, None, bedrock raknet IpRecentlyConnectedPacket,
    0x1c, BedrockRakNetOffline, Clientbound, bedrock raknet UnconnectedPongPacket,
    0xa0, BedrockRakNetOffline, Any, bedrock raknet NakPacket,
    0xc0, BedrockRakNetOffline, Any, bedrock raknet AckPacket
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
