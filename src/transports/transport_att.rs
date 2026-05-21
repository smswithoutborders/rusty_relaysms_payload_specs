use std::sync::Arc;
use crate::transports::{Transports, TransportsError};
use crate::transports::transport_att_true::{TransportAttTrue, TransportAttTrueN};
use crate::transports::TransportsError::{CategoryIdTooLarge, DeviceIdTooLarge, EncryptionIdTooLarge, KeyIdTooLarge, SessionIdTooLarge, VersionTooLarge};
use crate::utils;

const MAX_SPLIT_0: u8 = 134;
const MAX_SPLIT_N: u8 = 136;

#[derive(Debug, uniffi::Object)]
pub struct TransportAtt {
    version: u8,
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

    #[uniffi::constructor]
    pub fn new(
        version: u8,
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

        Ok(Arc::new(Self {
            version,
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
        let mut splits: Vec<Arc<dyn Transports>> = Vec::new();
        if !self.payload.is_some() {
            return Err(TransportsError::EmptyPayload)
        }

        let payload = self.payload.as_ref().unwrap();
        let mut seg_num :u8 = 0;

        let items = utils::take_n_from(payload, 0, MAX_SPLIT_0 as usize);
        let mut start_index: usize = items.len();
        let transport = match TransportAttTrue::new(
            self.version,
            seg_num,
            self.sess_id,
            self.k_id,
            self.e_id,
            self.cat_id,
            payload.len() as u16,
            self.device_id.clone(),
            Option::from(items),
        ) {
            Ok(transport) => transport,
            Err(e) => { return Err(TransportsError::from(e)); }
        };
        splits.push(transport);

        while start_index < payload.len() {
            let items = utils::take_n_from(payload, start_index, MAX_SPLIT_N as usize);
            start_index = items.len();
            let transport = match TransportAttTrueN::new(
                seg_num,
                self.sess_id,
                Option::from(items)
            ) {
                Ok(transport) => transport,
                Err(e) => { return Err(TransportsError::from(e)); }
            };
            splits.push(transport);
            seg_num += 1;
        }

        Ok(splits)
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

