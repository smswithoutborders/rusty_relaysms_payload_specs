mod sms;

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
trait Transport: Debug + Send + Sync {
    fn split(&self) -> Result<Vec<Vec<u8>>, V1TransportError>;
}

