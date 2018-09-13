extern crate concurrent_hashmap;

use std::net::{TcpStream, TcpListener, UdpSocket};
use std::io::{Write, Read};
use std::thread;
use std::sync::Arc;

mod server;
mod network;
use server::*;
use network::*;

fn main() {
    let mut server = Server::new();
    server.start();
    for join_handle in server.threads {
        join_handle.join().unwrap();
    }
}
