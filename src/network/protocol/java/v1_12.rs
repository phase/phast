use packet::*;
use types::*;
use protocol::*;

// Packets for Minecraft Java Edition Version 1.12.2 (protocol version 340)

protocol!(ProtocolJava_1_12, ProtocolType::JavaEdition, 340,
    // Handshake
    0, State::JavaHandshake, Bound::Serverbound, HandshakePacket,
    // Status
    0, State::JavaStatus, Bound::Clientbound, ResponsePacket,
    1, State::JavaStatus, Bound::Clientbound, PongPacket,
    0, State::JavaStatus, Bound::Serverbound, RequestPacket,
    1, State::JavaStatus, Bound::Serverbound, PingPacket,
    // Login
    0, State::JavaLogin, Bound::Serverbound, LoginStartPacket,
    1, State::JavaLogin, Bound::Serverbound, EncryptionResponsePacket,
    0, State::JavaLogin, Bound::Clientbound, DisconnectPacket,
    1, State::JavaLogin, Bound::Clientbound, EncryptionRequestPacket,
    2, State::JavaLogin, Bound::Clientbound, LoginSuccessPacket,
    3, State::JavaLogin, Bound::Clientbound, SetCompressionPacket
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
