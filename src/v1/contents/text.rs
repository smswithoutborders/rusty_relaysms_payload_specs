use std::sync::Arc;
use uniffi::constructor;
use crate::v1::contents::{V1ContentError, V1Contents};
use crate::v1::contents::message::{V1Messages};

type Result<T> = std::result::Result<T, V1ContentError>;
#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1Text {
    body: Vec<u8>,
    attachment: Option<Vec<u8>>,
}

#[uniffi::export]
impl V1Text {
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }

    pub fn get_attachment(&self) -> Option<Vec<u8>> { self.attachment.clone() }

    #[uniffi::constructor]
    pub fn new(
        body: Vec<u8>,
        attachment: Option<Vec<u8>>,
    ) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            body,
            attachment
        }))
    }

    #[uniffi::constructor]
    pub fn deserialize(data: Vec<u8>, len_att: u16) -> Result<Arc<V1Text>> {
        let body = data[..(data.len() - len_att as usize)].to_vec();
        let attachment = if len_att > 0 {
            Some(data[(data.len() - len_att as usize)..].to_vec())
        } else { None };

        V1Text::new(body, attachment)
    }

}


#[uniffi::export]
impl V1Contents for V1Text {
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = self.body.clone();
        if self.attachment.is_some() {
            content.extend(self.attachment.clone().unwrap());
        }
        Ok(content)
    }
}

#[test]
fn test_text_init() {
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let text = V1Text::new(
        body.to_vec(),
        None
    ).unwrap();

    let serialized = text.serialize().unwrap();
    let deserialized = V1Text::deserialize(serialized, 0).unwrap();
    assert_eq!(text, deserialized);

    const LEN_ATT: u16 = 140 * 50;
    let attachment = rand::random::<[u8; LEN_ATT as usize]>().to_vec();
    let text = V1Text::new(
        body.to_vec(),
        Some(attachment)
    ).unwrap();

    let serialized = text.serialize().unwrap();
    let deserialized = V1Text::deserialize(serialized, LEN_ATT).unwrap();

    assert_eq!(text, deserialized);
}
