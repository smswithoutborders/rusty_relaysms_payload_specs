use std::sync::Arc;
use crate::{bit_utils, utils};
use crate::contents::{deserialize_for_content, Contents};
use crate::contents::email::Emails;
use crate::payloads::{Payloads, PayloadsError};
use crate::payloads::payload_with_attachments::{PayloadWithAttachments, PayloadWithAttachmentsNoHeader};
use crate::payloads::PayloadsError::{CategoryIdTooLarge, ContentDeserializationError, ContentSerializationError, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, MissingDeviceID, MissingPayload, SessionIdTooLarge, VersionTooLarge};


#[derive(Debug, uniffi::Object)]
pub struct PayloadWithoutAttachment {
    i_att: bool,
    version: u8,
    k_id: u8,
    e_id: u8,
    cat_id: u8,
    payload: Option<Arc<dyn Contents>>,
}


#[uniffi::export]
impl PayloadWithoutAttachment {
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_cat_id(&self) -> u8 { self.cat_id }
    pub fn get_payload_content(&self) -> Option<Arc<dyn Contents>> { self.payload.clone() }


    #[uniffi::constructor]
    pub fn instance() -> Result<Arc<Self>, PayloadsError> {
        Ok(Arc::new(Self {
            i_att: false,
            version: 255,
            k_id: 255,
            e_id: 255,
            cat_id: 255,
            payload: None,
        }))
    }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        payload: Option<Arc<dyn Contents>>,
    ) -> Result<Arc<Self>, PayloadsError> {
        if version > (2u8.pow(4) - 1) {
            return Err(VersionTooLarge);
        }
        if e_id > (2u8.pow(3) - 1) {
            return Err(EncryptionIdTooLarge);
        }
        if k_id > (2u8.pow(4) - 1) {
            return Err(KeyIdTooLarge);
        }
        if cat_id > (2u8.pow(4) - 1) {
            return Err(CategoryIdTooLarge);
        }

        Ok(Arc::new(Self {
            i_att: false,
            version,
            e_id,
            k_id,
            cat_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, PayloadsError> {
        let version = bit_utils::get_bits(&data[0], 0, 6);
        let i_att = bit_utils::is_bit_on(&data[0], 7);
        let k_id = data[1];
        let e_id = bit_utils::get_bits(&data[2], 0, 3);
        let cat_id = bit_utils::get_bits(&data[2], 4, 7);
        let payload = match deserialize_for_content(
            cat_id,
            data[3..].to_vec()
        ) {
            Ok(payload) => Some(payload),
            Err(e) => return Err(ContentDeserializationError) // TODO: put in the error
        };

        Ok(Arc::new( Self {
            i_att,
            version,
            k_id,
            e_id,
            cat_id,
            payload
        }))
    }

}


impl PartialEq for PayloadWithoutAttachment {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.cat_id == other.cat_id
            && self.payload.as_ref()
            .zip(self.payload.as_ref())
            .map_or(false, |(a, b)| Arc::ptr_eq(a, b))
    }
}


#[uniffi::export]
impl Payloads for PayloadWithoutAttachment {
    fn serialize(&self) -> crate::payloads::Result<Vec<u8>> {
        if !self.payload.is_some() {
            return Err(MissingPayload)
        }

        let mut bytes: Vec<u8> = Vec::new();

        let byte1 : u8 = bit_utils::put_value(&0, 0, self.version, 1);
        if self.i_att { bit_utils::turn_bit_on(&byte1, 7); }
        bytes.push(byte1);

        bytes.push(self.k_id);

        let byte2 = bit_utils::put_value(&self.e_id, 4, self.cat_id, 4);
        bytes.push(byte2);

        let payload = match self.payload.as_ref().unwrap().serialize() {
            Ok(payload) => payload,
            Err(e) => return Err(ContentSerializationError)
        };
        bytes.extend(payload);
        Ok(bytes)
    }

    fn equals(&self, other: Arc<dyn Payloads>) -> bool {
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
    let from_id: u8 = 7; // 1
    let email = Emails::new(
        to,
        body,
        Option::from(subject.to_string()),
        &from_id
    ).unwrap();

    let version: u8 = 1;
    let e_id: u8 = 5;
    let k_id: u8 = 7;
    let cat_id: u8 = email.get_cat_id();
    let payload: Option<Arc<dyn Contents>> = Some(email);

    let transport_att_false = PayloadWithoutAttachment::new(
        version,
        e_id,
        k_id,
        cat_id,
        payload,
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = PayloadWithoutAttachment::instance().unwrap()
        .deserialize(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);
}
