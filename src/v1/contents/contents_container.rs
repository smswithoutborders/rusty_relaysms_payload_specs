use std::sync::Arc;
use crate::v1::contents::{V1ContentError, V1ContentCategories, v1_content_category_from_u8, V1Contents};
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;

type Result<T> = std::result::Result<T, V1ContentError>;

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
                Ok( V1ContentsContainer {
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
}
