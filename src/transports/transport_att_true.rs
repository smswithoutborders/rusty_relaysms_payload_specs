use std::sync::Arc;
use crate::bit_utils;
use crate::contents::{deserialize_for_content, Contents};
use crate::contents::email::Emails;
use crate::transports::{Transports, TransportsError};
use crate::transports::transport_att_false::TransportAttFalse;
use crate::transports::TransportsError::{ContentDeserializationError, DeviceIdTooLarge, SessionIdTooLarge};

#[derive(Debug)]
pub struct TransportAttTrue {
    i_did: bool,
    i_att: bool,
    i_end: bool,
    version: u8,
    seg_num: u8,
    sess_id: u8,
    k_id: u8,
    e_id: u8,
    cat_id: u8,
    len_att: u16,
    device_id: Option<Vec<u8>>,
    payload: Vec<u8>,
}

#[derive(Debug)]
pub struct TransportAttTrueN {
    seg_num: u8,
    sess_id: u8,
    payload: Vec<u8>
}

impl TransportAttTrue {
    pub fn get_i_did(&self) -> bool { self.i_did }
    pub fn get_i_att(&self) -> bool { self.i_att }
    pub fn get_i_cont(&self) -> bool { self.i_end }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_seg_num(&self) -> u8 { self.seg_num }
    pub fn get_sess_id(&self) -> u8 { self.sess_id }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_cat_id(&self) -> u8 { self.cat_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_device_id(&self) -> Option<Vec<u8>> { self.device_id.clone() }
    pub fn get_payload_content(&self) -> Vec<u8> { self.payload.clone() }

    pub fn init() -> Result<Arc<Self>, TransportsError> {
        Ok(Arc::new(Self {
            i_did: false,
            i_att: false,
            i_end: false,
            version: 255,
            seg_num: 255,
            sess_id: 255,
            k_id: 255,
            e_id: 255,
            cat_id: 255,
            len_att: 0,
            device_id: None,
            payload: Vec::new(),
        }))
    }

    pub fn new(
        version: u8,
        seg_num: u8,
        sess_id: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        len_att: u16,
        device_id: Option<Vec<u8>>,
        payload: Vec<u8>,
    ) -> Result<Arc<Self>, TransportsError> {
        let i_did = device_id.as_ref().map_or(false, |v| !v.is_empty());
        if i_did && (device_id.clone().unwrap().len() > u16::MAX as usize) {
            return Err(DeviceIdTooLarge);
        }
        let i_att = !payload.is_empty();
        let i_end = false; // TODO: change value after parsing

        Ok(Arc::new(Self {
            i_did,
            i_att,
            i_end,
            version,
            seg_num,
            sess_id,
            e_id,
            k_id,
            cat_id,
            len_att,
            device_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, TransportsError> {
        let i_did = bit_utils::is_bit_on(&data[0], 0);
        let i_att = bit_utils::is_bit_on(&data[0], 1);
        let i_end = bit_utils::is_bit_on(&data[0], 2);
        let version = bit_utils::get_bits(&data[0], 3, 7);
        let seg_num = data[1];
        let sess_id = bit_utils::get_bits(&data[2], 0, 3);
        let k_id = bit_utils::get_bits(&data[2], 4, 7);
        let e_id = bit_utils::get_bits(&data[3], 0, 2);
        let cat_id = bit_utils::get_bits(&data[3], 3, 6);
        let len_att = u16::from_le_bytes([data[4], data[5]]);

        let mut current_index: usize = 6;
        let device_id = if i_did {
            let device_id = Option::from(data[6..(6 + 16)].to_vec());
            current_index += 16;
            device_id
        } else { None };

        let payload = data[current_index..].to_vec();

        Ok(Arc::new(Self {
            i_did,
            i_att,
            i_end,
            version,
            seg_num,
            sess_id,
            k_id,
            e_id,
            cat_id,
            len_att,
            device_id,
            payload
        }))
    }
}

impl TransportAttTrueN {
    pub fn init() -> Result<Arc<Self>, TransportsError> {
        Ok(Arc::new(Self {
            seg_num: 255,
            sess_id: 255,
            payload: Vec::new(),
        }))
    }

    pub fn new(
        seg_num: u8,
        sess_id: u8,
        payload: Vec<u8>
    ) -> Result<Arc<Self>, TransportsError> {
        if sess_id > (2u8.pow(4) - 1) {
            return Err(SessionIdTooLarge);
        }
        Ok(Arc::new(Self {
            seg_num,
            sess_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, TransportsError> {
        let seg_num = data[0];
        let sess_id = data[1];
        let payload = data[2..].to_vec();

        Ok(Arc::new(Self {
            seg_num,
            sess_id,
            payload
        }))
    }
}

impl Transports for TransportAttTrue {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = if self.i_did { 1 } else { 0 };
        if self.i_att { byte1 = bit_utils::turn_bit_on(&byte1, 1); }
        if self.i_end { byte1 = bit_utils::turn_bit_on(&byte1, 2); }
        byte1 = bit_utils::put_value(&byte1, 3, self.version, 4);
        bytes.push(byte1);

        bytes.push(self.seg_num);

        let byte3 = bit_utils::put_value(&self.sess_id, 4, self.k_id, 4);
        bytes.push(byte3);

        let byte4 = bit_utils::put_value(&self.e_id, 3, self.cat_id, 4);
        bytes.push(byte4);

        let len_att = u16::to_le_bytes(self.len_att);
        bytes.push(len_att[0]);
        bytes.push(len_att[1]);

        if self.device_id.is_some() {
            bytes.extend(self.device_id.clone().unwrap());
        }

        bytes.extend(self.payload.clone());

        Ok(bytes)
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

}

impl Transports for TransportAttTrueN {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        let mut payload = vec![self.sess_id, self.seg_num];
        payload.extend(self.payload.clone());
        Ok(payload)
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}


impl PartialEq for TransportAttTrue {
    fn eq(&self, other: &Self) -> bool {
        self.i_did == other.i_did
            && self.i_att == other.i_att
            && self.i_end == other.i_end
            && self.version == other.version
            && self.seg_num == other.seg_num
            && self.sess_id == other.sess_id
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.cat_id == other.cat_id
            && self.len_att == other.len_att
            && self.device_id == other.device_id
            && self.payload == other.payload
    }
}

impl PartialEq for TransportAttTrueN {
    fn eq(&self, other: &Self) -> bool {
        self.seg_num == other.seg_num
            && self.sess_id == other.sess_id
            && self.payload == other.payload
    }
}


#[test]
fn att_true_n_serialize() {
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
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let cat_id: u8 = email.get_cat_id();
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());

    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;
    let mut payload = att;
    payload.extend(email.serialize().unwrap());

    let transport_att_false = TransportAttTrue::new(
        version,
        seg_num,
        sess_num,
        e_id,
        k_id,
        cat_id,
        len_att,
        device_id,
        payload.clone(),
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = TransportAttTrue::init().unwrap()
        .deserialize(&serialized).unwrap();
    assert_eq!(transport_att_false, deserialized);

    let transport_att_n_false = TransportAttTrueN::new(
        seg_num,
        sess_num,
        payload,
    ).unwrap();

    let serialized = transport_att_n_false.serialize().unwrap();
    let deserialized = TransportAttTrueN::init().unwrap()
        .deserialize(&serialized).unwrap();
    assert_eq!(transport_att_n_false, deserialized);
}
