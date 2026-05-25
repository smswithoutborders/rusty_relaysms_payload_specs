use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::contents::email::Emails;

pub mod email;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ContentError {
    #[error("Inconsistent subject indicator")]
    InconsistentSubjectIndicator,
    #[error("From ID too large")]
    FromIdTooLarge,
    #[error("Subject length too large")]
    SubjectLenTooLarge,
    #[error("To too large")]
    ToTooLarge,
    #[error("Bit parsing error")]
    BitParsingError,
    #[error("Invalid utf-8")]
    InvalidUtf8,
    #[error("Invalid category ID")]
    InvalidCategoryId
}
#[uniffi::export(with_foreign)]
pub trait Contents: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>, ContentError>;
    fn get_cat_id(&self) -> u8;
    fn equals(&self, other: Arc<dyn Contents>) -> bool;
}

pub fn deserialize_for_content(cat_id: u8, payload: Vec<u8>) -> Result<Arc<dyn Contents>, ContentError> {
    match cat_id {
        0 => {
            // email
            Ok(Emails::instance()?.deserialize(payload)?)
        }
        1 => {
            todo!()
        }
        2 => {
            todo!()
        }
        3 => {
            todo!()
        }
        4 => {
            todo!()
        }
        _ => Err(ContentError::InvalidCategoryId)
    }
}
