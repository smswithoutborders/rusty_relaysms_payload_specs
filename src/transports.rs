use std::fmt::Debug;
use std::sync::Arc;
use crate::contents::Contents;
use crate::contents::email::{deserialize_email, init_email, Emails};
use crate::transports::transport_att::{TransportAtt};
use crate::transports::transport_att_false::TransportAttFalse;

pub mod transport_att;
pub mod transport_att_false;

type Result<T> = std::result::Result<T, TransportsError>;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum TransportsError {
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
}

#[uniffi::export(with_foreign)]
pub trait Transports: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn equals(&self, other: Arc<dyn Transports>) -> bool;
}

#[uniffi::export]
pub fn init_transport(
    version: u8,
    seg_num: u8,
    sess_id: u8,
    e_id: u8,
    k_id: u8,
    cat_id: u8,
    len_att: u16,
    device_id: Option<Vec<u8>>,
    payload: Option<Vec<u8>>,
    payload_att_false: Option<Arc<dyn Contents>>,
) -> Result<Arc<dyn Transports>> {
    if payload.clone().map_or(false, |p| !p.is_empty()) {
        let t_att = TransportAtt::new(
            version,
            seg_num,
            sess_id,
            e_id,
            k_id,
            cat_id,
            len_att,
            device_id,
            payload
        ).map_err(|e| e)?;
        Ok(t_att)
    } else {
        let t_att = TransportAttFalse::new(
            version,
            e_id,
            k_id,
            cat_id,
            device_id,
            payload_att_false
        ).map_err(|e| e)?;
        Ok(t_att)
    }
}

#[uniffi::export]
pub fn deserialize_transport(data: &[u8]) -> Result<TransportAtt> {
    todo!()
}
