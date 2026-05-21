use std::sync::Arc;
use crate::bit_utils;
use crate::transports::{Transports, TransportsError};
use crate::transports::TransportsError::{DeviceIdTooLarge, SessionIdTooLarge};

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
    payload: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct TransportAttTrueN {
    seg_num: u8,
    sess_id: u8,
    payload: Option<Vec<u8>>,
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
    pub fn get_payload_content(&self) -> Option<Vec<u8>> { self.payload.clone() }

    pub fn new(
        version: u8,
        seg_num: u8,
        sess_id: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        len_att: u16,
        device_id: Option<Vec<u8>>,
        payload: Option<Vec<u8>>,
    ) -> Result<Arc<Self>, TransportsError> {
        let i_did = device_id.as_ref().map_or(false, |v| !v.is_empty());
        if i_did && (device_id.clone().unwrap().len() > u16::MAX as usize) {
            return Err(DeviceIdTooLarge);
        }
        let i_att = payload.as_ref().map_or(false, |v| !v.is_empty());
        let i_end = false; //change value after parsing

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

}

impl TransportAttTrueN {
    pub fn new(
        seg_num: u8,
        sess_id: u8,
        payload: Option<Vec<u8>>,
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

impl Transports for TransportAttTrue {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte1 : u8 = if self.i_did { 1 } else { 0 };
        if self.i_att { bit_utils::turn_bit_on(&byte1, 1); }
        if self.i_end { byte1 = bit_utils::turn_bit_on(&byte1, 2); }
        byte1 = bit_utils::put_value(&byte1, 3, 7, 0);
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
            bytes.extend_from_slice(self.device_id.clone().unwrap().as_slice());
        }

        if self.payload.is_some() {
            bytes.extend_from_slice(self.payload.clone().unwrap().as_slice());
        }

        Ok(bytes)
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        todo!()
    }

}

impl Transports for TransportAttTrueN {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        todo!()
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        todo!()
    }
}
