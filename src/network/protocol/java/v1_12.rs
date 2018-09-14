use packet::*;
use types::*;
use protocol::*;

// Packets for Minecraft Java Edition Version 1.12.2 (protocol version 340)

protocol!(ProtocolJava_1_12, ProtocolType::JavaEdition, 340,
    0, State::JavaHandshake, Bound::Serverbound, HandshakePacket,
    0, State::JavaStatus, Bound::Clientbound, ResponsePacket,
    1, State::JavaStatus, Bound::Clientbound, PongPacket,
    0, State::JavaStatus, Bound::Serverbound, RequestPacket,
    1, State::JavaStatus, Bound::Serverbound, PingPacket
);

// Handshake C->S

packet!(HandshakePacket,
    protocol_version: VarInt,
    server_address: VarIntLengthPrefixedString,
    server_port: u16,
    next_state: VarInt
);

// Status S->C

packet!(ResponsePacket,
    response: VarIntLengthPrefixedString
);

packet!(PongPacket,
    payload: u64
);

// Status C->S

packet!(RequestPacket,
    // no fields
);

packet!(PingPacket,
    payload: u64
);
