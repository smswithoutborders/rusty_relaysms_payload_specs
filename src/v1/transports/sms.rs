use std::sync::Arc;
use crate::utils;
use crate::v1::contents::contents_container::V1ContentsContainer;
use crate::v1::contents::V1ContentCategories;
use crate::v1::get_version;
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader};
use crate::v1::payloads::{V1Payloads, V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::PayloadTooLarge;
use crate::v1::transports::{Transport, V1TransportError};

#[derive(Debug, uniffi::Object)]
struct SMS{
    payload: V1Payloads,
}

#[uniffi::export]
impl Transport for SMS {
    fn split(&self) -> Result<Vec<Vec<u8>>, V1TransportError> {
        pub const SEG_0_HEADER_SIZE: u8 = 9;
        pub const SEG_N_HEADER_SIZE: u8 = 3;
        pub const MAX_PAYLOAD_SIZE: u8 = 104; // save space in case encoding required

        let max_value = MAX_PAYLOAD_SIZE - SEG_0_HEADER_SIZE;
        let items = utils::take_n_from(&self.payload, 0, max_value as usize);
        let mut start_index: usize = items.len();

        let payload_seg_0 = match V1PayloadWithAttachmentsHeader::new(
            self.sess_id,
            self.k_id,
            self.t_id,
            self.len_att,
            items,
        ) {
            Ok(transport) => transport,
            Err(e) => { return Err(V1PayloadsError::from(e)); }
        };
        let mut payloads: Vec<Vec<u8>> = Vec::new();
        payloads.push(payload_seg_0.serialize().expect("seg 0 should be serializable"));

        let mut seg_num :u8 = 1;
        let max_value = crate::v1::payloads::payload_container::MAX_PAYLOAD_SIZE - crate::v1::payloads::payload_container::SEG_N_HEADER_SIZE;
        while start_index < self.payload.len() {
            let items = utils::take_n_from(&self.payload, start_index, max_value as usize);
            start_index += items.len();

            if payload.len() as u32 > (MAX_PAYLOAD_SIZE - SEG_N_HEADER_SIZE) as u32 {
                return Err(PayloadTooLarge {
                    current: payload.len() as i32,
                    max: MAX_PAYLOAD_SIZE,
                })
            }
            let payload_seg_n =
                match V1PayloadWithAttachmentsNoHeader::new(
                    get_version(),
                    seg_num,
                    self.sess_id,
                    items,
                ) {
                    Ok(transport) => transport,
                    Err(e) => { return Err(V1PayloadsError::from(e)); }
                };
            payloads.push(payload_seg_n.serialize()
                .expect("seg n should be serializable"));
            seg_num += 1;
        }

        Ok(payloads)
    }

}

