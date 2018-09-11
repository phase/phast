use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::sync::Arc;
use std::mem::transmute;
use std::any::Any;

use network;
use network::packet;
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
    pub fn handle_read(&mut self, bytes: &mut Vec<u8>) {
        self.unprocessed_buffer.append(bytes);
        if !self.has_started_packet {
            self.has_started_packet = true;
            let result = self.start_packet_read();
            // if !result, we didn't read the full packet and we need to wait for more data
            // to come in
            if result {
                self.has_started_packet = false;
                self.unprocessed_buffer.clear()
            }
        } else {
            self.start_packet_read();
        }
    }

    pub fn start_packet_read(&mut self) -> bool {
        let bytes = &self.unprocessed_buffer.clone();
        let mut index: usize = 0;

        if self.is_tcp() {
            // java edition
            let length = match network::read_varint(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    l
                }
                None => return false
            };

            if bytes.len() < (length as usize) {
                // we don't have enough data yet
                return false;
            }

            let id = match network::read_varint(bytes, index) {
                Some((l, v)) => {
                    index += v;
                    l
                }
                None => return false
            };

            // get a vec of just the packet's bytes
            let packet_bytes = (&bytes[index..((length as usize) + 1)]).to_vec();

            // read the packet from the protocol
            if let Some(packet) = self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                let any: Box<Any> = packet.as_any();
                if let Some(handshake) = any.downcast_ref::<v1_12::HandshakePacket>() {
                    println!("GOT HANDSHAKE PACKET!!! {:?}", handshake);
                }
            } else {
                return false;
            }

            index = length as usize + 1;

            if bytes.len() > index {
                let remainder = &bytes[index..];
                println!("  remainder: {:X?}", remainder);
                self.unprocessed_buffer = remainder.to_vec();
                return false;
            } else {
                return true;
            }
        } else if self.is_udp() {
            // bedrock edition
            let remainder = &bytes[index..];
            println!("  Bytes: {:X?}", remainder);
            let id = bytes[0] as i32;
            index += 1;

            // get a vec of just the packet's bytes
            // TODO: length of packet?
            let packet_bytes = (&bytes[index..]).to_vec();

            // read the packet from the protocol
            if let Some(packet) = self.protocol.read(id, self.protocol_state, Bound::Serverbound, packet_bytes) {
                let any: Box<Any> = packet.as_any();
                if let Some(handshake) = any.downcast_ref::<raknet::UnconnectedPingPacket>() {
                    println!("GOT UNCONNECTED_PING PACKET!!! {:?}", handshake);
                }
            } else {
                return false;
            }

            if bytes.len() > index {
                let remainder = &bytes[index..];
                println!("  remainder: {:X?}", remainder);
                self.unprocessed_buffer = remainder.to_vec();
//                    return false;
            } else {
                return true;
            }
        }

        true
    }

    pub fn read(&mut self) -> usize {
        let mut buf = vec![0; 64];
        let mut length = 0;
        match self.socket {
            SocketWrapper::TCP(ref mut stream) => {
                length = stream.read(&mut buf).unwrap_or(0);
            }
            SocketWrapper::UDP(ref mut socket) => {
                // I don't think there's a way to explicitly read from a UDP address
            }
        };
        if length > 0 {
            self.handle_read(&mut buf[..length].to_vec());
        }

        length
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
