use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::v1::contents::email::V1Emails;

pub mod email;
mod message;
mod text;

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(uniffi::Enum, Debug, PartialEq)]
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
trait V1Contents: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
}

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1ContentsContainer {
    cat_id: V1ContentCategories,
    body: Vec<u8>,
    to: Option<Vec<u8>>,
    subject: Option<Vec<u8>>,
    attachment: Option<Vec<u8>>,
}

#[uniffi::export]
impl V1ContentsContainer {
    #[uniffi::constructor]
    pub fn new(
        cat_id: V1ContentCategories,
        body: Vec<u8>,
        to: Option<Vec<u8>>,
        subject: Option<Vec<u8>>,
        attachment: Option<Vec<u8>>,
    ) -> Self {
        Self {
            cat_id,
            body,
            to,
            subject,
            attachment,
        }
    }

    #[uniffi::constructor]
    pub fn deserialize(
        data: &[u8],
        cat_id: V1ContentCategories,
        len_att: u16
    ) -> Result<V1ContentsContainer> {
        match cat_id {
            V1ContentCategories::Email => {
                let email = match V1Emails::deserialize(data, len_att) {
                    Ok(email) => email,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentsContainer {
                    cat_id,
                    body: email.get_body(),
                    to: Some(email.get_to()),
                    subject: email.get_subject(),
                    attachment: email.get_attachment()
                })
            }
            V1ContentCategories::Message => {
                todo!()
            }
            V1ContentCategories::Text => {
                todo!()
            }
            V1ContentCategories::Bridge => {
                todo!()
            }
        }
    }

    pub fn serialize(
        &self,
        cat_id: V1ContentCategories,
    ) -> Result<Vec<u8>> {
        match cat_id {
            V1ContentCategories::Email => {
                let email = match V1Emails::new(
                    self.to.clone().unwrap(),
                    self.body.clone(),
                    self.subject.clone(),
                    self.attachment.clone(),
                ) {
                    Ok(email) => email,
                    Err(e) => return Err(e)
                };
                email.serialize()
            }
            V1ContentCategories::Message => {
                todo!()
            }
            V1ContentCategories::Text => {
                todo!()
            }
            V1ContentCategories::Bridge => {
                todo!()
            }
        }
    }
}
