#[macro_use]
use packet::*;
use types::*;

// Packets for Minecraft Java Edition Version 1.12.2 (protocol version 340)

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
