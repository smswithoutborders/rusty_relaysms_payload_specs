pub mod contents;
pub mod payloads;
pub mod cryptography;
mod transports;

const VERSION: u8 = 0x01;

pub fn get_version() -> u8 {
    VERSION
}

