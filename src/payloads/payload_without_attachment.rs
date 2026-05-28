use std::sync::Arc;
use crate::{bit_utils, utils};
use crate::contents::{deserialize_for_content, Contents};
use crate::contents::email::Emails;
use crate::payloads::{Payloads, PayloadsError, PayloadsWithoutAttachments};
use crate::payloads::payload_with_attachments::{PayloadWithAttachments, PayloadWithAttachmentsNoHeader};
use crate::payloads::PayloadsError::{CategoryIdTooLarge, ContentDeserializationError, ContentSerializationError, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, MissingDeviceID, MissingPayload, SessionIdTooLarge, VersionTooLarge};


#[derive(Debug, uniffi::Object)]
pub struct PayloadWithoutAttachments {
    i_att: bool,
    version: u8,
    k_id: u8,
    e_id: u8,
    f_id: u32,
    payload: Option<Arc<dyn Contents>>,
}


#[uniffi::export]
impl PayloadWithoutAttachments {
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_f_id(&self) -> u32 { self.f_id }
    pub fn get_payload_content(&self) -> Option<Arc<dyn Contents>> { self.payload.clone() }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        e_id: u8,
        k_id: u8,
        f_id: u32,
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

        Ok(Arc::new(Self {
            i_att: false,
            version,
            e_id,
            k_id,
            f_id,
            payload,
        }))
    }
}


#[uniffi::export]
pub fn deserialize_payload_without_attachments(
    data: &[u8]
) -> Result<Arc<PayloadWithoutAttachments>, PayloadsError> {
    let version = bit_utils::get_bits(&data[0], 0, 6);
    let i_att = bit_utils::is_bit_on(&data[0], 7);
    let k_id = data[1];
    let e_id = bit_utils::get_bits(&data[2], 0, 3);
    let f_id = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
    let cat_id = 0; // TODO("Figure this out")
    let payload = match deserialize_for_content(
        cat_id,
        data[7..].to_vec()
    ) {
        Ok(payload) => Some(payload),
        Err(e) => return Err(ContentDeserializationError) // TODO: put in the error
    };

    Ok(Arc::new( PayloadWithoutAttachments {
        i_att,
        version,
        k_id,
        e_id,
        f_id,
        payload
    }))
}


impl PartialEq for PayloadWithoutAttachments {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.i_att == other.i_att
            && self.f_id == other.f_id
            && self.payload.as_ref()
            .zip(self.payload.as_ref())
            .map_or(false, |(a, b)| Arc::ptr_eq(a, b))
    }
}


#[uniffi::export]
impl Payloads for PayloadWithoutAttachments {
    fn serialize(&self) -> crate::payloads::Result<Vec<u8>> {
        if !self.payload.is_some() {
            return Err(MissingPayload)
        }

        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = bit_utils::put_value(&0, 0, self.version, 1);
        if self.i_att { byte1 = bit_utils::turn_bit_on(&byte1, 7); }
        bytes.push(byte1);

        bytes.push(self.k_id);
        bytes.push(self.e_id);
        bytes.extend(self.f_id.to_le_bytes());

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
    let f_id: u32 = 2;
    let payload: Option<Arc<dyn Contents>> = Some(email);

    let transport_att_false = PayloadWithoutAttachments::new(
        version,
        e_id,
        k_id,
        f_id,
        payload,
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = deserialize_payload_without_attachments(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);
}
