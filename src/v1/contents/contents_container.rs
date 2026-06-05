use std::sync::Arc;
use crate::v1::contents::{V1ContentError, V1ContentCategories, v1_content_category_from_u8, V1Contents};
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::message::V1Messages;
use crate::v1::contents::text::V1Text;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1ContentsContainer {
    cat_id: u8,
    body: Vec<u8>,
    to: Option<Vec<u8>>,
    subject: Option<Vec<u8>>,
}

#[uniffi::export]
impl V1ContentsContainer {
    #[uniffi::constructor]
    pub fn new(
        cat_id: V1ContentCategories,
        body: Vec<u8>,
        to: Option<Vec<u8>>,
        subject: Option<Vec<u8>>,
    ) -> Self {
        Self {
            cat_id: cat_id.raw_values(),
            body,
            to,
            subject
        }
    }

    pub fn instance(&self) -> Result<Arc<dyn V1Contents>, V1ContentError> {
        match v1_content_category_from_u8(self.cat_id) {
            Ok(contents) => {
                match contents {
                    V1ContentCategories::Email | V1ContentCategories::Bridge => {
                        if !self.to.is_some() {
                            return Err(V1ContentError::MissingTo)
                        }
                        if self.body.is_empty() {
                            return Err(V1ContentError::EmptyBody)
                        }
                        let email = match V1Emails::new(
                            self.to.clone().unwrap(),
                            self.body.clone(),
                            self.subject.clone(),
                        ) {
                            Ok(email) => email,
                            Err(e) => return Err(V1ContentError::from(e))
                        };
                        Ok(email)
                    }
                    V1ContentCategories::Message => {
                        let message = match V1Messages::new(
                            self.to.clone().unwrap(),
                            self.body.clone(),
                        ) {
                            Ok(message) => message,
                            Err(e) => return Err(V1ContentError::from(e))
                        };
                        Ok(message)
                    }
                    V1ContentCategories::Text => {
                        let text = match V1Text::new(
                            self.body.clone(),
                        ) {
                            Ok(text) => text,
                            Err(e) => return Err(V1ContentError::from(e))
                        };
                        Ok(text)
                    }
                }
            },
            Err(e) => Err(V1ContentError::from(e))
        }
    }
}


impl V1Contents for V1ContentsContainer {
    fn serialize(&self) -> Result<Vec<u8>, V1ContentError> {
        todo!()
    }

    fn get_cat_id(&self) -> u8 {
        todo!()
    }

    fn equals(&self, other: Arc<dyn V1Contents>) -> bool {
        todo!()
    }
}
