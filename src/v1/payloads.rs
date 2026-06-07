use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::{bit_utils, utils};
use crate::bit_utils::BitParsingError;
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1ContentError, V1ContentsContainer};
use crate::v1::get_version;
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader, ATTACHMENT_SEG_N_HEADER_SIZE, ATTACHMENT_SEG_O_HEADER_SIZE};
use crate::v1::payloads::V1PayloadsError::PayloadTooLarge;
use crate::v1::transports::Transports;

pub mod payload_with_attachments;
pub mod payload_without_attachment;

type Result<T> = std::result::Result<T, V1PayloadsError>;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum V1PayloadsError {
    #[error("Version too large")]
    VersionTooLarge,

    #[error("Segment less than one")]
    SegmentLessThanOne,

    #[error("Session ID too large")]
    SessionIdTooLarge,

    #[error("Key ID too large")]
    KeyIdTooLarge,

    #[error("Category ID too large")]
    CategoryIdTooLarge,

    #[error("Device ID too large")]
    DeviceIdTooLarge,

    #[error("Empty payload")]
    EmptyPayload,
    
    #[error("Error parsing bits - {error}")]
    ErrorParsingBits {
        error: BitParsingError,
    },

    #[error("Content deserialization error: {error}")]
    ContentDeserializationError {
        error: V1ContentError
    },
    
    #[error("Content serialization error")]
    ContentSerializationError,

    #[error("Missing device ID")]
    MissingDeviceID,

    #[error("Missing payload")]
    MissingPayload,

    #[error("Payload too large; wanted {max} got {current}")]
    PayloadTooLarge {
        current: i32,
        max: u8,
    },

    #[error("Header too large; wanted {max} got {current}")]
    HeaderTooLarge {
        current: i32,
        max: u8,
    },

    #[error("N Header for seg: {segment} too large; wanted {max} got {current}")]
    NHeaderTooLarge {
        segment: u8,
        current: i32,
        max: u8,
    },

    #[error("Invalid start segment number. Expected 0 got {seg_num}")]
    InvalidStartSegmentNumber { seg_num: u8 },

    #[error("Session IDs differ for {seg_num}; expected {expected}, got {actual}")]
    DifferingSessionIds {
        expected: u8,
        actual: u8,
        seg_num: u8,
    },

    #[error("Error from content: {error}")]
    ErrorFromContent {
        error: V1ContentError,
    },
}

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1Payloads {
    payload: Arc<V1ContentsContainer>,
    k_id: u8,
    len_att: u16,
    t_id: Option<u32>,
    sess_id: Option<u8>,
}


#[uniffi::export]
impl V1Payloads {
    pub fn get_payload(&self) -> Arc<V1ContentsContainer> { self.payload.clone() }
    pub fn get_kid(&self) -> u8 { self.k_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_sess_id(&self) -> Option<u8> { self.sess_id }


    #[uniffi::constructor]
    pub fn new(
        payload: Arc<V1ContentsContainer>,
        k_id: u8,
        len_att: u16,
        t_id: Option<u32>,
        sess_id: Option<u8>,
    ) -> Self {
        Self {
            payload,
            k_id,
            len_att,
            t_id,
            sess_id,
        }
    }

    #[uniffi::constructor]
    pub fn join(
        payload: Vec<Vec<u8>>,
        cat_id: V1ContentCategories,
    ) -> Result<V1ContentsContainer> {
        let seg_0 =
            match V1PayloadWithAttachmentsHeader::deserialize(&payload[0]) {
                Ok(T) => T,
                Err(T) => return Err(T),
            };

        let mut content = seg_0.get_payload_content();
        let sess_id = seg_0.get_sess_id();
        for i in 1..payload.len() {
            let seg_n =
                match V1PayloadWithAttachmentsNoHeader::deserialize(&payload[i]) {
                    Ok(T) => T,
                    Err(T) => return Err(T),
                };
            if seg_n.get_sess_id() != sess_id {
                return Err(V1PayloadsError::DifferingSessionIds {
                    expected: sess_id,
                    actual: seg_n.get_sess_id(),
                    seg_num: seg_n.get_seg_num()
                })
            }
            content.extend(seg_n.get_payload());
        };

        match V1ContentsContainer::deserialize(
            content.as_slice(),
            cat_id,
            seg_0.get_len_att()
        ) {
            Ok(T) => Ok(T),
            Err(T) => { Err(V1PayloadsError::ErrorFromContent { error: T }) }
        }
    }

    pub fn split(
        &self,
        transport: Arc<dyn Transports>,
        cat_id: V1ContentCategories
    ) -> Result<Vec<Vec<u8>>> {
        let max_transport_payload_size = transport.get_max_payload_size();
        let max_payload_size: u8 = u8::try_from(max_transport_payload_size).unwrap_or(u8::MAX);
        let max_value = max_payload_size - ATTACHMENT_SEG_O_HEADER_SIZE;

        let payload = match self.payload.serialize(cat_id) {
            Ok(T) => T,
            Err(e) => return Err(V1PayloadsError::ContentSerializationError)
        };
        let items = utils::take_n_from(&payload, 0, max_value as usize);
        let mut start_index: usize = items.len();

        let payload_seg_0 = match V1PayloadWithAttachmentsHeader::new(
            self.sess_id.unwrap(),
            self.k_id,
            self.t_id,
            self.len_att,
            items,
        ) {
            Ok(transport) => transport,
            Err(e) => { return Err(V1PayloadsError::from(e)); }
        };
        let seg_0 = payload_seg_0.serialize().expect("seg 0 should be serializable");
        if seg_0.len() as u32 > (max_payload_size - ATTACHMENT_SEG_O_HEADER_SIZE) as u32 {
            return Err(PayloadTooLarge {
                current: seg_0.len() as i32,
                max: max_payload_size,
            })
        }
        let mut payloads: Vec<Vec<u8>> = Vec::new();
        payloads.push(seg_0);

        let mut seg_num :u8 = 1;
        let max_value = max_payload_size - ATTACHMENT_SEG_N_HEADER_SIZE;
        while start_index < payload.len() {
            let items = utils::take_n_from(&payload, start_index, max_value as usize);
            start_index += items.len();

            let payload_seg_n =
                match V1PayloadWithAttachmentsNoHeader::new(
                    get_version(),
                    seg_num,
                    self.sess_id.unwrap(),
                    items,
                ) {
                    Ok(transport) => transport,
                    Err(e) => { return Err(V1PayloadsError::from(e)); }
                };
            let seg_n = payload_seg_n.serialize().expect("seg n should be serializable");
            if seg_n.len() as u32 > (max_payload_size - ATTACHMENT_SEG_N_HEADER_SIZE) as u32 {
                return Err(PayloadTooLarge {
                    current: seg_n.len() as i32,
                    max: max_payload_size,
                })
            }
            payloads.push(seg_n);
            seg_num += 1;
        }

        Ok(payloads)
    }
}
