use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem::transmute;
use std::any::Any;
use std::{thread, time};

use network;
use network::types::*;
use network::packet::*;
use network::protocol::*;
use network::protocol::bedrock::*;
use network::protocol::java::*;

/// Used to send data back to the client
pub enum SocketWrapper {
    TCP(TcpStream),
    UDP(Arc<UdpSocket>),
}

/// A connection with a client
/// `unprocessed_buffer` will contain any data sent from the connection that needs to be processed
pub struct Connection {
    pub address: SocketAddr,
    pub socket: SocketWrapper,
    pub protocol_state: State,
    pub protocol: Box<Protocol>,
    // processing packets
    unprocessed_buffer: Vec<u8>,
}

enum PacketResult {
    CompletePacket(Box<Packet>),
    NeedMoreData,
    Skipped(usize),
}

use self::PacketResult::*;

impl Connection {
    /// Constructs a new Connection from an Address and a SocketWrapper.
    /// The caller should wrap their TCP/UDP connection in a SocketWrapper
    pub fn new(address: SocketAddr, socket: SocketWrapper) -> Connection {
        Connection {
            address,
            protocol_state: match socket {
                SocketWrapper::TCP(_) => State::JavaHandshake,
                SocketWrapper::UDP(_) => State::BedrockRakNet,
            },
            protocol: match socket {
                SocketWrapper::TCP(_) => Box::new(v1_12::ProtocolJava_1_12),
                SocketWrapper::UDP(_) => Box::new(raknet::ProtocolBedrockRakNet),
            },
            socket,
            unprocessed_buffer: vec![],
        }
    }

    pub fn is_tcp(&self) -> bool {
        match self.socket {
            SocketWrapper::TCP(_) => true,
            _ => false
        }
    }

    pub fn is_udp(&self) -> bool {
        match self.socket {
            SocketWrapper::UDP(_) => true,
            _ => false
        }
    }

    // might need a lock so we only handle one read at a time
    pub fn handle_read(&mut self, bytes: &mut Vec<u8>) -> Vec<Box<Packet>> {
        let mut packets: Vec<Box<Packet>> = Vec::with_capacity(1);
        self.unprocessed_buffer.append(bytes);
        let mut needs_more_data = false;
        while self.unprocessed_buffer.len() > 0 && !needs_more_data {
            match self.start_packet_read() {
                CompletePacket(packet) => {
                    // switch state depending on packet
                    // this is done here because there may be more bytes for us to read that
                    // are in a different state
                    if let Some(state) = packet.next_state() {
                        self.protocol_state = state;
                    }

                    packets.push(packet);
                }
                NeedMoreData => {
                    needs_more_data = true;
                }
                Skipped(amount) => {
                    self.unprocessed_buffer = (&self.unprocessed_buffer[amount..]).to_vec();
                }
            }
        }
        if self.unprocessed_buffer.len() > 0 {
            println!("Unused bytes: {:X?}", self.unprocessed_buffer);
        }
        packets
    }

    fn start_packet_read(&mut self) -> PacketResult {
        let bytes = &self.unprocessed_buffer.clone();
        let mut index: usize = 0;

        if self.is_tcp() {
            if bytes.len() >= 3 {
                let first = bytes[0];
                let second = bytes[1];
                let third = bytes[2];
                if first == 0xFE && second == 0x01 && third == 0xFA {
                    println!("Skipping legacy ping");
                    // XXX: Legacy Ping 1.6
                    index = 3;
                    for s in 0..2 {
                        let s1 = bytes[index];
                        index += 1;
                        let s2 = bytes[index];
                        index += 1;
                        let data_size: u16 = ((s1 as u16) << 8) | s2 as u16;
                        index += (data_size as i32 * (2 - s)) as usize;
                    }
                    return Skipped(index);
                }
            }

            // java edition
            let length = match <VarInt as ReadField>::read(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    l.0
                }
                None => return NeedMoreData
            };


            if bytes.len() < (length as usize) {
                // we don't have enough data yet
                return NeedMoreData;
            }

            let mut id_length = 0;
            let id = match <VarInt as ReadField>::read(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    id_length = v;
                    l.0
                }
                None => return NeedMoreData
            };

            // get a vec of just the packet's bytes
            let packet_bytes = (&bytes[index..((length as usize) + 1)]).to_vec();

            // read the packet from the protocol
            let packet = match self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                Some(packet) => packet,
                None => return NeedMoreData
            };

            index += length as usize - id_length;

            let remainder = &bytes[index..];
            self.unprocessed_buffer = remainder.to_vec();

            CompletePacket(packet)
        } else if self.is_udp() {
            // bedrock edition
            let remainder = &bytes[index..];
            let id = bytes[0] as i32;
            index += 1;

            if index & 0x80 == 0x80 {
                // is datagram

                // header parts
                let is_ack = index & 0x40 == 0x40;
                let has_ba_and_as = is_ack && (index & 0x10 == 0x10);
                let is_nack = !is_ack && (index & 0x20 == 0x20);
                let is_packet_pair = !is_ack && !is_nack && (index & 0x10 == 0x10);
                let is_continuous_send = !is_ack && !is_nack && (index & 0x8 == 0x8);
                let needs_ba_and_as = !is_ack && !is_nack && (index & 0x4 == 0x4);

                if bytes.len() < index + 3 { return NeedMoreData; }
                let sequence_number = [bytes[index], bytes[index + 1], bytes[index + 2]];
                index += 3;

                let flags = bytes[index];
                index += 1;
                let reliability = flags & 0xD0;
                let split = flags & 0x10 == 0x10;

                let length = match <u16 as ReadField>::read(bytes, index) {
                    Some((l, v)) => {
                        index += v;
                        l
                    }
                    None => return NeedMoreData
                };

                let reliability_message = match reliability {
                    2 | 3 | 4 => {
                        let s = [bytes[index], bytes[index + 1], bytes[index + 2]];
                        index += 3;
                        s
                    }
                    _ => [0, 0, 0]
                };

                let sequencing_index = match reliability {
                    1 | 4 => {
                        let s = [bytes[index], bytes[index + 1], bytes[index + 2]];
                        index += 3;
                        s
                    }
                    _ => [0, 0, 0]
                };

                // todo: finish this

                NeedMoreData
            } else {
                // get a vec of just the packet's bytes
                let packet_bytes = (&bytes[index..]).to_vec();

                // read the packet from the protocol
                let packet = match self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                    Some(packet) => packet,
                    None => return NeedMoreData
                };

                // TODO: Not this hack. Maybe return the length of the packet from protocol#read ?
                let remainder = &bytes[index..(packet.write().len())];
                self.unprocessed_buffer = remainder.to_vec();

                CompletePacket(packet)
            }
        } else {
            NeedMoreData
        }
    }

    pub fn send_packet(&mut self, packet: Box<Packet>) {
        match self.protocol.write(packet, Bound::Clientbound) {
            Some(bytes) => {
                self.write(bytes.as_slice());
            }
            None => {}
        }
    }

    /// Writes `bytes` to the connected client
    pub fn write(&mut self, bytes: &[u8]) {
        match self.socket {
            SocketWrapper::TCP(ref mut stream) => {
                stream.write(bytes);
            }
            SocketWrapper::UDP(ref mut socket) => {
                socket.send_to(bytes, self.address);
            }
        }
    }
}
