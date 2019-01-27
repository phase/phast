#![feature(proc_macro_hygiene)]

extern crate concurrent_hashmap;
extern crate paste;

mod server;
mod network;
use server::*;
use network::*;

fn main() {
    let mut server = Server::new();
    server.start();
    server.join_network_threads();
}
