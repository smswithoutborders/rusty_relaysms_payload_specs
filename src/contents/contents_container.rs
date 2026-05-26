use std::option;
use std::sync::Arc;
use crate::contents::{content_category_from_u8, ContentCategories, ContentError, Contents};
use crate::contents::email::Emails;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct ContentsContainer {
    cat_id: u8,
    body: String,
    from_id: u8,
    to: Option<String>,
    subject: Option<String>,
}

#[uniffi::export]
impl ContentsContainer {
    #[uniffi::constructor]
    pub fn new(
        cat_id: ContentCategories,
        body: String,
        from_id: u8,
        to: Option<String>,
        subject: Option<String>,
    ) -> Self {
        Self {
            cat_id: cat_id.raw_values(),
            body,
            from_id,
            to,
            subject
        }
    }

    pub fn instance(&self) -> Result<Arc<dyn Contents>, ContentError> {
        match content_category_from_u8(self.cat_id) {
            ContentCategories::Email => {
                if !self.to.is_some() {
                    return Err(ContentError::MissingTo)
                }
                if self.body.is_empty() {
                    return Err(ContentError::EmptyBody)
                }
                let email = match Emails::new(
                    self.to.as_ref().unwrap().as_str(),
                    self.body.as_str(),
                    self.subject.clone(),
                    &self.from_id
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
