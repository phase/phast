use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::thread;
use std::time::*;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;

use network::*;
use network::types::*;
use network::packet::*;

use network::protocol::*;
use network::protocol::java::*;
use network::protocol::bedrock::*;

pub struct Server {
    pub network_manager: NetworkManager,
    pub connection_manager: Arc<ConnectionManager>,
    pub threads: Vec<JoinHandle<()>>,
    // Packet Channel
    pub packet_sender: Sender<(SocketAddr, Packet)>,
    pub packet_receiver: Receiver<(SocketAddr, Packet)>,
}

impl Server {
    pub fn new() -> Self {
        let connection_manager = Arc::new(ConnectionManager::new());
        let (packet_sender, packet_receiver) = channel::<(SocketAddr, Packet)>();
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
//                        println!("[Server] Sleeping for {:?}", sleep);
                        thread::sleep(sleep);
                    } else {
                        println!("[Server] WARNING: Game Loop took {:?}! Game Loop Thread will not sleep.", elapsed);
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

    fn handle_packet(&mut self, address: SocketAddr, packet: Packet) {
        match packet {
            // Ping
            Packet::java_v1_7_HandshakePacket(packet) => {
                let protocol_version = packet.protocol_version.0;
                let protocol = protocol::get_protocol(protocol_version);
                if let Some(protocol) = protocol.clone() {
                    dbg!(protocol.name());
                    if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                        connection.get().protocol = protocol
                    }
                }

                match packet.next_state.0 {
                    1 => {
                        let support = if let Some(protocol) = protocol.clone() {
                            format!("§2Client: {}", protocol.name())
                        } else {
                            "§cYour version is not supported yet!".to_string()
                        };

                        // Server List Ping
                        let response_string = format!("{{\
                            \"version\": {{\
                                \"name\": \"§7[§f1.7-1.14§7] §e0§b/§e1000\",\
                                \"protocol\": -1\
                            }},\
                            \"players\": {{\
                                \"max\": 3,\
                                \"online\": 2,\
                                \"sample\": [\
                                    {{\
                                        \"name\": \"§bphast §fis §calpha §fsoftware.\",\
                                        \"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"\
                                    }},\
                                    {{\
                                        \"name\": \"don't expect much\",\
                                        \"id\": \"4566e69f-c907-48ee-8d71-d7ba5aa00d20\"\
                                    }}\
                                ]\
                            }},\
                            \"description\": {{\
                                \"text\": \"§bphast §eserver\\n§f§ohttps://pha.st/ {}\"\
                            }}\
                        }}", support);
                        let response = Packet::java_v1_7_ResponsePacket(v1_7::ResponsePacket::new(VarIntLengthPrefixedString(response_string.to_string())));
                        self.send_packet(address, response);
                    }
                    2 => {
                        // Login state handled earlier
                    }
                    _ => {}
                }
            }
            Packet::java_v1_7_RequestPacket(_) => { /* do nothing */ }
            Packet::java_v1_7_PingPacket(packet) => {
                // You can send ResponsePackets here for an animated MOTD on 1.7 clients
                let response = Packet::java_v1_7_PongPacket(v1_7::PongPacket::new(packet.payload));
                self.send_packet(address, response);
            }
            Packet::bedrock_raknet_UnconnectedPingPacket(packet) => {
                let response_string = "MCPE;phast test;282;1.6.0;1;2;9999;test2;Survival;";
                let response = Packet::bedrock_raknet_UnconnectedPongPacket(raknet::UnconnectedPongPacket::new(
                    0,
                    1234,
                    RAKNET_MAGIC,
                    ShortLengthPrefixedString(response_string.to_string()),
                ));
                self.send_packet(address, response);
            }
            // Login
            Packet::java_v1_7_LoginStartPacket(packet) => {
                println!("[Server] Player wants to login: {}", packet.name.0);

//                let response = Packet::EncryptionRequestPacket(v1_12::EncryptionRequestPacket::new(
//                    VarIntLengthPrefixedString("".to_string()),
//                    VarIntLengthPrefixedByteArray(vec![0x0Au8, 0x0Bu8, 0x0Cu8, 0x0Du8, 0x0Eu8, 0x0Fu8]),
//                    VarIntLengthPrefixedByteArray(vec![0x0Au8, 0x0Bu8, 0x0Cu8, 0x0Du8]),
//                ));
//                self.send_packet(address, response);

                let response = Packet::java_v1_7_LoginSuccessPacket(v1_7::LoginSuccessPacket::new(
                    // includes hyphens
                    VarIntLengthPrefixedString("e63a1d61-adf1-4d47-b5f8-43efc5c84908".to_string()),
                    VarIntLengthPrefixedString(packet.name.0),
                ));
                self.send_packet(address, response);
                if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                    connection.get().protocol_state = State::JavaPlay;
                    println!("[Server] Set connection state to JavaPlay");
                }

                if let Some(protocol) = self.connection_manager.get_protocol(address) {
                    match protocol {
                        Protocol::ProtocolJava_1_7(_) => {
                            let join_game = Packet::java_v1_7_JoinGamePacket(v1_7::JoinGamePacket::new(
                                0, // entity id
                                0, // survival
                                0, // overworld
                                1, // peaceful
                                70, // legacy max player count
                                VarIntLengthPrefixedString("default".to_string()),
                            ));
                            self.send_packet(address, join_game);
                        }
                        Protocol::ProtocolJava_1_8(_) => {
                            let join_game = Packet::java_v1_8_JoinGamePacket(v1_8::JoinGamePacket::new(
                                0, // entity id
                                0, // survival
                                0, // overworld
                                1, // peaceful
                                70, // legacy max player count
                                VarIntLengthPrefixedString("default".to_string()),
                                0, // debug
                            ));
                            self.send_packet(address, join_game);
                        }
                        _ => {}
                    }
                }
            }
            Packet::bedrock_raknet_OpenConnectionRequest1Packet(packet) => {
                let response = Packet::bedrock_raknet_OpenConnectionReply1Packet(raknet::OpenConnectionReply1Packet::new(
                    RAKNET_MAGIC,
                    1234u64,
                    0u8,
                    800u16,
                ));
                self.send_packet(address, response);
            }
            Packet::bedrock_raknet_OpenConnectionRequest2Packet(packet) => {
                println!("{:#?}", packet);

                let response = Packet::bedrock_raknet_OpenConnectionReply2Packet(raknet::OpenConnectionReply2Packet::new(
                    RAKNET_MAGIC,
                    1234,
                    Address(address),
                    packet.mtu_size,
                    0,
                ));
                self.send_packet(address, response);

                if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
                    connection.get().protocol_state = State::BedrockRakNet;
                }
            }
            Packet::bedrock_raknet_ConnectionRequestPacket(packet) => {
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

                let response = Packet::bedrock_raknet_ConnectionRequestAcceptedPacket(raknet::ConnectionRequestAcceptedPacket::new(
                    Address(loopback),
                    0,
                    addresses,
                    packet.timestamp,
                    timestamp,
                ));

                self.send_packet(address, response)
            }
            _ => {
                dbg!(packet);
            }
        }
    }

    fn send_packet(&self, address: SocketAddr, packet: Packet) {
        println!("[Server] Sending {} to {}", packet.name(), address);
        if let Some(mut connection) = self.connection_manager.connections.find_mut(&address) {
            connection.get().send_packet(packet);
        }
    }
}
