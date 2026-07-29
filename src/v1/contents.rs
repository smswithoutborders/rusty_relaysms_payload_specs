use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::AsAny;
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;
use crate::v1::payloads::{V1Payloads, V1PayloadsError};

pub mod email;
pub mod message;
pub mod text;

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(uniffi::Enum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[repr(u8)]
pub enum V1ContentCategories {
    Email = 0x0,
    Message = 0x1,
    Text = 0x2,
}

#[uniffi::export]
pub fn v1_content_category_from_u8(value: u8) -> Result<V1ContentCategories> {
    match value {
        0x0 => Ok(V1ContentCategories::Email),
        0x1 => Ok(V1ContentCategories::Message),
        0x2 => Ok(V1ContentCategories::Text),
        _ => Err(V1ContentError::InvalidCategory),
    }
}


#[derive(Debug, thiserror::Error, uniffi::Error, PartialEq)]
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

    #[error("Error serializing for storage")]
    ErrorSerializingForStorage,

    #[error("Error deserializing for storage")]
    ErrorDeserializingForStorage,
}

#[uniffi::export(with_foreign)]
pub trait V1Contents: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>>;
}

#[derive(PartialEq, Debug, uniffi::Object, Serialize, Deserialize)]
pub struct V1ContentsContainer {
    cat_id: V1ContentCategories,
    body: Vec<u8>,
    to: Option<Vec<u8>>,
    subject: Option<Vec<u8>>,
    attachment: Option<Vec<u8>>,
}

#[uniffi::export]
impl V1ContentsContainer {
    pub fn get_cat_id(&self) -> V1ContentCategories { self.cat_id.clone() }
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }
    pub fn get_to(&self) -> Option<Vec<u8>> { self.to.clone() }
    pub fn get_subject(&self) -> Option<Vec<u8>> { self.subject.clone() }
    pub fn get_attachment(&self) -> Option<Vec<u8>> { self.attachment.clone() }

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
            attachment
        }
    }

    pub fn serialize_for_storage(&self) -> Result<Vec<u8>> {
        match serde_json::to_vec(&self) {
            Ok(v) => Ok(v),
            Err(e) => Err(V1ContentError::ErrorSerializingForStorage)
        }
    }

    #[uniffi::constructor]
    pub fn deserialize_from_storage(bytes: Vec<u8>) -> Result<Self> {
        match serde_json::from_slice::<V1ContentsContainer>(&bytes) {
            Ok(v) => Ok(v),
            Err(e) => Err(V1ContentError::ErrorDeserializingForStorage)
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
                    attachment: email.get_attachment(),
                })
            }
            V1ContentCategories::Message => {
                let message = match V1Messages::deserialize(data, len_att) {
                    Ok(message) => message,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentsContainer {
                    cat_id,
                    body: message.get_body(),
                    to: Some(message.get_to()),
                    subject: None,
                    attachment: message.get_attachment(),
                })
            }
            V1ContentCategories::Text => {
                let text = match V1Text::deserialize(data, len_att) {
                    Ok(text) => text,
                    Err(e) => return Err(e)
                };
                Ok(V1ContentsContainer {
                    cat_id,
                    body: text.get_body(),
                    to: None,
                    subject: None,
                    attachment: text.get_attachment(),
                })
            }
        }
    }

    pub fn serialize( &self, ) -> Result<Vec<u8>> {
        match self.cat_id {
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
        }
    }
}

#[test]
fn test_serialization_for_storage() {
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;

    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let sess_id: u8 = 15;
    let k_id: u8 = 13;
    let t_id: Option<u32> = Option::from(255);


    let cat_id = V1ContentCategories::Message;
    let contents = V1ContentsContainer::new(
        cat_id.clone(),
        body.to_vec(),
        Some(to.to_vec()),
        Some(subject.to_vec()),
        Some(att)
    );

    let serialized = contents.serialize_for_storage().unwrap();
    let deserialized = V1ContentsContainer::deserialize_from_storage(
        serialized).unwrap();
    assert_eq!(deserialized, contents);
}