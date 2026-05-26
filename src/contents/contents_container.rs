use std::option;
use std::sync::Arc;
use crate::contents::{ContentCategories, ContentError, Contents};
use crate::contents::email::Emails;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct ContentsContainer;

#[uniffi::export]
impl ContentsContainer {
    #[uniffi::constructor]
    pub fn new(
        cat_id: ContentCategories,
        body: String,
        from_id: u8,
        to: Option<String>,
        subject: Option<String>,
    ) -> Result<Arc<dyn Contents>, ContentError> {
        match cat_id {
            ContentCategories::Email => {
                if !to.is_some() {
                    return Err(ContentError::MissingTo)
                }
                if body.is_empty() {
                    return Err(ContentError::EmptyBody)
                }
                let email = match Emails::new(
                    to.unwrap().as_str(),
                    body.as_str(),
                    subject,
                    &from_id
                ) {
                    Ok(email) => email,
                    Err(e) => return Err(ContentError::from(e))
                };
                Ok(email)
            }
            ContentCategories::Message => {
                todo!()
            }
            ContentCategories::Text => {
                todo!()
            }
            ContentCategories::Bridge => {
                todo!()
            }
        }
    }
}


impl Contents for ContentsContainer {
    fn serialize(&self) -> Result<Vec<u8>, ContentError> {
        todo!()
    }

    fn get_cat_id(&self) -> u8 {
        todo!()
    }

    fn equals(&self, other: Arc<dyn Contents>) -> bool {
        todo!()
    }
}
