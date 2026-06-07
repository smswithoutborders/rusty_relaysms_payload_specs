use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;

pub mod email;
pub mod message;
pub mod text;

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(uniffi::Enum, Debug, PartialEq)]
#[repr(u8)]
pub enum V1ContentCategories {
    Email = 0x0,
    Message = 0x1,
    Text = 0x2,
    Bridge = 0x3,

    EMAIL { value: Arc<V1Emails> },
    MESSAGE { value: Arc<V1Messages> },
    TEXT { value: Arc<V1Text> },
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
    ) -> Result<V1ContentCategories> {
        match cat_id {
            V1ContentCategories::Email | V1ContentCategories::Bridge => {
                let email = match V1Emails::deserialize(data, len_att) {
                    Ok(email) => email,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentCategories::EMAIL { value: email })
            }
            V1ContentCategories::Message => {
                let message = match V1Messages::deserialize(data, len_att) {
                    Ok(email) => email,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentCategories::MESSAGE { value: message })
            }
            V1ContentCategories::Text => {
                let text = match V1Text::deserialize(data, len_att) {
                    Ok(email) => email,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentCategories::TEXT { value: text })
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn serialize(
        &self,
        cat_id: V1ContentCategories,
    ) -> Result<Vec<u8>> {
        match cat_id {
            V1ContentCategories::Email | V1ContentCategories::Bridge => {
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
                let message = match V1Messages::new(
                    self.to.clone().unwrap(),
                    self.body.clone(),
                    self.attachment.clone(),
                ) {
                    Ok(message) => message,
                    Err(e) => return Err(e)
                };
                message.serialize()
            }
            V1ContentCategories::Text => {
                let text = match V1Text::new(
                    self.body.clone(),
                    self.attachment.clone(),
                ) {
                    Ok(text) => text,
                    Err(e) => return Err(e)
                };
                text.serialize()
            }
            _ => {
                todo!()
            }
        }
    }
}
