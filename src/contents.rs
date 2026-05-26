use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::contents::contents_container::ContentsContainer;
use crate::contents::email::Emails;

pub mod email;
pub mod contents_container;

#[derive(uniffi::Enum)]
#[repr(u8)]
pub enum ContentCategories {
    Email = 0x0,
    Message = 0x1,
    Text = 0x2,
    Bridge = 0x3,
}

#[uniffi::export]
pub fn content_category_from_u8(value: u8) -> ContentCategories {
    // TODO: put a guard here to make sure in range
    match value {
        0x0 => ContentCategories::Email,
        0x1 => ContentCategories::Message,
        0x2 => ContentCategories::Text,
        0x3 => ContentCategories::Bridge,
        _ => ContentCategories::Email,
    }
}


#[uniffi::export]
impl ContentCategories {
    pub fn raw_values(&self) -> u8 {
        match self {
            ContentCategories::Email => 0x0,
            ContentCategories::Message => 0x1,
            ContentCategories::Text => 0x2,
            ContentCategories::Bridge => 0x3,
        }
    }
}

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
    InvalidCategoryId,

    #[error("Missing to")]
    MissingTo,

    #[error("Body is empty")]
    EmptyBody,
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
