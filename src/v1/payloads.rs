use std::any::Any;
use std::cmp::max;
use std::fmt::Debug;
use std::ops::Div;
use std::sync::Arc;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use crate::{bit_utils, utils, AsAny};
use crate::bit_utils::BitParsingError;
use crate::utils::{calculate_b64_min_size};
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1ContentError, V1Contents, V1ContentsContainer};
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;
use crate::v1::get_version;
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader, ATTACHMENT_SEG_N_HEADER_SIZE, ATTACHMENT_SEG_O_HEADER_SIZE, ATTACHMENT_SEG_O_TID_HEADER_SIZE};
use crate::v1::payloads::payload_without_attachment::V1PayloadWithoutAttachments;
use crate::v1::payloads::V1PayloadsError::PayloadTooLarge;
use crate::v1::transports::{Transports};

pub mod payload_with_attachments;
pub mod payload_without_attachment;

type Result<T> = std::result::Result<T, V1PayloadsError>;

#[derive(PartialEq, Debug, thiserror::Error, uniffi::Error)]
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

    #[error("Payload serialization error")]
    PayloadSerializationError,

    #[error("Payload deserialization error")]
    PayloadDeserializationError,

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

    #[error("Serializer needs session id")]
    SerializerNeedsSessionId,

    #[error("Protocol is not supported")]
    UnsupportedProtocol,

    #[error("Protocol is not supported")]
    PayloadTooShort,

    #[error("Segments missing")]
    MissingSegments,

    #[error("No last segment")]
    NoLastSegments,
}

#[derive(Debug, uniffi::Object, Serialize, Deserialize)]
pub struct V1Payloads {
    // contents: Arc<dyn V1Contents>, // cannot handle encrypted payload
    contents: Vec<u8>, // encrypted payload should go here
    k_id: u8,
    len_att: u16,
    t_id: Option<u32>,
    sess_id: Option<u8>,
}

