use std::net::{TcpStream, TcpListener, UdpSocket};
use std::io::{Write, Read};
use std::thread;

mod network;

// Java Edition uses TCP
fn start_tcp() {
    let listener = TcpListener::bind("0.0.0.0:25565").unwrap();
    println!("TCP on 0.0.0.0:25565");

    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                thread::spawn(move || {
                    loop {
                        let mut buf = vec![0; 64];
                        let length = socket.read(&mut buf).unwrap_or(0);
                        if length > 0 {
                            println!("TCP: {:?} {:X?}", socket.peer_addr().unwrap(), &buf[..length]);
                        }
                    }
                });
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}

// Bedrock Edition uses UDP
fn start_udp() {
    let socket = UdpSocket::bind("0.0.0.0:19132").unwrap();
    println!("UDP on 0.0.0.0:19132");

    loop {
        let mut buf = [0; 64];
        let (length, address) = socket.recv_from(&mut buf).unwrap();
        if length > 0 {
            println!("UDP: {:?} {:X?}", address, &buf[..length])
        }
    }
}

fn main() {
    let tcp_handle = thread::spawn(move || {
        start_tcp();
    });

    let udp_handle = thread::spawn(move || {
        start_udp();
    });

    tcp_handle.join().unwrap();
    udp_handle.join().unwrap();
}
