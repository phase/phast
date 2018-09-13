use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Write, Read};
use std::thread;
use std::time::*;
use std::thread::JoinHandle;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;

use network::*;
use network::packet::*;
use network::connection::*;

pub struct Server {
    pub connection_manager: Arc<ConnectionManager>,
    pub threads: Vec<JoinHandle<()>>,
    // Packet Channel
    pub packet_sender: Sender<(SocketAddr, Box<Packet>)>,
    pub packet_receiver: Receiver<(SocketAddr, Box<Packet>)>,
    // Unprocessed Data Channel
    pub unprocessed_sender: Sender<(SocketAddr, Vec<u8>)>,
}

impl Server {
    pub fn new() -> Self {
        let (packet_sender, packet_receiver) = channel::<(SocketAddr, Box<Packet>)>();
        let (unprocessed_sender, unprocessed_receiver) = channel::<(SocketAddr, Vec<u8>)>();

        let mut this = Self {
            connection_manager: Arc::new(ConnectionManager::new()),
            threads: Vec::with_capacity(4),
            packet_sender,
            packet_receiver,
            unprocessed_sender,
        };

        this.start(unprocessed_receiver);

        this
    }

    pub fn start(&mut self, unprocessed_receiver: Receiver<(SocketAddr, Vec<u8>)>) {
        let tcp_listener_thread = thread::Builder::new().name("TCP-Listener".into());
        let tcp_listener_handle = tcp_listener_thread.spawn({
            let connection_manager = self.connection_manager.clone();
            move || {
                Server::start_tcp_listener(connection_manager);
            }
        }).unwrap();
        self.threads.push(tcp_listener_handle);

        let tcp_read_thread = thread::Builder::new().name("TCP-Read".into());
        let tcp_read_handle = tcp_read_thread.spawn({
            let connection_manager = self.connection_manager.clone();
            let unprocessed_sender = self.unprocessed_sender.clone();
            move || {
                Server::start_tcp_reads(connection_manager, unprocessed_sender);
            }
        }).unwrap();
        self.threads.push(tcp_read_handle);

        let udp_thread = thread::Builder::new().name("UDP".into());
        let udp_handle = udp_thread.spawn({
            let connection_manager = self.connection_manager.clone();
            let unprocessed_sender = self.unprocessed_sender.clone();
            move || {
                Server::start_udp(connection_manager, unprocessed_sender);
            }
        }).unwrap();
        self.threads.push(udp_handle);

        let packet_parse_thread = thread::Builder::new().name("Packet-Parse".into());
        let packet_parse_handle = packet_parse_thread.spawn({
            let connection_manager = self.connection_manager.clone();
            let packet_sender = self.packet_sender.clone();
            move || {
                Server::start_packet_parse_loop(connection_manager, unprocessed_receiver, packet_sender);
            }
        }).unwrap();
        self.threads.push(packet_parse_handle);
    }

    fn start_packet_parse_loop(
        connection_manager: Arc<ConnectionManager>,
        bytes: Receiver<(SocketAddr, Vec<u8>)>,
        packet_channel: Sender<(SocketAddr, Box<Packet>)>,
    ) {
        loop {
            match bytes.recv() {
                Ok((address, mut bytes)) => {
                    println!("[Packet-Parse]: Received Bytes from {}: {:X?}", address, bytes);
                    if let Some(mut connection) = connection_manager.connections.find_mut(&address) {
                        let packets = (*connection.get()).handle_read(&mut bytes);
                        println!("[Packet-Parse]: Received Packets from {}: {}", address, packets.len());
                        for packet in packets {
                            packet_channel.send((address, packet));
                        }
                    }
                }
                Err(e) => {
                    println!("[Packet-Parse]: Error when receiving bytes in parse loop: {}", e);
                }
            }
        }
    }

    /// Starts listening for incoming connections and adds them to the `connection_manager`
    fn start_tcp_listener(connection_manager: Arc<ConnectionManager>) {
        let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
        // this thread can be blocking since it isn't locking anything
        listener.set_nonblocking(false);
        println!("TCP on 0.0.0.0:25565");

        for stream in listener.incoming() {
            match stream {
                Ok(mut socket) => {
                    // these connections need to be non-blocking so we don't hog
                    // the lock to the connection in the thread below
                    socket.set_nonblocking(true);
                    let address = socket.peer_addr().unwrap();
                    let mut connection = Connection::new(address, SocketWrapper::TCP(socket));
                    connection_manager.connections.insert(address, connection);
                    let mut tcp_addresses = connection_manager.tcp_addresses.lock().unwrap();
                    tcp_addresses.push(address);
                    println!("[TCP-Listener]: Accepted new connection from {}", address);
                }
                Err(e) => {
//                    println!("[TCP-Listener]: Failed to accept connection: {}", e)
                }
            }
        }
    }

    fn start_tcp_reads(connection_manager: Arc<ConnectionManager>, byte_sender: Sender<(SocketAddr, Vec<u8>)>) {
        let read_tick = Duration::from_millis(100);
        loop {
            let now = SystemTime::now();
            for address in connection_manager.tcp_addresses.lock().unwrap().iter() {
                if let Some(mut connection) = connection_manager.connections.find_mut(&address) {
                    match connection.get().socket {
                        SocketWrapper::TCP(ref mut stream) => {
                            let mut buf = vec![0; 64];
                            let length = stream.read(&mut buf).unwrap_or(0);

                            if length > 0 {
                                println!("[TCP-Read]: Read {} bytes from {}", length, address);
                                byte_sender.send((*address, (&buf[..length]).to_vec()));
                            }
                        }
                        _ => {}
                    };
                }
            }
            match now.elapsed() {
                Ok(elapsed) => {
                    let sleep = read_tick - elapsed;
//                    println!("[TCP-Read]: Sleeping for {:?}", sleep);
                    thread::sleep(sleep);
                }
                Err(_) => {}
            }
        }
    }

    // Bedrock Edition uses UDP
    fn start_udp(connection_manager: Arc<ConnectionManager>, byte_sender: Sender<(SocketAddr, Vec<u8>)>) {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:19132").unwrap());
        println!("UDP on 0.0.0.0:19132");

        loop {
            let mut buf = vec![0; 64];
            let (length, address) = socket.recv_from(&mut buf).unwrap();
            if length > 0 {
                let buf = (&mut buf[..length]).to_vec();

                println!("[UDP]: Read {} bytes from {}", buf.len(), address);
                if let None = connection_manager.connections.find_mut(&address) {
                    // this is a new connection
                    println!("[UDP]: Accepted new connection from {}", address);
                    let mut connection = Connection::new(address, SocketWrapper::UDP(socket.clone()));
                    connection_manager.connections.insert(address, connection);
                }
                byte_sender.send((address, buf));
            }
        }
    }
}
