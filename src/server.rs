use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
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

macro_rules! if_packet {
    ($packet:ident = $t:ty $b:block) => {
        if let Some($packet) = $packet.downcast_ref::<$t>() $b
    };
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
                    self.handle_packet(address, packet);
                }
                _ => {
                    packets_left = false;
                }
            }
        }
    }

    fn handle_packet(&mut self, address: SocketAddr, packet: Box<Packet>) {
        let packet: Box<Any> = packet.as_any();

        // Ping

        if_packet!(packet = v1_12::HandshakePacket {
            let protocol_version = packet.protocol_version.0;
            match packet.next_state.0 {
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

                    self.send_packet(address, response);
                }
                2 => {
                    // Login state handled earlier
                }
                _ => {}
            }
        });

        if_packet!(packet = v1_12::PingPacket {
            let response: Box<v1_12::PongPacket> = Box::new(v1_12::PongPacket::new(packet.payload));
            self.send_packet(address, response);
        });

        if_packet!(packet = raknet::UnconnectedPingPacket {
            let response_string = "MCPE;rserver test;282;1.6.0;1;2;9999;test2;Survival;";
            let response = Box::new(raknet::UnconnectedPongPacket::new(
                0,
                1234,
                RAKNET_MAGIC,
                ShortLengthPrefixedString(response_string.to_string()),
            ));
            self.send_packet(address, response);
        });


        // Login

        if_packet!(packet = v1_12::LoginStartPacket {
            println!("[Server] LoginStartPacket: {}", packet.name.0);
//
//            let response = Box::new(v1_12::EncryptionRequestPacket::new(
//                VarIntLengthPrefixedString("".to_string()),
//                VarIntLengthPrefixedByteArray(vec![0x0Au8, 0x0Bu8, 0x0Cu8, 0x0Du8, 0x0Eu8, 0x0Fu8]),
//                VarIntLengthPrefixedByteArray(vec![0x0Au8, 0x0Bu8, 0x0Cu8, 0x0Du8]),
//            ));
//            self.send_packet(address, response);
        });

        if_packet!(packet = raknet::OpenConnectionRequest1Packet {
            let response = Box::new(raknet::OpenConnectionReply1Packet::new(
                RAKNET_MAGIC,
                1234u64,
                0u8,
                800u16,
            ));
            self.send_packet(address, response);
        });

        if_packet!(packet = raknet::OpenConnectionRequest2Packet {
            println!("{:#?}", packet);

            let response = Box::new(raknet::OpenConnectionReply2Packet::new(
                RAKNET_MAGIC,
                1234,
                Address(address),
                packet.mtu_size,
                0
            ));
            self.send_packet(address, response);

            if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                connection.get().protocol_state = State::BedrockRakNet;
            }
        });

        if_packet!(packet = raknet::ConnectionRequestPacket {
            println!("{:#?}", packet);
            let loopback = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 19132);
            let garbage = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 19132);

            let addresses = vec![
                Address(loopback), Address(garbage), Address(garbage), Address(garbage), Address(garbage),
                Address(garbage), Address(garbage), Address(garbage), Address(garbage), Address(garbage),
                Address(garbage), Address(garbage), Address(garbage), Address(garbage), Address(garbage),
                Address(garbage), Address(garbage), Address(garbage), Address(garbage), Address(garbage),
            ];

            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
            let timestamp = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

            let response = Box::new(raknet::ConnectionRequestAcceptedPacket::new(
                Address(loopback),
                0,
                addresses,
                packet.timestamp,
                timestamp
            ));

            self.send_packet(address, response)
        });
    }

    fn send_packet(&self, address: SocketAddr, packet: Box<Packet>) {
        println!("[Server]: Sending {} to {}", packet.name(), address);
        if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
            connection.get().send_packet(packet);
        }
    }
}
