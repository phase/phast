extern crate concurrent_hashmap;

use std::net::{TcpStream, TcpListener, UdpSocket};
use std::io::{Write, Read};
use std::thread;
use std::sync::Arc;

mod network;

use network::*;
use network::connection::*;

/// Starts listening for incoming connections and adds them to the `connection_manager`
fn start_tcp_listener(connection_manager: Arc<ConnectionManager>) {
    let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
    println!("TCP on 0.0.0.0:25565");

    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                let address = socket.peer_addr().unwrap();
                let mut connection = Connection::new(address, SocketWrapper::TCP(socket));
                connection_manager.connections.insert(address, connection);
                println!("[TCP-Listener]: Accepted new connection from {}", address);
            }
            Err(e) => {
                println!("[TCP-Listener]: Failed to accept connection: {}", e)
            }
        }
    }
}

fn start_tcp_reads(connection_manager: Arc<ConnectionManager>) {
    loop {
        let mut tcp_addresses = Vec::new();
        for (address, mut connection) in connection_manager.connections.iter() {
            if connection.is_tcp() {
                tcp_addresses.push(address);
            }
        }

        for address in tcp_addresses {
            if let Some(mut connection) = connection_manager.connections.find_mut(&address) {
                let length = (*connection.get()).read();
                if length > 0 {
                    println!("[TCP-Read]: Read {} bytes from {}", length, address);
                }
            }
        }
    }
}

// Bedrock Edition uses UDP
fn start_udp(connection_manager: Arc<ConnectionManager>) {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:19132").unwrap());
    println!("UDP on 0.0.0.0:19132");

    loop {
        let mut buf = vec![0; 64];
        let (length, address) = socket.recv_from(&mut buf).unwrap();
        if length > 0 {
            let buf = &mut buf[..length].to_vec();

            println!("[UDP]: Read {} bytes from {}", buf.len(), address);
            if let Some(mut connection) = connection_manager.connections.find_mut(&address) {
                // connection exists
                (*connection.get()).handle_read(buf);
            } else {
                // this is a new connection
                println!("[UDP]: Accepted new connection from {}", address);
                let mut connection = Connection::new(address, SocketWrapper::UDP(socket.clone()));
                connection_manager.connections.insert(address, connection);
                // we need to use the connection after we've inserted it into the manager
                if let Some(mut connection) = connection_manager.connections.find_mut(&address) {
                    (*connection.get()).handle_read(buf);
                }
            }
        }
    }
}

fn main() {
    let connection_manager = Arc::new(ConnectionManager::new());

    let tcp_listener_handle = thread::spawn({
        let connection_manager = connection_manager.clone();
        move || {
            start_tcp_listener(connection_manager);
        }
    });

    let tcp_read_handle = thread::spawn({
        let connection_manager = connection_manager.clone();
        move || {
            start_tcp_reads(connection_manager);
        }
    });

    let udp_handle = thread::spawn({
        let connection_manager = connection_manager.clone();
        move || {
            start_udp(connection_manager);
        }
    });

    tcp_listener_handle.join().unwrap();
    tcp_read_handle.join().unwrap();
    udp_handle.join().unwrap();
}
