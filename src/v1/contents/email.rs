use std::any::Any;
use std::sync::Arc;
use crate::{bit_utils, AsAny};
use crate::v1::contents::{V1Contents, V1ContentError};

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct V1Emails {
    i_sub: bool,
    len_subject: u8,
    len_to: u8,
    to: Vec<u8>,
    body: Vec<u8>,
    subject: Option<Vec<u8>>,
}


#[uniffi::export]
impl V1Emails {
    pub fn get_i_sub(&self) -> bool { self.i_sub }
    pub fn get_len_subject(&self) -> u8 { self.len_subject }
    pub fn get_len_to(&self) -> u8 { self.len_to }
    pub fn get_to(&self) -> Vec<u8> { self.to.clone() }
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }
    pub fn get_subject(&self) -> Option<Vec<u8>> { self.subject.clone() }

    #[uniffi::constructor]
    pub fn new(
        to: Vec<u8>,
        body: Vec<u8>,
        subject: Option<Vec<u8>>,
    ) -> Result<Arc<Self>> {
        let len_subject = subject
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0);

        if len_subject > (2u8.pow(7) - 1) as usize {
            return Err(V1ContentError::SubjectLenTooLarge);
        }

        if to.len() > (2u8.pow(7) - 1) as usize {
            return Err(V1ContentError::ToTooLarge);
        }

        Ok(Arc::new(Self {
            i_sub: subject.is_some() && !subject.as_ref().unwrap().is_empty(),
            len_to: to.len() as u8,
            len_subject: len_subject as u8,
            to,
            body,
            subject,
        }))
    }

}


#[uniffi::export]
pub fn v1_deserialize_email_content(data: Vec<u8>) -> Result<Arc<V1Emails>> {
    let i_sub = bit_utils::is_bit_on(&data[0], 0);
    let len_subject = bit_utils::get_bits(&data[0], 1, 7);
    let len_to = bit_utils::get_bits(&data[1], 0, 6);

    let mut current_index: usize = 2;
    let to = data[2..current_index + len_to as usize].to_vec();
    current_index += len_to as usize;

    let subject = if i_sub {
        let slice = data[current_index..current_index + len_subject as usize].to_vec();
        current_index += len_subject as usize;
        Some(slice)
    } else { None };

    let body = data[current_index..].to_vec();

    Ok(Arc::new(V1Emails {
        i_sub,
        len_subject,
        len_to,
        to,
        body,
        subject
    }))
}

#[uniffi::export]
impl V1Contents for V1Emails {
    fn serialize(&self) -> std::result::Result<Vec<u8>, V1ContentError> {
        let mut bytes: Vec<u8> = Vec::new(); // TODO: put size here

        if self.i_sub {
            let ls = bit_utils::put_value(&1, 1, self.len_subject, 1);
            bytes.push(ls);
            bytes.push(self.len_to);
        } else {
            let byte = bit_utils::put_value(&0, 1, self.len_to, 1);
            bytes.push(byte);
        }

        bytes.extend(self.to.clone());
        if self.subject.is_some() {
            bytes.extend(self.subject.clone().unwrap());
        }
        bytes.extend(self.body.clone());
        Ok(bytes)
    }

    fn get_cat_id(&self) -> u8 { 0 }

    fn equals(&self, other: Arc<dyn V1Contents>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}



#[test]
fn test_email_init() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let email = V1Emails::new(
        to.to_vec(),
        body.to_vec(),
        Option::from(subject.to_vec()),
    ).unwrap();

    let serialized = email.serialize().unwrap();
    let deserialized = v1_deserialize_email_content(serialized).unwrap();

    assert_eq!(email, deserialized);
    // assert_eq!((2 + to.len() + body.len() + subject.len()), serialized.len());
    // let email1 = init_email(
    //     to,
    //     body,
    //     None,
    //     &from_id
    // ).unwrap();
    //
    // let serialized = email1.serialize().unwrap();
    // let deserialized = deserialize_email(serialized.as_slice()).unwrap();
    // assert_eq!(email1, deserialized);
}
