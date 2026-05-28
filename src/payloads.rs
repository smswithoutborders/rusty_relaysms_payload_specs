use std::fmt::Debug;
use std::sync::Arc;
use crate::bit_utils;
use crate::bit_utils::BitParsingError;
use crate::contents::{ContentError, Contents};
use crate::contents::email::{Emails};
use crate::payloads::payload_with_attachments::{PayloadWithAttachments};
use crate::payloads::payload_without_attachment::PayloadWithoutAttachment;

pub mod payload_with_attachments;
pub mod payload_without_attachment;
type Result<T> = std::result::Result<T, PayloadsError>;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PayloadsError {
    #[error("Version too large")]
    VersionTooLarge,

    #[error("Encryption ID too large")]
    EncryptionIdTooLarge,

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

    #[error("Content deserialization error")]
    ContentDeserializationError,
    
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
}

#[uniffi::export(with_foreign)]
pub trait Payloads: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn Payloads>) -> bool;
}

#[uniffi::export(with_foreign)]
pub trait PayloadsWithoutAttachments: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn PayloadsWithoutAttachments>) -> bool;
}

#[uniffi::export(with_foreign)]
pub trait PayloadsWithAttachments: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn PayloadsWithAttachments>) -> bool;
}
