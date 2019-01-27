use packet::*;
use protocol::*;

// Packets for Minecraft Java Edition Version 1.14 (protocol version 457)
// https://wiki.vg/Pre-release_protocol

protocol!("1.14", ProtocolJava_1_14, ProtocolEdition::JavaEdition, 457,
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
    0x25, JavaPlay, Clientbound, java v1_9 JoinGamePacket
);
