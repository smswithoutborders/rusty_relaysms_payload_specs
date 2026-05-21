use std::sync::Arc;
use crate::contents::Contents;
use crate::transports::{Transports, TransportsError};
use crate::transports::TransportsError::{CategoryIdTooLarge, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, SessionIdTooLarge, VersionTooLarge};

#[derive(Debug)]
pub struct TransportAttFalse {
    i_did: bool,
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
            version,
            e_id,
            k_id,
            cat_id,
            device_id,
            payload,
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
        todo!()
    }

    fn equals(&self, other: Arc<dyn Transports>) -> bool {
        todo!()
    }

}
