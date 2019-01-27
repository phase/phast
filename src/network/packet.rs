use network::protocol;

pub trait PacketType: Send + Sync {
    fn name(&self) -> &str;
    fn read(&mut self, bytes: Vec<u8>) -> bool;
    fn write(&self) -> Vec<u8>;
    fn next_state(&self) -> Option<protocol::State>;
}

/// Read type from bytes
pub trait ReadField {
    /// returns the type & the length to increment the index by
    fn read(bytes: &Vec<u8>, index: usize) -> Option<(Self, usize)> where Self: Sized;
}

/// Write type to bytes
pub trait WriteField where Self: Sized {
    // TODO: Vec is probably going to kill performance
    fn write(&self) -> Vec<u8>;
}

#[macro_export]
macro_rules! packet {
    ($packet_name:ident, $($field:ident: $t:ty),*) => {
        packet!($packet_name, $($field: $t),*; |_s:&$packet_name|{None});
    };
    ($packet_name:ident, $($field:ident: $t:ty),*; $next_state:expr) => {
        #[derive(Clone, Default, Debug)]
        pub struct $packet_name {
            $(
                pub $field: $t,
            )*
        }

        #[allow(dead_code)]
        impl $packet_name {
            pub fn new($($field: $t,)*) -> Self {
                Self {
                    $(
                        $field,
                    )*
                }
            }
        }

        #[allow(dead_code)]
        impl PacketType for $packet_name {
            fn name(&self) -> &str {
                stringify!($packet_name)
            }

            #[allow(unused_assignments)]
            #[allow(unused_variables)]
            fn read(&mut self, bytes: Vec<u8>) -> bool {
                #[allow(unused_mut)]
                let mut index = 0usize;
                $(
                    match <$t as ReadField>::read(&bytes, index) {
                        Some((value, length)) => {
                            self.$field = value;
                            index += length;
                        },
                        None => return false
                    }
                )*
                true
            }

            fn write(&self) -> Vec<u8> {
                #[allow(unused_mut)]
                let mut buf = Vec::<u8>::new();
                $(
                    buf.append(&mut self.$field.write());
                )*
                buf
            }

            fn next_state(&self) -> Option<State> {
                $next_state(self)
            }
        }
    };
}
