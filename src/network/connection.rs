use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::sync::Arc;

use network;
use network::protocol;
use network::protocol::java;

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
    pub protocol_type: protocol::ProtocolType,
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
            protocol_type: match socket {
                SocketWrapper::TCP(_) => protocol::ProtocolType::JavaEdition,
                SocketWrapper::UDP(_) => protocol::ProtocolType::BedrockEdition,
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
        match self.socket {
            SocketWrapper::TCP(_) => {
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

                // TODO: read packet based on protocol & id
                {
                    println!("Found TCP packet: length={} id={}", length, id);
                    let remainder = &bytes[index..];
                    println!("  packet_data: {:X?}", remainder);

                    if id == 0 && length > 0 {
                        println!("TCP C->S Handshake Packet");

                        let protocol_version = match network::read_varint(bytes, index) {
                            Some((l, v)) => {
                                index += v;
                                l
                            }
                            None => return false
                        };
                        println!("index {}/{}| protocol_version {}", index, length, protocol_version);

                        let address = match network::read_string(bytes, index) {
                            Some((s, v)) => {
                                index += v;
                                s
                            }
                            None => return false
                        };
                        println!("index {}/{}| address {}", index, length, address);

                        let port = match network::read_ushort(bytes, index) {
                            Some((s, v)) => {
                                index += v;
                                s
                            }
                            None => return false
                        };
                        println!("index {}/{}| port {}", index, length, port);

                        let next_state = match network::read_varint(bytes, index) {
                            Some((l, v)) => {
                                index += v;
                                l
                            }
                            None => return false
                        };
                        println!("index {}/{}| next_state {}", index, length, next_state);
                    }
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
            }
            SocketWrapper::UDP(_) => {
                // bedrock edition
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
                socket.send_to(bytes, self.address);
            }
        }
    }
}
