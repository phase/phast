use std::net::{TcpStream, TcpListener, UdpSocket};
use std::io::{Write, Read};
use std::thread;

// Java Edition uses TCP
fn start_tcp() {
    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
    println!("TCP on 127.0.0.1:25565");

    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                thread::spawn(move || {
                    loop {
                        let mut buf = vec![0; 64];
                        let length = socket.read(&mut buf).unwrap_or(0);
                        if length > 0 {
                            println!("TCP: {:?} {:?}", socket.peer_addr().unwrap(), &buf[..length]);
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
    let mut socket = UdpSocket::bind("127.0.0.1:19132").unwrap();
    println!("UDP on 127.0.0.1:19132");

    loop {
        let mut buf = [0; 64];
        let (length, socket) = socket.recv_from(&mut buf).unwrap();
        if length > 0 {
            println!("UDP: {:?} {:?}", socket, &buf[..length])
        }
    }
}

fn main() {
    thread::spawn(move || {
        start_tcp();
    });

    thread::spawn(move || {
        start_udp();
    });
    loop{}
}
