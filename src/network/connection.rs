use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::sync::Arc;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::mem::transmute;
use std::any::Any;
use std::{thread, time};

use network;
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
    has_started_packet: bool,
}

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
            has_started_packet: false,
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

    // might need a mutex so we only handle one read at a time
    pub fn handle_read(&mut self, bytes: &mut Vec<u8>) -> Vec<Box<Packet>> {
        let mut packets = Vec::with_capacity(1);
        self.unprocessed_buffer.append(bytes);
        while self.unprocessed_buffer.len() > 0 && !self.has_started_packet {
            if !self.has_started_packet {
                self.has_started_packet = true;
                let result = self.start_packet_read();
                match result {
                    Some(packet) => {
                        self.has_started_packet = false;
                        self.unprocessed_buffer.clear();
                        packets.push(packet)
                    }
                    None => {
                        // if !result, we didn't read the full packet and we need to wait for more data
                        // to come in
                        self.has_started_packet = true;
                    }
                }
            } else {
                match self.start_packet_read() {
                    Some(packet) => packets.push(packet),
                    None => {}
                }
            }
        }
        if self.unprocessed_buffer.len() > 0 {
            println!("Unused bytes: {:X?}", self.unprocessed_buffer);
        }
        packets
    }

    pub fn start_packet_read(&mut self) -> Option<Box<Packet>> {
        let bytes = &self.unprocessed_buffer.clone();
        let mut index: usize = 0;

        if self.is_tcp() {
            // java edition
            let length = match network::read_varint(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    l
                }
                None => return None
            };

            if length == 1 {
                // XXX: this is a weird ping thing. it'll only send [1, 0]
                self.unprocessed_buffer = (&bytes[2..]).to_vec();
                return None;
            }

            if bytes.len() < (length as usize) {
                // we don't have enough data yet
                return None;
            }

            let mut id_length = 0;
            let id = match network::read_varint(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    id_length = v;
                    l
                }
                None => return None
            };

            // get a vec of just the packet's bytes
            let packet_bytes = (&bytes[index..((length as usize) + 1)]).to_vec();

            // read the packet from the protocol
            let packet = match self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                Some(packet) => packet,
                None => return None
            };

            index += length as usize - id_length;

            let remainder = &bytes[index..];
            self.unprocessed_buffer = remainder.to_vec();

            Some(packet)
        } else if self.is_udp() {
            // bedrock edition
            let remainder = &bytes[index..];
            let id = bytes[0] as i32;
            index += 1;

            // get a vec of just the packet's bytes
            // TODO: length of packet?
            let packet_bytes = (&bytes[index..]).to_vec();

            // read the packet from the protocol
            let packet = match self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                Some(packet) => packet,
                None => return None
            };

            // TODO: Not this hack. Maybe return the length of the packet from protocol#read ?
            let remainder = &bytes[index..(packet.write().len())];
            self.unprocessed_buffer = remainder.to_vec();

            Some(packet)
        } else {
            None
        }
    }

    /// Writes `bytes` to the connected client
    pub fn write(&mut self, bytes: &[u8]) {
        match self.socket {
            SocketWrapper::TCP(ref mut stream) => {
                stream.write(bytes);
            }
            SocketWrapper::UDP(ref mut socket) => {
                println!("UDP SEND: {:X?}", bytes);
                socket.send_to(bytes, self.address);
            }
        }
    }
}
