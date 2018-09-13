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

            match now.elapsed() {
                Ok(elapsed) => {
                    if elapsed < tick_time {
                        let sleep = tick_time - elapsed;
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
}
