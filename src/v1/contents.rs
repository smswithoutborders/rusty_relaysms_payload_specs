use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::v1::contents::email::{v1_deserialize_email_content};

pub mod email;
pub mod contents_container;
mod message;
mod text;

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(uniffi::Enum)]
#[repr(u8)]
pub enum V1ContentCategories {
    Email = 0x0,
    Message = 0x1,
    Text = 0x2,
    Bridge = 0x3,
}

#[uniffi::export]
pub fn v1_content_category_from_u8(value: u8) -> Result<V1ContentCategories> {
    match value {
        0x0 => Ok(V1ContentCategories::Email),
        0x1 => Ok(V1ContentCategories::Message),
        0x2 => Ok(V1ContentCategories::Text),
        0x3 => Ok(V1ContentCategories::Bridge),
        _ => Err(V1ContentError::InvalidCategory),
    }
}


#[uniffi::export]
impl V1ContentCategories {
    pub fn raw_values(&self) -> u8 {
        match self {
            V1ContentCategories::Email => 0x0,
            V1ContentCategories::Message => 0x1,
            V1ContentCategories::Text => 0x2,
            V1ContentCategories::Bridge => 0x3,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum V1ContentError {
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

    #[error("Invalid category id")]
    InvalidCategory,
}
#[uniffi::export(with_foreign)]
pub trait V1Contents: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
    fn get_cat_id(&self) -> u8;
    fn equals(&self, other: Arc<dyn V1Contents>) -> bool;
}