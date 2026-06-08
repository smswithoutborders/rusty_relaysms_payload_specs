use std::any::Any;
use std::sync::Arc;
use std::thread::current;
use crate::{bit_utils, AsAny};
use crate::v1::contents::{V1ContentError, V1Contents};

type Result<T> = std::result::Result<T, V1ContentError>;

#[derive(PartialEq, Debug, Clone, uniffi::Object)]
pub struct V1Emails {
    i_sub: bool,
    len_subject: u8,
    len_to: u8,
    to: Vec<u8>,
    body: Vec<u8>,
    subject: Option<Vec<u8>>,
    attachment: Option<Vec<u8>>,
}

impl V1Emails {
    pub fn get_i_sub(&self) -> bool { self.i_sub }
    pub fn get_len_subject(&self) -> u8 { self.len_subject }
    pub fn get_len_to(&self) -> u8 { self.len_to }
    pub fn get_to(&self) -> Vec<u8> { self.to.clone() }
    pub fn get_body(&self) -> Vec<u8> { self.body.clone() }
    pub fn get_subject(&self) -> Option<Vec<u8>> { self.subject.clone() }
    pub fn get_attachment(&self) -> Option<Vec<u8>> { self.attachment.clone() }

    pub fn new(
        to: Vec<u8>,
        body: Vec<u8>,
        subject: Option<Vec<u8>>,
        attachment: Option<Vec<u8>>,
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
            attachment,
        }))
    }


    pub fn deserialize(data: &[u8], len_att: u16) -> Result<Arc<V1Emails>> {
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

        let body = data[current_index..(data.len() - len_att as usize)].to_vec();
        let attachment = if len_att > 0 {
            Some(data[(data.len() - len_att as usize)..].to_vec())
        } else { None };

        V1Emails::new(to, body, subject, attachment)
    }

}

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
        if(self.attachment.is_some()) {
            bytes.extend(self.attachment.clone().unwrap());
        }
        Ok(bytes)
    }
}

#[test]
fn test_emails() {
    let to  = b"example@gmail.com"; //2
    let body = b"Here is some heavy Lorem Ipsum shit"; //4
    let subject = b"More things"; //7
    let email = V1Emails::new(
        to.to_vec(),
        body.to_vec(),
        Option::from(subject.to_vec()),
        None
    ).unwrap();

    let serialized = email.serialize().unwrap();
    let deserialized = V1Emails::deserialize(serialized.as_slice(), 0).unwrap();
    assert_eq!(email, deserialized);

    const LEN_ATT: u16 = 140 * 50;
    let attachment = rand::random::<[u8; LEN_ATT as usize]>().to_vec();

    let email = V1Emails::new(
        to.to_vec(),
        body.to_vec(),
        Option::from(subject.to_vec()),
        Some(attachment)
    ).unwrap();

    let serialized = email.serialize().unwrap();
    let deserialized = V1Emails::deserialize(serialized.as_slice(), LEN_ATT).unwrap();
    assert_eq!(email, deserialized);
}
