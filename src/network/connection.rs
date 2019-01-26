use std::net::{TcpStream, UdpSocket, SocketAddr};
use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};

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
    datagram_sequence_id: AtomicIsize,
}

enum PacketResult {
    CompletePacket(Packet),
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
                SocketWrapper::UDP(_) => State::BedrockRakNetOffline,
            },
            protocol: match socket {
                SocketWrapper::TCP(_) => Box::new(v1_12::ProtocolJava_1_12),
                SocketWrapper::UDP(_) => Box::new(raknet::ProtocolBedrockRakNet),
            },
            socket,
            unprocessed_buffer: vec![],
            datagram_sequence_id: AtomicIsize::new(42),
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
    pub fn handle_read(&mut self, bytes: &mut Vec<u8>) -> Vec<Packet> {
        let mut packets: Vec<Packet> = Vec::with_capacity(1);
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

            let id_length;
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
            let id = bytes[index] as i32;
            index += 1;

            if id & 0x80 == 0x80 && id != 0xA0 && id != 0xC0 {
                println!("packet is datagram {:#b}\n  SPECIAL: {:X?}", id, bytes);
                // is datagram

                // header parts
                let is_ack = id & 0x40 == 0x40;
                let has_ba_and_as = is_ack && (id & 0x10 == 0x10);
                let is_nack = !is_ack && (id & 0x20 == 0x20);
                let is_packet_pair = !is_ack && !is_nack && (id & 0x10 == 0x10);
                let is_continuous_send = !is_ack && !is_nack && (id & 0x8 == 0x8);
                let needs_ba_and_as = !is_ack && !is_nack && (id & 0x4 == 0x4);

                if bytes.len() < index + 3 { return NeedMoreData; }
                let sequence_number = [bytes[index], bytes[index + 1], bytes[index + 2]];
                index += 3;

                let flags = bytes[index];
                index += 1;
                let reliability = (flags & 0xE0) >> 5;
                println!("flags {:#b} {}", reliability, reliability);
                let split = flags & 0x10 == 0x10;

                let length = match <u16 as ReadField>::read(bytes, index) {
                    Some((l, v)) => {
                        index += v;
                        // this short is actually the data *bit* length
                        (l / 8) as usize
                    }
                    None => return NeedMoreData
                };

                // TODO: Abstract this out of here

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

                let (order_index, order_channel) = match reliability {
                    1 | 3 | 4 | 7 => {
                        let s = [bytes[index], bytes[index + 1], bytes[index + 2]];
                        index += 3;
                        let c = bytes[index];
                        index += 1;
                        (s, c)
                    }
                    _ => ([0, 0, 0], 0)
                };

                let (packet_count, packet_id, packet_index) = if split {
                    println!("This packet was split!");
                    let packet_count = match <u32 as ReadField>::read(bytes, index) {
                        Some((l, v)) => {
                            index += v;
                            l
                        }
                        None => return NeedMoreData
                    };
                    let packet_id = match <u16 as ReadField>::read(bytes, index) {
                        Some((l, v)) => {
                            index += v;
                            l
                        }
                        None => return NeedMoreData
                    };
                    let packet_index = match <u32 as ReadField>::read(bytes, index) {
                        Some((l, v)) => {
                            index += v;
                            l
                        }
                        None => return NeedMoreData
                    };
                    (packet_count, packet_id, packet_index)
                } else {
                    (0, 0, 0)
                };

                let id = bytes[index] as i32;
                index += 1;

                if index + length > bytes.len() {
                    println!("need {}/{} ({:X?}/{:X?}) bytes!!", length, bytes.len(), length, bytes.len());
                    return NeedMoreData;
                }

                let packet_bytes = (&bytes[index..(index + length - 1)]).to_vec();
                println!("ID: {} LENGTH: {}\nBYTES:{:X?}", id, length, packet_bytes);

                let packet = match self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                    Some(packet) => packet,
                    None => return NeedMoreData
                };

                let remainder = &bytes[(index + length - 1)..];
                self.unprocessed_buffer = remainder.to_vec();

                CompletePacket(packet)
            } else {
                // get a vec of just the packet's bytes
                let packet_bytes = (&bytes[index..]).to_vec();

                // read the packet from the protocol
                let packet = match self.protocol.read(id, State::BedrockRakNetOffline, Bound::Serverbound, packet_bytes) {
                    Some(packet) => packet,
                    None => return NeedMoreData
                };

                // TODO: Not this hack. Maybe return the length of the packet from protocol#read ?
                let remainder = &bytes[(packet.write().len() + 1)..];
                self.unprocessed_buffer = remainder.to_vec();

                CompletePacket(packet)
            }
        } else {
            NeedMoreData
        }
    }

    pub fn send_packet(&mut self, packet: Packet) {
        match self.protocol.write(packet, Bound::Clientbound) {
            Some(mut bytes) => {
                if self.protocol_state == State::BedrockRakNet {
                    // online raknet packets need a special header
                    let len = (bytes.len() * 8) as u16;

                    let d = self.datagram_sequence_id.fetch_add(1, Ordering::SeqCst);
                    let mut header = vec![
                        0x84u8, // todo: real raknet header
                        (0xFF & d) as u8,
                        (0xFF & (d >> 8)) as u8,
                        (0xFF & (d >> 16)) as u8,
                        0x40,
                        ((len >> 8) & 0xFF) as u8,
                        (len & 0xFF) as u8,
                        0, 0, 0
                    ];
                    header.append(&mut bytes);
                    self.write(header.as_slice());
                } else {
                    self.write(bytes.as_slice());
                }
            }
            None => {}
        }
    }

    /// Writes `bytes` to the connected client
    pub fn write(&mut self, bytes: &[u8]) {
        match self.socket {
            SocketWrapper::TCP(ref mut stream) => {
                stream.write(bytes).unwrap();
            }
            SocketWrapper::UDP(ref mut socket) => {
                socket.send_to(bytes, self.address).unwrap();
            }
        }
    }
}
