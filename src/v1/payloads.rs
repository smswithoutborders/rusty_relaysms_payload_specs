use std::fmt::Debug;
use std::sync::Arc;
use crate::bit_utils::BitParsingError;

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
pub trait V1Payloads: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn V1Payloads>) -> bool;
}

#[uniffi::export(with_foreign)]
pub trait V1PayloadsWithoutAttachments: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn V1PayloadsWithoutAttachments>) -> bool;
}

#[uniffi::export(with_foreign)]
pub trait V1PayloadsWithAttachments: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn V1PayloadsWithAttachments>) -> bool;
}
