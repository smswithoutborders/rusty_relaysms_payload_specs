use std::sync::Arc;
use crate::{bit_utils};
use crate::v1::contents::{v1_deserialize_for_content, V1Contents};
use crate::v1::contents::email::V1Emails;
use crate::v1::payloads::{V1Payloads, V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::{ContentDeserializationError, ContentSerializationError, KeyIdTooLarge, MissingPayload, VersionTooLarge};


#[derive(Debug, uniffi::Object)]
pub struct PayloadWithoutAttachments {
    i_att: bool,
    version: u8,
    k_id: u8,
    t_id: u32,
    payload: Option<Arc<dyn V1Contents>>,
}


#[uniffi::export]
impl PayloadWithoutAttachments {
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_t_id(&self) -> u32 { self.t_id }
    pub fn get_payload_content(&self) -> Option<Arc<dyn V1Contents>> { self.payload.clone() }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        k_id: u8,
        t_id: u32,
        payload: Option<Arc<dyn V1Contents>>,
    ) -> Result<Arc<Self>, V1PayloadsError> {
        if version > (2u8.pow(4) - 1) {
            return Err(VersionTooLarge);
        }
        if k_id > (2u8.pow(4) - 1) {
            return Err(KeyIdTooLarge);
        }

        Ok(Arc::new(Self {
            i_att: false,
            version,
            k_id,
            t_id,
            payload,
        }))
    }
}


#[uniffi::export]
pub fn deserialize_payload_without_attachments(
    data: &[u8]
) -> Result<Arc<PayloadWithoutAttachments>, V1PayloadsError> {
    let version = bit_utils::get_bits(&data[0], 0, 6);
    let i_att = bit_utils::is_bit_on(&data[0], 7);
    let k_id = data[1];
    let t_id = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
    let cat_id = 0; // TODO("Figure this out")
    let payload = match v1_deserialize_for_content(
        cat_id,
        data[6..].to_vec()
    ) {
        Ok(payload) => Some(payload),
        Err(e) => return Err(ContentDeserializationError) // TODO: put in the error
    };

    Ok(Arc::new( PayloadWithoutAttachments {
        i_att,
        version,
        k_id,
        t_id,
        payload
    }))
}


impl PartialEq for PayloadWithoutAttachments {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.k_id == other.k_id
            && self.i_att == other.i_att
            && self.t_id == other.t_id
            && self.payload.as_ref()
            .zip(self.payload.as_ref())
            .map_or(false, |(a, b)| Arc::ptr_eq(a, b))
    }
}


#[uniffi::export]
impl V1Payloads for PayloadWithoutAttachments {
    fn serialize(&self) -> crate::v1::payloads::Result<Vec<u8>> {
        if !self.payload.is_some() {
            return Err(MissingPayload)
        }

        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = bit_utils::put_value(&0, 0, self.version, 1);
        if self.i_att { byte1 = bit_utils::turn_bit_on(&byte1, 7); }
        bytes.push(byte1);

        bytes.push(self.k_id);
        bytes.extend(self.t_id.to_le_bytes());

        let payload = match self.payload.as_ref().unwrap().serialize() {
            Ok(payload) => payload,
            Err(e) => return Err(ContentSerializationError)
        };
        bytes.extend(payload);
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
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let email = V1Emails::new(
        to,
        body,
        Option::from(subject.to_string()),
    ).unwrap();

    let version: u8 = 1;
    let e_id: u8 = 5;
    let k_id: u8 = 7;
    let t_id: u32 = 2;
    let payload: Option<Arc<dyn V1Contents>> = Some(email);

    let transport_att_false = PayloadWithoutAttachments::new(
        version,
        k_id,
        t_id,
        payload,
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = deserialize_payload_without_attachments(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);
}
