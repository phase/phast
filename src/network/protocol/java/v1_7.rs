use packet::*;
use protocol::*;

// Packets for Minecraft Java Edition Version 1.7.10 (protocol version 5)
// https://wiki.vg/index.php?title=Protocol&oldid=6003

protocol!("1.7", ProtocolJava_1_7, ProtocolEdition::JavaEdition, 5,
    // Handshake
    0x00, JavaHandshake, Serverbound, java v1_7 HandshakePacket,
    // Status
    0x00, JavaStatus, Clientbound, java v1_7 ResponsePacket,
    0x01, JavaStatus, Clientbound, java v1_7 PongPacket,
    0x00, JavaStatus, Serverbound, java v1_7 RequestPacket,
    0x01, JavaStatus, Serverbound, java v1_7 PingPacket,
    // Login
    0x00, JavaLogin, Serverbound, java v1_7 LoginStartPacket,
    0x01, JavaLogin, Serverbound, java v1_7 EncryptionResponsePacket,
    0x00, JavaLogin, Clientbound, java v1_7 DisconnectPacket,
    0x01, JavaLogin, Clientbound, java v1_7 EncryptionRequestPacket,
    0x02, JavaLogin, Clientbound, java v1_7 LoginSuccessPacket,
    0x03, JavaLogin, Clientbound, java v1_7 SetCompressionPacket,
    // Play
    0x00, JavaPlay, Clientbound, java v1_7 KeepAlivePacket,
    0x01, JavaPlay, Clientbound, java v1_7 JoinGamePacket
);

// Handshake C->S

packet!(HandshakePacket,
    protocol_version: VarInt,
    server_address: VarIntLengthPrefixedString,
    server_port: u16,
    next_state: VarInt // next state of the protocol
    ; |s: &HandshakePacket| {
        match s.next_state.0 {
            1 => Some(State::JavaStatus),
            2 => Some(State::JavaLogin),
            _ => None
        }
    }

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

// Login S->C

packet!(DisconnectPacket,
    reason: VarIntLengthPrefixedString
);

packet!(EncryptionRequestPacket,
    server_id: VarIntLengthPrefixedString,
    public_key: VarIntLengthPrefixedByteArray,
    verify_token: VarIntLengthPrefixedByteArray
);

packet!(LoginSuccessPacket, // switches connection state to Play
    uuid: VarIntLengthPrefixedString,
    username: VarIntLengthPrefixedString
);

packet!(SetCompressionPacket,
    threshold: VarInt
);

// Login C->S

packet!(LoginStartPacket,
    name: VarIntLengthPrefixedString
);

packet!(EncryptionResponsePacket,
    shared_secret: VarIntLengthPrefixedByteArray,
    verify_token: VarIntLengthPrefixedByteArray
);

// Play S->C

packet!(KeepAlivePacket,
    id: i32
);

packet!(JoinGamePacket,
    entity_id: i32,
    game_mode: u8,
    dimension: u8,
    difficulty: u8,
    max_players: u8,
    level_type: VarIntLengthPrefixedString
);
