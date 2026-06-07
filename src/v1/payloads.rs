use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::bit_utils;
use crate::bit_utils::BitParsingError;
use crate::v1::contents::contents_container::V1ContentsContainer;
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1ContentError};
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader};

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

#[derive(Debug, uniffi::Enum)]
pub enum V1Payloads {
    WithAttachments,
    WithoutAttachments,
}


#[uniffi::export]
impl V1Payloads {

    #[uniffi::constructor]
    pub fn new(
        payload: Vec<u8>,
        k_id: u8,
        len_att: u16,
        t_id: Option<u32>,
        sess_id: Option<u8>,
    ) -> Result<Self> {
        todo!()
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
}
