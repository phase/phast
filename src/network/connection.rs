use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};

use network::protocol;

/// Used to send data back to the client
pub enum SocketWrapper {
    TCP(TcpStream),
    UDP(UdpSocket),
}

/// A connection with a client
/// `unprocessed_buffer` will contain any data sent from the connection that needs to be processed
pub struct Connection {
    pub address: SocketAddr,
    pub socket: SocketWrapper,
    pub protocol_type: protocol::ProtocolType,
    unprocessed_buffer: Vec<u8>,
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
        }
    }

    pub fn handle_read(&self, bytes: &[u8]) {}

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