impl PartialEq for V1Payloads {
    fn eq(&self, other: &Self) -> bool {
        self.contents == other.contents &&
            self.k_id == other.k_id &&
            self.len_att == other.len_att &&
            self.t_id == other.t_id &&
            self.sess_id == other.sess_id
    }
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum V1PayloadsTypes {
    WithoutAttachment = 0x0,
    WithAttachmentHeader = 0x1,
    WithAttachmentNoHeader = 0x2,
}

#[derive(uniffi::Enum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u64)]
pub enum V1PayloadsSupportedProtocols {
    OAuth20 = 0,
    Pnba = 1,
}

#[uniffi::export]
pub fn v1_payload_support_protocols_from_u8(value: u8) -> Result<V1PayloadsSupportedProtocols> {
    match value {
        0x0 => Ok(V1PayloadsSupportedProtocols::OAuth20),
        0x1 => Ok(V1PayloadsSupportedProtocols::Pnba),
        _ => Err(V1PayloadsError::UnsupportedProtocol),
    }
}

#[uniffi::export]
impl V1Payloads {
    pub fn get_content(&self) -> Vec<u8> { self.contents.clone() }
    pub fn get_kid(&self) -> u8 { self.k_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_sess_id(&self) -> Option<u8> { self.sess_id }


    #[uniffi::constructor]
    pub fn new(
        contents: Vec<u8>,
        k_id: u8,
        len_att: u16,
        t_id: Option<u32>,
        sess_id: Option<u8>,
    ) -> Result<Self> {
        if len_att == 0 && sess_id.is_some() {
            return Err(V1PayloadsError::InconsistentLengthSessionId)
        };

        Ok(Self {
            contents,
            k_id,
            len_att,
            t_id,
            sess_id,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        match serde_json::to_vec(&self) {
            Ok(v) => Ok(v),
            Err(e) => Err(V1PayloadsError::PayloadSerializationError)
        }
    }

    #[uniffi::constructor]
    pub fn deserialize(bytes: Vec<u8>) -> Result<Self> {
        match serde_json::from_slice::<V1Payloads>(&bytes) {
            Ok(v) => Ok(v),
            Err(e) => Err(V1PayloadsError::PayloadSerializationError)
        }
    }

    pub fn serialize_with_attachment(&self) -> Result<Vec<u8>> {
        if self.sess_id.is_none() {
            return Err(V1PayloadsError::SerializerNeedsSessionId)
        }

        V1PayloadWithAttachmentsHeader::new(
            self.sess_id.unwrap(),
            self.k_id,
            self.t_id,
            self.len_att,
            self.contents.clone()
        )?.serialize()
    }

    #[uniffi::constructor]
    pub fn deserialize_with_attachment(data: &[u8]) -> Result<Arc<V1Payloads>> {
        let payload =
            V1PayloadWithAttachmentsHeader::deserialize(data)?;
        Ok(Arc::new(V1Payloads::new(
            payload.get_content(),
            payload.get_k_id(),
            payload.get_len_att(),
            payload.get_t_id(),
            Some(payload.get_sess_id())
        )?))
    }

    // For content without attachment
    pub fn serialize_without_attachment(&self) -> Result<Vec<u8>> {
        if self.len_att > 0 {
            return Err(V1PayloadsError::AttachmentLengthPresentForSerialize{ len: self.len_att })
        };

        V1PayloadWithoutAttachments::new(
            self.k_id,
            self.t_id,
            self.contents.as_slice()
        )?.serialize()
    }

    #[uniffi::constructor]
    pub fn deserialize_without_attachment(data: &[u8]) -> Result<Arc<V1Payloads>> {
        let payload =
            V1PayloadWithoutAttachments::deserialize(data)?;
        Ok(Arc::new(V1Payloads::new(
            payload.get_payload_content(),
            payload.get_k_id(),
            0,
            payload.get_t_id(),
            None
        )?))
    }

    #[uniffi::constructor]
    pub fn join(s_payload: Vec<Vec<u8>>) -> Result<V1Payloads> {
        let mut payload : Vec<Vec<u8>> = vec![Vec::new(); s_payload.len()];
        for p in s_payload {
            let seg = BASE64_STANDARD.decode(p)
                .expect("Payload should be able to decode");
            let seg_num = match v1_get_payload_segment_number(seg.as_slice()) {
                Ok(seg_num) => seg_num as usize,
                Err(e) => return Err(e)
            };
            if seg_num >= payload.len() {
                return Err(V1PayloadsError::MissingSegments)
            }
            payload[seg_num] = seg;
        }

        // TODO: test this
        if let Some(last_seg) = payload.last() {
            if v1_get_payload_segment_number(last_seg) == Ok(0) ||
                !v1_get_is_last_segment(last_seg) {
                return Err(V1PayloadsError::NoLastSegments)
            }
        }

        // // TODO: test this
        // for p in payload.clone() {
        //     if p.is_empty() {
        //         return Err(V1PayloadsError::MissingSegments)
        //     }
        // }

        let seg_0 =
            match V1PayloadWithAttachmentsHeader::deserialize(payload[0].as_slice()) {
                Ok(T) => T,
                Err(T) => return Err(T),
            };

        let mut content = seg_0.get_content();
        let sess_id = seg_0.get_sess_id();

        for i in 1..payload.len() {
            let seg_n =
                match V1PayloadWithAttachmentsNoHeader::deserialize(payload[i].as_slice()) {
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

        V1Payloads::new(
            content,
            seg_0.get_k_id(),
            seg_0.get_len_att(),
            seg_0.get_t_id(),
            Some(seg_0.get_sess_id()),
        )
    }

    pub fn split(&self, transport: Transports) -> Result<Vec<Vec<u8>>> {
        // TODO(There's a risk here with minus overflow if data is less than header)
        // TODO(Check for the risk above)
        let max_transport_payload_size = transport.get_max_payload_size();
        let max_payload_size: u8 = u8::try_from(max_transport_payload_size).unwrap_or(u8::MAX);
        let header: u8 = if self.t_id.is_some() {
            ATTACHMENT_SEG_O_TID_HEADER_SIZE
        } else {
            ATTACHMENT_SEG_O_HEADER_SIZE
        };

        let min_take_for_base64 = calculate_b64_min_size(max_payload_size as usize) as u8;

        let items = utils::take_n_from(
            &self.contents, 0, min_take_for_base64 as usize - header as usize);
        let mut start_index: usize = items.len();
        if items.len() as u8 > max_payload_size {
            return Err(PayloadTooLarge {
                current: items.len() as i32,
                max: max_payload_size,
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
        let seg_0 = BASE64_STANDARD.encode(seg_0);
        let mut payloads: Vec<Vec<u8>> = Vec::new();

        payloads.push(seg_0.as_bytes().to_vec());

        let mut seg_num :u8 = 1;

        loop {
            let items = utils::take_n_from(
                &self.contents, start_index,
                min_take_for_base64 as usize - ATTACHMENT_SEG_N_HEADER_SIZE as usize
            );
            if items.len() as u8 > max_payload_size {
                return Err(PayloadTooLarge {
                    current: items.len() as i32,
                    max: max_payload_size,
                    seg_num
                })
            }
            start_index += items.len();
            let i_l = start_index  >= self.contents.len();

            let payload_seg_n =
                match V1PayloadWithAttachmentsNoHeader::new(
                    get_version(),
                    seg_num,
                    self.sess_id.unwrap(),
                    i_l,
                    items,
                ) {
                    Ok(transport) => transport,
                    Err(e) => { return Err(V1PayloadsError::from(e)); }
                };
            let seg_n_b64 = payload_seg_n.serialize().expect("seg n should be serializable");
            let seg_n = BASE64_STANDARD.encode(seg_n_b64.clone());
            payloads.push(seg_n.as_bytes().to_vec());

            if i_l {
                assert!(v1_get_is_last_segment(seg_n_b64.as_slice()));
                break
            }

            seg_num += 1;
        }
        Ok(payloads)
    }

}

#[uniffi::export]
pub fn v1_get_payload_session_id(data: &[u8]) -> Result<u8> {
    if data.len() < 2 {
        return Err(V1PayloadsError::PayloadTooShort)
    }
    Ok(match bit_utils::bit_wrap(
        &data[0], 5, &data[1], 3) {
        Ok(s) => s,
        Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
    })
}


// TODO: test this
pub fn v1_get_payload_segment_number(data: &[u8]) -> Result<u8> {
    if data.len() < 2 {
        return Err(V1PayloadsError::PayloadTooShort)
    }
    let sn = match bit_utils::bit_wrap(
        &data[1], 4, &data[2], 3) {
        Ok(s) => s,
        Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
    };
    Ok(sn)
}

#[uniffi::export]
pub fn v1_get_is_last_segment(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false
    }
    bit_utils::is_bit_on(&data[2], 4)
}


#[uniffi::export]
pub fn v1_get_payload_type(data: &[u8]) -> Result<V1PayloadsTypes> {
    let i_att = bit_utils::is_bit_on(&data[0], 4);
    let seg_num: u8 = match bit_utils::bit_wrap(
        &data[1], 4, &data[2], 3) {
        Ok(s) => s,
        Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
    };
    if i_att {
        return if seg_num > 0 {
            Ok(V1PayloadsTypes::WithAttachmentNoHeader)
        } else {
            Ok(V1PayloadsTypes::WithAttachmentHeader)
        }
    }
    Ok(V1PayloadsTypes::WithoutAttachment)
}

#[test]
fn test_payload_without_attachments() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let k_id: u8 = 7;
    // let t_id: u32 = 2;
    let t_id: u32 = 0xFFFFFFFF;
    let cat_id = V1ContentCategories::Bridge;

    let contents = V1ContentsContainer::new(
        cat_id.clone(),
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        None
    );

    let transport_att_false = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        0,
        Some(t_id),
        None
    ).unwrap();

    let serialized = transport_att_false.serialize_without_attachment().unwrap();
    let t = v1_get_payload_type(serialized.as_slice()).unwrap();
    assert_eq!(V1PayloadsTypes::WithoutAttachment, t);

    let deserialized = V1Payloads::deserialize_without_attachment(&serialized).unwrap();
    assert_eq!(Arc::new(transport_att_false), deserialized);

    let content = V1ContentsContainer::deserialize(
        deserialized.get_content().as_slice(), cat_id.clone(), 0).unwrap();
    assert_eq!(contents, content);

    // No token id
    let contents = V1ContentsContainer::new(
        cat_id.clone(),
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        None
    );

    let transport_att_false = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        0,
        None,
        None
    ).unwrap();

    let serialized = transport_att_false.serialize_without_attachment().unwrap();
    let t = v1_get_payload_type(serialized.as_slice()).unwrap();
    assert_eq!(V1PayloadsTypes::WithoutAttachment, t);

    let deserialized = V1Payloads::deserialize_without_attachment(&serialized).unwrap();
    assert_eq!(Arc::new(transport_att_false), deserialized);

    let content = V1ContentsContainer::deserialize(
        deserialized.get_content().as_slice(), cat_id, 0).unwrap();
    assert_eq!(contents, content);
}

#[test]
fn test_payload_with_attachments() {
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;

    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let sess_id: u8 = 15;
    let k_id: u8 = 13;
    let t_id: Option<u32> = Option::from(255);

    let cat_id = V1ContentCategories::Message;
    let contents = V1ContentsContainer::new(
        cat_id.clone(),
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        Some(att)
    );

    let payload_att = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        len_att,
        // t_id,
        None,
        Some(sess_id)
    ).unwrap();

    let mut split = payload_att.split(Transports::Sms).unwrap();
    assert_eq!(160, split[0].len() as u32);
    assert_eq!(160, split[1].len() as u32);
    let mut missing_segment_split = split.clone();
    missing_segment_split.remove(3);

    let missing_joined = V1Payloads::join(missing_segment_split);
    assert!(missing_joined.is_err());
    assert_eq!(missing_joined.err().unwrap(), V1PayloadsError::MissingSegments);

    let seg_0 = BASE64_STANDARD.decode(&split[0]).unwrap();
    let t = v1_get_payload_type(seg_0.as_slice()).unwrap();
    assert_eq!(V1PayloadsTypes::WithAttachmentHeader, t);

    let seg_n = BASE64_STANDARD.decode(&split.last().unwrap()).unwrap();
    let t = v1_get_payload_type(seg_n.as_slice()).unwrap();
    assert_eq!(V1PayloadsTypes::WithAttachmentNoHeader, t);
    assert!(v1_get_is_last_segment(seg_n.as_slice()));

    let s = v1_get_payload_session_id(seg_0.as_slice()).unwrap();
    assert_eq!(sess_id, s);

    let s = v1_get_payload_session_id(seg_n.as_slice()).unwrap();
    assert_eq!(sess_id, s);

    let sn = v1_get_payload_segment_number(seg_0.as_slice()).unwrap();
    assert_eq!(sn, 0);

    // let sn = v1_get_payload_segment_number(seg_n.as_slice()).unwrap();
    // assert_eq!(sn, 1);

    let mut rng = rand::rng();
    split.shuffle(&mut rng);


    let joined = V1Payloads::join(split.clone()).unwrap();
    assert_eq!(payload_att, joined);

    let payload_att = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        len_att,
        t_id,
        Some(sess_id)
    ).unwrap();

    let split = payload_att.split(Transports::Sms).unwrap();

    let joined = V1Payloads::join(split).unwrap();
    assert_eq!(payload_att, joined)
}

#[test]
fn test_serialization() {
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;

    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let sess_id: u8 = 15;
    let k_id: u8 = 13;
    let t_id: Option<u32> = Option::from(255);


    let cat_id = V1ContentCategories::Message;
    let contents = V1ContentsContainer::new(
        cat_id.clone(),
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        Some(att)
    );

    let payload_att = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        len_att,
        // t_id,
        None,
        Some(sess_id)
    ).unwrap();
    let serialized = payload_att.serialize().unwrap();


    let deserialized = V1Payloads::deserialize(serialized).unwrap();
    assert_eq!(payload_att, deserialized);


    let payload_att = V1Payloads::new(
        contents.serialize().unwrap(),
        k_id,
        len_att,
        t_id,
        Some(sess_id)
    ).unwrap();

    let serialized = payload_att.serialize().unwrap();
    let deserialized = V1Payloads::deserialize(serialized).unwrap();
    assert_eq!(payload_att, deserialized);
}