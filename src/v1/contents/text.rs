use std::sync::Arc;
use crate::v1::contents::{V1ContentError, V1Contents};
use crate::v1::contents::message::{v1_deserialize_message_content, V1Messages};

type Result<T> = std::result::Result<T, V1ContentError>;
#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1Text {
    body: Vec<u8>,
}

#[uniffi::export]
impl V1Text {
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }

    #[uniffi::constructor]
    pub fn new(
        body: Vec<u8>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            body,
        }))
    }

}


#[uniffi::export]
pub fn v1_deserialize_text_content(data: Vec<u8>) -> Result<Arc<V1Text>> {
    Ok(Arc::new(V1Text {
        body: data,
    }))
}

#[uniffi::export]
impl V1Contents for V1Text {
    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(self.body.clone())
    }

    fn get_cat_id(&self) -> u8 { 2 }

    fn equals(&self, other: Arc<dyn V1Contents>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}


#[test]
fn test_text_init() {
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let text = V1Text::new(
        body.to_vec(),
    ).unwrap();

    let serialized = text.serialize().unwrap();
    let deserialized = v1_deserialize_text_content(serialized).unwrap();

    assert_eq!(text, deserialized);
}
