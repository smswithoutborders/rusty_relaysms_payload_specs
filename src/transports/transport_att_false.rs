use std::sync::Arc;
use crate::bit_utils;
use crate::contents::{deserialize_for_content, Contents};
use crate::contents::email::Emails;
use crate::transports::{Transports, TransportsError};
use crate::transports::TransportsError::{CategoryIdTooLarge, ContentDeserializationError, ContentSerializationError, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, MissingDeviceID, MissingPayload, SessionIdTooLarge, VersionTooLarge};

#[derive(Debug)]
pub struct TransportAttFalse {
    i_did: bool,
    i_att: bool,
    i_end: bool,
    version: u8,
    k_id: u8,
    e_id: u8,
    cat_id: u8,
    device_id: Option<Vec<u8>>,
    payload: Option<Arc<dyn Contents>>,
}

impl TransportAttFalse {
    pub fn get_i_did(&self) -> bool { self.i_did }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_cat_id(&self) -> u8 { self.cat_id }
    pub fn get_device_id(&self) -> Option<Vec<u8>> { self.device_id.clone() }
    pub fn get_payload_content(&self) -> Option<Arc<dyn Contents>> { self.payload.clone() }


    #[uniffi::constructor]
    pub fn init() -> Result<Arc<Self>, TransportsError> {
        Ok(Arc::new(Self {
            i_did: false,
            i_att: false,
            i_end: false,
            version: 255,
            k_id: 255,
            e_id: 255,
            cat_id: 255,
            device_id: None,
            payload: None,
        }))
    }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        device_id: Option<Vec<u8>>,
        payload: Option<Arc<dyn Contents>>,
    ) -> Result<Arc<Self>, TransportsError> {
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

        let i_did = device_id.as_ref().map_or(false, |v| !v.is_empty());
        if i_did && (device_id.clone().unwrap().len() > u16::MAX as usize) {
            return Err(DeviceIdTooLarge);
        }

        Ok(Arc::new(Self {
            i_did,
            i_att: false,
            i_end: false,
            version,
            e_id,
            k_id,
            cat_id,
            device_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, TransportsError> {
        let i_did = bit_utils::is_bit_on(&data[0], 0);
        let i_att = bit_utils::is_bit_on(&data[0], 1);
        let i_end = bit_utils::is_bit_on(&data[0], 2);
        let version = bit_utils::get_bits(&data[0], 3, 7);
        let k_id = data[1];
        let e_id = bit_utils::get_bits(&data[2], 0, 3);
        let cat_id = bit_utils::get_bits(&data[2], 4, 7);
        let device_id = Option::from(data[3..(3 + 16)].to_vec());

        let payload = match deserialize_for_content(
            cat_id,
            data[(3 + 16)..].to_vec()
        ) {
            Ok(payload) => Some(payload),
            Err(e) => return Err(ContentDeserializationError) // TODO: put in the error
        };

        Ok(Arc::new( Self {
            i_did,
            i_att,
            i_end,
            version,
            k_id,
            e_id,
            cat_id,
            device_id,
            payload
        }))
    }
}

impl PartialEq for TransportAttFalse {
    fn eq(&self, other: &Self) -> bool {
        self.i_did == other.i_did
            && self.version == other.version
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.cat_id == other.cat_id
            && self.device_id == other.device_id
            && self.payload.as_ref()
            .zip(self.payload.as_ref())
            .map_or(false, |(a, b)| Arc::ptr_eq(a, b))
    }
}

impl Transports for TransportAttFalse {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        if !self.payload.is_some() {
            return Err(MissingPayload)
        }

        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = if self.i_did { 1 } else { 0 };
        if self.i_att { bit_utils::turn_bit_on(&byte1, 1); }
        if self.i_end { byte1 = bit_utils::turn_bit_on(&byte1, 2); }
        byte1 = bit_utils::put_value(&byte1, 3, self.version, 4);
        bytes.push(byte1);
        bytes.push(self.k_id);
        let byte2 = bit_utils::put_value(
            &self.e_id,
            0,
            self.cat_id,
            4
        );
        bytes.push(byte2);

        if self.i_did {
            if !self.device_id.is_some() {
                return Err(MissingDeviceID)
            }
            bytes.extend_from_slice(self.device_id.as_ref().unwrap());
        }
        let payload = match self.payload.as_ref().unwrap().serialize() {
            Ok(payload) => payload,
            Err(e) => return Err(ContentSerializationError)
        };
        bytes.extend(payload);
        Ok(bytes)
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
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
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let cat_id: u8 = email.get_cat_id();
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());
    let payload: Option<Arc<dyn Contents>> = Some(email);

    let transport_att_false = TransportAttFalse::new(
        version,
        e_id,
        k_id,
        cat_id,
        device_id,
        payload,
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = TransportAttFalse::init().unwrap()
        .deserialize(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);
}
