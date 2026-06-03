use std::sync::Arc;
use crate::v1::contents::{V1ContentError, V1ContentCategories, v1_content_category_from_u8, V1Contents};
use crate::v1::contents::email::V1Emails;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1ContentsContainer {
    cat_id: u8,
    body: String,
    to: Option<String>,
    subject: Option<String>,
}

#[uniffi::export]
impl V1ContentsContainer {
    #[uniffi::constructor]
    pub fn new(
        cat_id: V1ContentCategories,
        body: String,
        to: Option<String>,
        subject: Option<String>,
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
                            self.to.as_ref().unwrap().as_str(),
                            self.body.as_str(),
                            self.subject.clone(),
                        ) {
                            Ok(email) => email,
                            Err(e) => return Err(V1ContentError::from(e))
                        };
                        Ok(email)
                    }
                    V1ContentCategories::Message => {
                        todo!()
                    }
                    V1ContentCategories::Text => {
                        todo!()
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
