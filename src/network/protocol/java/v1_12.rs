#[macro_use]
use packet::*;
use types::*;

packet!(Handshaking,
    protocol_version: VarInt,
    server_address: VarIntLengthPrefixedString,
    server_port: u16,
    next_state: VarInt
);
