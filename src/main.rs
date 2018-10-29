extern crate concurrent_hashmap;

mod server;
mod network;
use server::*;
use network::*;

fn main() {
    let mut server = Server::new();
    server.start();
    server.join_network_threads();
}
