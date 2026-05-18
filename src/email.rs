use crate::{bit_utils, ContentError, Contents};

type Result<T> = std::result::Result<T, ContentError>;

#[derive(PartialEq, Debug)]
pub struct Emails {
    from_id: u8,
    len_subject: u8,
    len_to: u8,
    to: String,
    body: String,
    subject: Option<String>,
}

impl Emails {
    pub fn new(
        to: &str,
        body: &str,
        subject: Option<&str>,
        from_id: &u8,
    ) -> Result<Emails> {
        if from_id > &(2u8.pow(3) - 1) {
            return Err(ContentError::FromIdTooLarge);
        }

        let len_subject = subject.unwrap_or("").len() as u8;
        if len_subject > (2u8.pow(5) - 1) {
            return Err(ContentError::SubjectLenTooLarge);
        }

        if to.len() > u8::MAX as usize {
            return Err(ContentError::ToTooLarge);
        }

        Ok(Emails {
            len_to: to.len() as u8,
            len_subject,
            from_id: *from_id,
            to: to.to_string(),
            body: body.to_string(),
            subject: subject.map(|s| s.to_string()),
        })
    }
}

impl Contents for Emails {
    fn serialize(&self) -> std::result::Result<Vec<u8>, ContentError> {
        let mut bytes: Vec<u8> = Vec::new(); // TODO: put size here

        let index_0 = bit_utils::put_value(
            &self.from_id, 3, self.len_subject, 2);
        bytes.push(index_0);
        bytes.push(self.len_to);

        bytes.extend_from_slice(self.to.as_bytes());
        let subject_bytes = self.subject.as_deref().unwrap_or("").as_bytes();
        if subject_bytes.len() > 0 {
            bytes.extend_from_slice(subject_bytes);
        }
        bytes.extend_from_slice(self.body.as_bytes());
        Ok(bytes)
    }

    fn deserialize(data: &[u8]) -> std::result::Result<Emails, ContentError> {
        let from_id = bit_utils::get_bits(&data[0], 0, 2);
        let len_subject = bit_utils::get_bits(&data[0], 3, 7);
        let len_to = data[1];

        let mut current_index: usize = 2;
        let to = data[2..current_index + len_to as usize].to_vec();
        current_index += len_to as usize;

        let subject = if len_subject > 0 {
            let slice = data[current_index..current_index + len_subject as usize].to_vec();
            current_index += len_subject as usize;
            match String::from_utf8(slice) {
                Ok(s) => Some(s),
                Err(_) => None
            }
        } else { None };

        let body = data[current_index..].to_vec();

        Ok(Emails {
            from_id,
            len_subject,
            len_to,
            to: String::from_utf8(to).unwrap(),
            body: String::from_utf8(body).unwrap(),
            subject
        })
    }
}