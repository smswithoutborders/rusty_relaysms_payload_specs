use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::{bit_utils, utils, AsAny};
use crate::bit_utils::BitParsingError;
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1ContentError, V1ContentVariation, V1Contents, V1ContentsContainer};
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;
use crate::v1::get_version;
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader, ATTACHMENT_SEG_N_HEADER_SIZE, ATTACHMENT_SEG_O_HEADER_SIZE};
use crate::v1::payloads::payload_without_attachment::V1PayloadWithoutAttachments;
use crate::v1::payloads::V1PayloadsError::PayloadTooLarge;
use crate::v1::transports::{Transports, SMS};

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

    #[error("Payload ({seg_num}) too large; wanted {max} got {current}")]
    PayloadTooLarge {
        current: i32,
        max: u8,
        seg_num: u8,
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

    #[error("Error downcasting content")]
    ErrorDowncastingContent,

    #[error("Attachment length present for serialize: len={len}")]
    AttachmentLengthPresentForSerialize {
        len: u16
    },

    #[error("Inconsistent attachment length and session id")]
    InconsistentLengthSessionId,
}

#[derive(Debug, uniffi::Object)]
pub struct V1Payloads {
    contents: Arc<dyn V1Contents>,
    cat_id: V1ContentCategories,
    k_id: u8,
    len_att: u16,
    t_id: Option<u32>,
    sess_id: Option<u8>,
}

impl PartialEq for V1Payloads {
    fn eq(&self, other: &Self) -> bool {
        self.contents.serialize().unwrap_or_default() ==
            other.contents.serialize().unwrap_or_default() &&
            self.k_id == other.k_id &&
            self.len_att == other.len_att &&
            self.t_id == other.t_id &&
            self.sess_id == other.sess_id
    }
}


#[uniffi::export]
impl V1Payloads {
    pub fn get_payload(&self) -> Arc<dyn V1Contents> { self.contents.clone() }
    pub fn get_kid(&self) -> u8 { self.k_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_sess_id(&self) -> Option<u8> { self.sess_id }


    #[uniffi::constructor]
    pub fn new(
        content: V1ContentVariation,
        k_id: u8,
        len_att: u16,
        t_id: Option<u32>,
        sess_id: Option<u8>,
    ) -> Result<Self> {
        if len_att == 0 && sess_id.is_some() {
            return Err(V1PayloadsError::InconsistentLengthSessionId)
        };

        let cat_id: V1ContentCategories;
        let contents: Arc<dyn V1Contents> = match content {
            V1ContentVariation::EMAIL { value } => {
                cat_id = V1ContentCategories::Email;
                value
            }
            V1ContentVariation::MESSAGE { value } => {
                cat_id = V1ContentCategories::Message;
                value
            }
            V1ContentVariation::TEXT { value } => {
                cat_id = V1ContentCategories::Text;
                value
            }
        };

        Ok(Self {
            contents,
            cat_id,
            k_id,
            len_att,
            t_id,
            sess_id,
        })
    }

    // For content without attachment
    pub fn serialize(&self) -> Result<Vec<u8>> {
        if self.len_att > 0 {
            return Err(V1PayloadsError::AttachmentLengthPresentForSerialize{ len: self.len_att })
        };

        let content = match self.contents.serialize() {
            Ok(payload) => payload,
            Err(error) => { return Err(V1PayloadsError::ErrorFromContent { error }) }
        };

        V1PayloadWithoutAttachments::new(
            self.k_id,
            self.t_id,
            content.as_slice()
        )?.serialize()
    }

    #[uniffi::constructor]
    pub fn deserialize(data: &[u8], cat_id: V1ContentCategories) -> Result<Arc<V1Payloads>> {
        let payload =
            V1PayloadWithoutAttachments::deserialize(data)?;
        let content_cat =
            match V1ContentsContainer::deserialize(
                payload.get_payload_content().as_slice(), cat_id.clone(), 0) {
                Ok(content) => content,
                Err(error) => {
                    return Err(V1PayloadsError::ContentDeserializationError {error}) }
            };
        Ok(Arc::new(V1Payloads::new(
            content_cat,
            payload.get_k_id(),
            0,
            payload.get_t_id(),
            None
        )?))
    }

    #[uniffi::constructor]
    pub fn join(
        payload: Vec<Vec<u8>>,
        cat_id: V1ContentCategories,
    ) -> Result<V1ContentVariation> {
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
    ) -> Result<Vec<Vec<u8>>> {
        let max_transport_payload_size = transport.get_max_payload_size();
        let max_payload_size: u8 = u8::try_from(max_transport_payload_size).unwrap_or(u8::MAX);
        let max_value = max_payload_size - ATTACHMENT_SEG_O_HEADER_SIZE;

        let payload = match self.contents.clone().serialize() {
            Ok(T) => T,
            Err(e) => return Err(V1PayloadsError::ContentSerializationError)
        };
        let items = utils::take_n_from(&payload, 0, max_value as usize);
        let mut start_index: usize = items.len();
        if items.len() as u8 > max_value {
            return Err(PayloadTooLarge {
                current: items.len() as i32,
                max: max_value,
                seg_num: 0
            })
        }

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

        let mut payloads: Vec<Vec<u8>> = Vec::new();
        payloads.push(seg_0);

        let mut seg_num :u8 = 1;
        let max_value = max_payload_size - ATTACHMENT_SEG_N_HEADER_SIZE;
        while start_index < payload.len() {
            let items = utils::take_n_from(&payload, start_index, max_value as usize);
            if items.len() as u8 > max_value {
                return Err(PayloadTooLarge {
                    current: items.len() as i32,
                    max: max_payload_size,
                    seg_num
                })
            }
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
            payloads.push(seg_n);
            seg_num += 1;
        }

        Ok(payloads)
    }
}

#[test]
fn test_payload_without_attachments() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let k_id: u8 = 7;
    let t_id: u32 = 2;
    let sess_id: u8 = 7;

    let contents = V1ContentsContainer::new(
        V1ContentCategories::Message,
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        None
    );

    let transport_att_false = V1Payloads::new(
        contents.content_from().unwrap(),
        k_id,
        0,
        Some(t_id),
        None
    ).unwrap();

    let serialized = transport_att_false.serialize().unwrap();
    let deserialized = V1Payloads::deserialize(
        &serialized, V1ContentCategories::Message).unwrap();
    assert_eq!(Arc::new(transport_att_false), deserialized);
}

#[test]
fn test_payload_with_attachments() {
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;

    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let email = V1Emails::new(
        to.to_vec(),
        body.to_vec(),
        Option::from(subject.to_vec()),
        Some(att)
    ).unwrap();

    let sess_id: u8 = 15;
    let k_id: u8 = 13;
    let t_id: Option<u32> = Option::from(255);

    let payload_with_attachment = V1Payloads::new(
        V1ContentVariation::EMAIL { value: email.clone() },
        k_id,
        len_att,
        t_id,
        Some(sess_id)
    ).unwrap();

    let split = payload_with_attachment.split(Arc::new(SMS)).unwrap();

    let joined = V1Payloads::join(split, V1ContentCategories::Email).unwrap();
    let V1ContentVariation::EMAIL { value } = joined else {
        panic!("expected an EMAIL")
    };
    assert_eq!(value, email.clone());
}