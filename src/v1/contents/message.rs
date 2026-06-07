use std::sync::Arc;
use crate::bit_utils;
use crate::v1::contents::email::{V1Emails};
use crate::v1::contents::{V1ContentError, V1Contents};

type Result<T> = std::result::Result<T, V1ContentError>;
#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1Messages {
    len_to: u8,
    to: Vec<u8>,
    body: Vec<u8>,
}

#[uniffi::export]
impl V1Messages {
    pub fn get_len_to(&self) -> u8 { self.len_to }
    pub fn get_to(&self) -> Vec<u8> { self.to.clone() }
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }

    #[uniffi::constructor]
    pub fn new(
        to: Vec<u8>,
        body: Vec<u8>,
    ) -> Result<Arc<Self>> {
        if to.len() > u8::MAX as usize {
            return Err(V1ContentError::ToTooLarge);
        }

        Ok(Arc::new(Self {
            len_to: to.len() as u8,
            to,
            body,
        }))
    }

}


// #[uniffi::export]
// pub fn v1_deserialize_message_content(data: Vec<u8>) -> Result<Arc<V1Messages>> {
//     let len_to = data[0];
//     let to = data[1..(1 + len_to as usize)].to_vec();
//     let body = data[(1 + len_to as usize)..].to_vec();
// 
//     Ok(Arc::new(V1Messages {
//         len_to,
//         to,
//         body,
//     }))
// }
// 
// #[uniffi::export]
// impl V1Contents for V1Messages {
//     fn serialize(&self) -> Result<Vec<u8>> {
//         let mut bytes: Vec<u8> = Vec::new(); // TODO: put size here
// 
//         bytes.push(self.len_to);
//         bytes.extend(self.to.clone());
//         bytes.extend(self.body.clone());
//         Ok(bytes)
//     }
// 
//     fn get_cat_id(&self) -> u8 { 1 }
// 
//     fn equals(&self, other: Arc<dyn V1Contents>) -> bool {
//         match (self.serialize(), other.serialize()) {
//             (Ok(a), Ok(b)) => a == b,
//             _ => false,
//         }
//     }
// }
// 
// 
// #[test]
// fn test_message_init() {
//     let to  = b"example@gmail.com"; //2
//     let body = b"Here is some heavy Lorem Ipsum shit"; //4
//     let message = V1Messages::new(
//         to.to_vec(),
//         body.to_vec(),
//     ).unwrap();
// 
//     let serialized = message.serialize().unwrap();
//     let deserialized = v1_deserialize_message_content(serialized).unwrap();
// 
//     assert_eq!(message, deserialized);
// }
