use std::sync::Arc;
use crate::{bit_utils, v1};
use crate::v1::contents::{V1Contents};
use crate::v1::contents::email::V1Emails;
use crate::v1::get_version;
use crate::v1::payloads::{V1Payloads, V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::{ContentDeserializationError, ContentSerializationError, KeyIdTooLarge, MissingPayload, VersionTooLarge};


#[derive(Debug, PartialEq, uniffi::Object)]
pub struct V1PayloadWithoutAttachments {
    version: u8,
    i_tid: bool,
    i_att: bool,
    k_id: u8,
    t_id: Option<u32>,
    payload: Vec<u8>,
}


#[uniffi::export]
impl V1PayloadWithoutAttachments {
    pub fn get_i_att(&self) -> bool { self.i_att }
    pub fn get_i_tid(&self) -> bool { self.i_tid }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_payload_content(&self) -> Vec<u8> { self.payload.clone() }

    #[uniffi::constructor]
    pub fn new(
        k_id: u8,
        t_id: Option<u32>,
        payload: Vec<u8>,
    ) -> Result<Arc<Self>, V1PayloadsError> {
        if k_id > (2u8.pow(4) - 1) {
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
            payload,
        }))
    }
}


#[uniffi::export]
pub fn v1_deserialize_payload_without_attachments(
    data: &[u8]
) -> Result<Arc<V1PayloadWithoutAttachments>, V1PayloadsError> {
    let version = bit_utils::get_bits(&data[0], 0, 2);
    let i_tid = bit_utils::is_bit_on(&data[0], 3);
    let i_att = bit_utils::is_bit_on(&data[0], 4);
    let k_id = data[1];
    let t_id = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
    let payload = data[6..].to_vec();

    Ok(Arc::new( V1PayloadWithoutAttachments {
        version,
        i_tid,
        i_att,
        k_id,
        t_id: Some(t_id),
        payload
    }))
}

#[uniffi::export]
impl V1Payloads for V1PayloadWithoutAttachments {
    fn serialize(&self) -> crate::v1::payloads::Result<Vec<u8>> {
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

    fn equals(&self, other: Arc<dyn V1Payloads>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}

#[test]
fn att_false_serialize() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let email = V1Emails::new(
        to.to_vec(),
        body.to_vec(),
        Option::from(subject.to_vec()),
    ).unwrap();

    let k_id: u8 = 7;
    let t_id: u32 = 2;
    let payload = email.serialize().unwrap();

    let transport_att_false = V1PayloadWithoutAttachments::new(
        k_id,
        Some(t_id),
        payload,
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = v1_deserialize_payload_without_attachments(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);
}
