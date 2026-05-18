use crate::{bit_utils, Contents};
use crate::email::Emails;

type Result<T> = std::result::Result<T, TransportError>;

#[derive(Debug)]
pub enum TransportError {
    VersionTooLarge,
    EncryptionIdTooLarge,
}

#[derive(Debug)]
pub struct Transport {
    i_did: bool,
    version: u8,
    encryption_id: u8,
    key_id: u8,
    device_id: Option<Vec<u8>>,
    payload_content: Box<dyn Contents>,
}

impl PartialEq for Transport {
    fn eq(&self, other: &Self) -> bool {
        self.i_did == other.i_did
            && self.version == other.version
            && self.encryption_id == other.encryption_id
            && self.key_id == other.key_id
            && self.device_id == other.device_id
            && self.payload_content.equals(other.payload_content.as_ref())
    }
}

impl Transport {
    pub fn new(
        i_did: bool,
        version: u8,
        encryption_id: u8,
        key_id: u8,
        device_id: Option<Vec<u8>>,
        payload_content: Box<dyn Contents>,
    ) -> Result<Transport> {
        if version > (2u8.pow(4) -1) {
            return Err(TransportError::VersionTooLarge);
        }

        if encryption_id > (2u8.pow(3) -1) {
            return Err(TransportError::EncryptionIdTooLarge);
        }

        Ok(Transport {
            i_did,
            version,
            encryption_id,
            key_id,
            device_id,
            payload_content,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut byte1 = if self.i_did { 1 } else { 0 };
        byte1 = bit_utils::put_value(&byte1, 1, self.version, 4);
        byte1 = bit_utils::put_value(&byte1, 5, self.encryption_id, 5);
        bytes.push(byte1);
        bytes.push(self.key_id);
        if self.device_id.is_some() {
            bytes.extend_from_slice(self.device_id.as_ref().unwrap().as_slice())
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
            let slice = data[current_index..current_index + 16].to_vec();
            current_index += 16;
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