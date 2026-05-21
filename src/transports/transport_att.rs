use std::sync::Arc;
use crate::bit_utils;
use crate::contents::Contents;
use crate::contents::email::{deserialize_email, init_email};
use crate::transports::{Transports, TransportsError};
use crate::transports::TransportsError::{CategoryIdTooLarge, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, SegmentLessThanOne, SessionIdTooLarge, VersionTooLarge};

const MAX_SPLIT: u8 = 130;

#[derive(Debug, uniffi::Object)]
pub struct TransportAtt {
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

#[uniffi::export]
impl TransportAtt {
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

    #[uniffi::constructor]
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
        if version > (2u8.pow(4) - 1) {
            return Err(VersionTooLarge);
        }
        if sess_id > (2u8.pow(4) - 1) {
            return Err(SessionIdTooLarge);
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

    /**
    Assumption, payload already processed just needs splitting for transmission
    **/
    pub fn split(&self) -> crate::transports::Result<Vec<Arc<dyn Transports>>> {
        todo!()
    }

}

impl PartialEq for TransportAtt {
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

#[uniffi::export]
impl Transports for TransportAtt {
    fn serialize(&self) -> crate::transports::Result<Vec<u8>> {
        todo!()
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        todo!()
    }

}



