use std::sync::Arc;
use std::thread::current;
use crate::bit_utils;
use crate::v1::contents::email::{V1Emails};
use crate::v1::contents::{V1ContentError, V1Contents};

type Result<T> = std::result::Result<T, V1ContentError>;
#[derive(PartialEq, Debug, Clone, uniffi::Object)]
pub struct V1Messages {
    len_to: u8,
    to: Vec<u8>,
    body: Vec<u8>,
    attachment: Option<Vec<u8>>,
}

#[uniffi::export]
impl V1Messages {
    pub fn get_len_to(&self) -> u8 { self.len_to }
    pub fn get_to(&self) -> Vec<u8> { self.to.clone() }
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }
    pub fn get_attachment(&self) -> Option<Vec<u8>> { self.attachment.clone() }

    #[uniffi::constructor]
    pub fn new(
        to: Vec<u8>,
        body: Vec<u8>,
        attachment: Option<Vec<u8>>,
    ) -> Result<Arc<Self>> {
        if to.len() > u8::MAX as usize {
            return Err(V1ContentError::ToTooLarge);
        }

        Ok(Arc::new(Self {
            len_to: to.len() as u8,
            to,
            body,
            attachment,
        }))
    }

    #[uniffi::constructor]
    pub fn deserialize(data: &[u8], len_att: u16) -> Result<Arc<V1Messages>> {
        let len_to = data[0];
        let to = data[1..(1 + len_to as usize)].to_vec();
        let body = data[(1 + len_to as usize)..data.len() - len_att as usize].to_vec();

        let attachment = if len_att > 0 {
            Some(data[(data.len() - len_att as usize)..].to_vec())
        } else { None };

        V1Messages::new(to, body, attachment)
    }


}


#[uniffi::export]
impl V1Contents for V1Messages {
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new(); // TODO: put size here

        bytes.push(self.len_to);
        bytes.extend(self.to.clone());
        bytes.extend(self.body.clone());
        if self.attachment.is_some() {
            bytes.extend(self.attachment.clone().unwrap());
        }
        Ok(bytes)
    }
}

#[test]
fn test_message_init() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let message = V1Messages::new(
        to.to_vec(),
        body.to_vec(),
        None
    ).unwrap();

    let serialized = message.serialize().unwrap();
    let deserialized = V1Messages::deserialize(serialized.as_slice(), 0).unwrap();
    assert_eq!(message, deserialized);


    const LEN_ATT: u16 = 140 * 50;
    let attachment = rand::random::<[u8; LEN_ATT as usize]>().to_vec();
    let message = V1Messages::new(
        to.to_vec(),
        body.to_vec(),
        Some(attachment)
    ).unwrap();

    let serialized = message.serialize().unwrap();
    let deserialized = V1Messages::deserialize(serialized.as_slice(), LEN_ATT).unwrap();
    assert_eq!(message, deserialized);
}
