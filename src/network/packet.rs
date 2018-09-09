use network::connection;

trait Packet {
    fn read(bytes: &[u8]);
    fn write(&self) -> &[u8];
}


pub struct UnspecifiedPacket {
    // java uses varint, bedrock uses unsigned byte
    id: i32,
    bytes: [u8],
}

impl Packet for UnspecifiedPacket {
    fn read(bytes: &[u8]) {}

    fn write(&self) -> &[u8] {
//        Vec::from(self.bytes).as_slice()
        panic!("unimplemented")
    }
}
