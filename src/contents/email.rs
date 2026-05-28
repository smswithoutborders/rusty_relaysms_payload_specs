use std::any::Any;
use std::sync::Arc;
use crate::{bit_utils, AsAny};
use crate::contents::{ContentError, Contents};

type Result<T> = std::result::Result<T, ContentError>;

#[derive(PartialEq, Debug, uniffi::Object)]
pub struct Emails {
    i_sub: bool,
    len_subject: u8,
    len_to: u8,
    to: String,
    body: String,
    subject: Option<String>,
}


#[uniffi::export]
impl Emails {
    pub fn get_i_sub(&self) -> bool { self.i_sub }
    pub fn get_len_subject(&self) -> u8 { self.len_subject }
    pub fn get_len_to(&self) -> u8 { self.len_to }
    pub fn get_to(&self) -> String { self.to.clone() }
    pub fn get_body(&self) -> String { self.body.clone() }
    pub fn get_subject(&self) -> Option<String> { self.subject.clone() }

    #[uniffi::constructor]
    pub fn new(
        to: &str,
        body: &str,
        subject: Option<String>,
        from_id: &u8,
    ) -> Result<Arc<Self>> {
        if from_id > &(2u8.pow(3) - 1) {
            return Err(ContentError::FromIdTooLarge);
        }

        let len_subject = subject
            .as_ref()
            .map(|s| s.chars().count())
            .unwrap_or(0);

        if len_subject > (2u8.pow(7) - 1) as usize {
            return Err(ContentError::SubjectLenTooLarge);
        }

        if to.len() > (2u8.pow(7) - 1) as usize {
            return Err(ContentError::ToTooLarge);
        }

        Ok(Arc::new(Self {
            i_sub: subject.is_some() && !subject.as_ref().unwrap().is_empty(),
            len_to: to.len() as u8,
            len_subject: len_subject as u8,
            to: to.to_string(),
            body: body.to_string(),
            subject: subject.map(|s| s.to_string()),
        }))
    }

}

pub fn deserialize_email_content(data: Vec<u8>) -> std::result::Result<Arc<Emails>, ContentError> {
    let i_sub = bit_utils::is_bit_on(&data[0], 0);
    let len_subject = bit_utils::get_bits(&data[0], 1, 7);
    let len_to = bit_utils::get_bits(&data[1], 0, 6);

    let mut current_index: usize = 2;
    let to = data[2..current_index + len_to as usize].to_vec();
    current_index += len_to as usize;

    let subject = if i_sub {
        let slice = data[current_index..current_index + len_subject as usize].to_vec();
        current_index += len_subject as usize;
        match String::from_utf8(slice) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    } else { None };

    let body = data[current_index..].to_vec();

    Ok(Arc::new(Emails {
        i_sub,
        len_subject,
        len_to,
        to: String::from_utf8(to).unwrap(),
        body: String::from_utf8(body).unwrap(),
        subject
    }))
}

#[uniffi::export]
impl Contents for Emails {
    fn serialize(&self) -> std::result::Result<Vec<u8>, ContentError> {
        let mut bytes: Vec<u8> = Vec::new(); // TODO: put size here

        let mut byte: u8 = if self.i_sub { 1 } else { 0 };
        if self.i_sub {
            byte = bit_utils::put_value(&byte, 1, self.len_subject, 1);
            bytes.push(byte);
            bytes.push(self.len_to);
        } else {
            byte = bit_utils::put_value(&byte, 1, self.len_to, 1);
            bytes.push(byte);
        }

        bytes.extend_from_slice(self.to.as_bytes());
        let subject_bytes = self.subject.as_deref().unwrap_or("").as_bytes();
        if subject_bytes.len() > 0 {
            bytes.extend_from_slice(subject_bytes);
        }
        bytes.extend_from_slice(self.body.as_bytes());
        Ok(bytes)
    }

    fn get_cat_id(&self) -> u8 { 0 }

    fn equals(&self, other: Arc<dyn Contents>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}



#[test]
fn test_email_init() {
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let from_id: u8 = 7; // 1
    let email = Emails::new(
        to,
        body,
        Option::from(subject.to_string()),
        &from_id
    ).unwrap();

    let serialized = email.serialize().unwrap();
    let deserialized = deserialize_email_content(serialized).unwrap();

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
