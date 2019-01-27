use packet::*;
use protocol::*;

// Packets for Minecraft Java Edition Version 1.9.4 (protocol version 110)
// https://wiki.vg/index.php?title=Protocol&oldid=7959

protocol!("1.9", ProtocolJava_1_9, ProtocolEdition::JavaEdition, 110,
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
    0x23, JavaPlay, Clientbound, java v1_9 JoinGamePacket
);

// Play S->C

packet!(JoinGamePacket,
    entity_id: i32,
    game_mode: u8,
    dimension: i32,
    difficulty: u8,
    max_players: u8,
    level_type: VarIntLengthPrefixedString,
    reduced_debug: u8
);
