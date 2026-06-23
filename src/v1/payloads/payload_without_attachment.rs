use std::sync::Arc;
use crate::{bit_utils, v1};
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1Contents, V1ContentsContainer};
use crate::v1::payloads::{V1Payloads, V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::{ContentSerializationError, KeyIdTooLarge, MissingPayload, VersionTooLarge};

type Result<T> = std::result::Result<T, V1PayloadsError>;

// #[derive(Debug, PartialEq, uniffi::Object)]
#[derive(Debug, PartialEq)]
pub struct V1PayloadWithoutAttachments {
    version: u8,
    i_tid: bool,
    i_att: bool,
    k_id: u8,
    t_id: Option<u32>,
    payload: Vec<u8>,
}

impl V1PayloadWithoutAttachments {
    pub fn get_i_tid(&self) -> bool { self.i_tid }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_payload_content(&self) -> Vec<u8> { self.payload.clone() }

    pub fn new(
        k_id: u8,
        t_id: Option<u32>,
        payload: &[u8],
    ) -> Result<Arc<Self>> {
        if k_id > (2u8.pow(8) - 1) {
            return Err(KeyIdTooLarge);
        }

        let i_tid = t_id.is_some();

        let version = v1::get_version();
        Ok(Arc::new(Self {
            version,
            i_tid,
            i_att: false,
            k_id,
            t_id,
            payload: payload.to_vec(),
        }))
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = bit_utils::put_value(&0, 0, self.version, 5);
        if self.i_tid { byte1 = bit_utils::turn_bit_on(&byte1, 3); }
        if self.i_att { byte1 = bit_utils::turn_bit_on(&byte1, 4); }
        bytes.push(byte1);

        bytes.push(self.k_id);
        if self.i_tid {
            bytes.extend(self.t_id.as_ref().unwrap().to_le_bytes());
        }
        bytes.extend(self.payload.clone());
        Ok(bytes)
    }

    pub fn deserialize(data: &[u8]) -> Result<Arc<V1PayloadWithoutAttachments>> {
        let version = bit_utils::get_bits(&data[0], 0, 2);
        let i_tid = bit_utils::is_bit_on(&data[0], 3);
        let i_att = bit_utils::is_bit_on(&data[0], 4);
        let k_id = data[1];
        let mut current_index = 2;
        let t_id = if i_tid {
            current_index = 6;
            Some(u32::from_le_bytes([data[2], data[3], data[4], data[5]]))
        } else { None };
        let payload = data[current_index..].to_vec();

        V1PayloadWithoutAttachments::new(
            k_id,
            t_id,
            payload.as_slice(),
        )
    }
}

