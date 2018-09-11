use packet::*;
use types::*;
use protocol::*;

// RakNet protocol
protocol!(ProtocolBedrockRakNet, ProtocolType::BedrockEdition, 9,
    0x01, State::BedrockRakNet, Bound::Serverbound, UnconnectedPingPacket,
    0x1c, State::BedrockRakNet, Bound::Clientbound, UnconnectedPongPacket
);

packet!(UnconnectedPingPacket,
    ping_time: u64,
    magic: RakNetMagic,
    guid: u64
);

packet!(UnconnectedPongPacket,
    ping_time: u64,
    guid: u64,
    magic: RakNetMagic,
    motd: ShortLengthPrefixedString
);
