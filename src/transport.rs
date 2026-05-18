use crate::{bit_utils, Contents, TransportError};
use crate::email::Emails;

type Result<T> = std::result::Result<T, TransportError>;
#[derive(Debug)]
pub struct Transport {
    i_did: bool,
    version: u8,
    encryption_id: u8,
    key_id: u8,
    device_id: Option<u16>,
    payload_content: Box<dyn Contents>,
}

impl Transport {
    pub fn new(
        i_did: bool,
        version: u8,
        encryption_id: u8,
        key_id: u8,
        device_id: Option<u16>,
        payload_content: Box<dyn Contents>,
    ) -> Transport {
        Transport {
            i_did,
            version,
            encryption_id,
            key_id,
            device_id,
            payload_content,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte1 = if self.i_did { 1 } else { 0 };
        byte1 = bit_utils::put_value(&byte1, 1, self.version, 4);
        byte1 = bit_utils::put_value(&byte1, 5, self.encryption_id, 5);
        bytes.push(byte1);
        bytes.push(self.key_id);
        if self.device_id.is_some() {
            let device_id = u16::to_le_bytes(self.device_id.unwrap());
            bytes.push(device_id[0]);
            bytes.push(device_id[1]);
        }
        let payload_content = self.payload_content.serialize();
        if payload_content.is_ok() {
            bytes.extend_from_slice(&payload_content.unwrap());
        }
        Ok(bytes)
    }

    pub fn deserialize(data: &[u8]) -> Result<Transport> {
        let i_did = bit_utils::is_bit_on(&data[0], 0);
        let version = bit_utils::get_bits(  &data[1], 1, 4);
        let encryption_id = bit_utils::get_bits(  &data[1], 5, 7);
        let key_id = data[1];
        let mut current_index: usize = 2;
        let device_id = if i_did {
            let slice = u16::from_le_bytes([data[current_index], data[current_index+1]]);
            current_index += 1;
            Option::from(slice)
        } else { None };
        let payload_content = Emails::deserialize(
            &data[current_index..]);

        Ok(Transport {
            i_did,
            version,
            encryption_id,
            key_id,
            device_id,
            payload_content: Box::new(payload_content.unwrap()),
        })
    }
}