use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::thread;
use std::time::*;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use std::any::Any;

use network::*;
use network::types::*;
use network::packet::*;
use network::connection::*;

use network::protocol::*;
use network::protocol::java::*;
use network::protocol::bedrock::*;

pub struct Server {
    pub network_manager: NetworkManager,
    pub connection_manager: Arc<ConnectionManager>,
    pub threads: Vec<JoinHandle<()>>,
    // Packet Channel
    pub packet_sender: Sender<(SocketAddr, Box<Packet>)>,
    pub packet_receiver: Receiver<(SocketAddr, Box<Packet>)>,
}

impl Server {
    pub fn new() -> Self {
        let connection_manager = Arc::new(ConnectionManager::new());
        let (packet_sender, packet_receiver) = channel::<(SocketAddr, Box<Packet>)>();
        let network_manager = NetworkManager::new(connection_manager.clone(), packet_sender.clone());

        Self {
            network_manager,
            connection_manager,
            threads: Vec::with_capacity(4),
            packet_sender,
            packet_receiver,
        }
    }

    pub fn start(&mut self) {
        self.network_manager.start();
        let ticks_per_second = 20;
        let tick_time = Duration::from_millis(1000 / ticks_per_second);
        // Main Game Loop
        loop {
            let now = SystemTime::now();

            // tick
            self.handle_packets();

            match now.elapsed() {
                Ok(elapsed) => {
                    if elapsed < tick_time {
                        let sleep = tick_time - elapsed;
//                        println!("[Server]: Sleeping for {:?}", sleep);
                        thread::sleep(sleep);
                    } else {
                        println!("[Server]: WARNING: Game Loop took {:?}! Game Loop Thread will not sleep.", elapsed);
                    }
                }
                Err(_) => {}
            }
        }
    }

    pub fn join_network_threads(self) {
        self.network_manager.join();
    }

    pub fn handle_packets(&mut self) {
        let max_packets_to_read = 1000;

        let mut packets_left = true;
        let mut packets_read = 0;
        while packets_left || packets_read >= max_packets_to_read {
            match self.packet_receiver.try_recv() {
                Ok((address, packet)) => {
                    packets_read += 1;
                    println!("[Server]: Got {} from {}", packet.name(), address);
                    self.handle_packet(address, packet);
                }
                _ => {
                    packets_left = false;
                }
            }
        }
    }

    fn handle_packet(&mut self, address: SocketAddr, packet: Box<Packet>) {
        let any: Box<Any> = packet.as_any();
        if let Some(handshake) = any.downcast_ref::<v1_12::HandshakePacket>() {
            let h: &v1_12::HandshakePacket = handshake;
            let protocol_version = h.protocol_version.0;
            match h.next_state.0 {
                1 => {
                    // Server List Ping
                    let response_string = format!("{{\
                        \"version\": {{\
                            \"name\": \"1.12.2\",\
                            \"protocol\": {}\
                        }},\
                        \"players\": {{\
                            \"max\": 100,\
                            \"online\": 5,\
                            \"sample\": [\
                                {{\
                                    \"name\": \"phase\",\
                                    \"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"\
                                }}\
                            ]\
                        }},\
                        \"description\": {{\
                            \"text\": \"rserver test\"\
                        }}\
                    }}", protocol_version);

                    let response: Box<v1_12::ResponsePacket> = Box::new(v1_12::ResponsePacket::new(VarIntLengthPrefixedString(response_string.to_string())));

                    if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                        let mut connection = connection.get();
                        connection.protocol_state = State::JavaStatus;
                        connection.send_packet(response);
                    }
                }
                2 => {
                    // Login
                    if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                        let mut connection = connection.get();
                        connection.protocol_state = State::JavaLogin;
                    }
                }
                _ => {}
            }
        }
        if let Some(ping) = any.downcast_ref::<v1_12::PingPacket>() {
            let response: Box<v1_12::PongPacket> = Box::new(v1_12::PongPacket::new(ping.payload));
            if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                let mut connection = connection.get();
                connection.send_packet(response);
            }
        }

        if let Some(unconnected_ping) = any.downcast_ref::<raknet::UnconnectedPingPacket>() {
            let response_string = "MCPE;rserver test;282;1.6.0;1;2;9999;test2;Survival;";
            let response = Box::new(raknet::UnconnectedPongPacket::new(
                0,
                1234,
                RAKNET_MAGIC,
                ShortLengthPrefixedString(response_string.to_string())
            ));
            if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                let mut connection = connection.get();
                connection.send_packet(response);
            }
        }
    }

    fn send_packet(&self, address: SocketAddr, packet: Box<Packet>) {
        if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
            connection.get().send_packet(packet);
        }
    }
}
