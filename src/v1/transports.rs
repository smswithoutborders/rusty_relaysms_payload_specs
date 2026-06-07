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


#[uniffi::export(with_foreign)]
pub trait Transports: Debug + Send + Sync {
    fn get_max_payload_size(&self) -> u32;
}

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct SMS;

#[uniffi::export]
impl Transports for SMS {
    fn get_max_payload_size(&self) -> u32{ 104 }
}


