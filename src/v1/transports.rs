use std::fmt::Debug;
use std::io::BufRead;
use std::sync::Arc;
use crate::utils;
use crate::v1::get_version;
use crate::v1::payloads::payload_with_attachments::{V1PayloadWithAttachmentsHeader, V1PayloadWithAttachmentsNoHeader};
use crate::v1::payloads::{V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::PayloadTooLarge;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum V1TransportError {
    #[error("Failed to split")]
    FailedToSplit,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Transports {
    Sms
}

#[uniffi::export]
impl Transports {
    pub fn get_max_payload_size(&self) -> u32{
        match self {
            Transports::Sms => 104,
        }
    }
}


