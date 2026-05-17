use crate::bit_utils;

type Result<T> = std::result::Result<T, ContentError>;

#[derive(Debug)]
pub enum ContentError {
    InconsistentSubjectIndicator,
    FromIdTooLarge,
    BitParsingError,
    InvalidUtf8
}


#[derive(PartialEq, Debug)]
pub struct Email {
    i_subject: bool,
    len_to: u8,
    len_body: u16,
    len_subject: u8,
    to: String,
    body: String,
    subject: Option<String>,
    from_id: u8,
}

impl Email {
    pub fn new(
        to: &str,
        body: &str,
        subject: Option<&str>,
        from_id: &u8,
    ) -> Result<Email> {
        let len_subject = subject.unwrap_or("").len() as u8;
        if from_id > &(2u8.pow(6) - 1) {
            return Err(ContentError::FromIdTooLarge);
        }

        Ok(Email {
            i_subject: len_subject > 0,
            len_to: to.len() as u8,
            len_body: body.len() as u16,
            len_subject,
            from_id: *from_id,
            to: to.to_string(),
            body: body.to_string(),
            subject: subject.map(|s| s.to_string()),
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut indicator: u8 = 0;

        if self.i_subject { indicator = bit_utils::turn_bit_on(&indicator, 0) }
        indicator = bit_utils::put_value(&indicator, 1, self.len_to, 1);
        bytes.push(indicator);

        let len_body_bytes = self.len_body.to_le_bytes();
        bytes.push(len_body_bytes[0]);

        let subject_bytes = self.subject.as_deref().unwrap_or("").as_bytes();
        if self.i_subject {
            let len_subject = subject_bytes.len() as u8;

            // byte 2: bits 0-1 = high len_body, bits 2-7 = low 6 bits of len_subject
            let mut byte2 = bit_utils::get_bits(&len_body_bytes[1], 0, 1);
            byte2 = bit_utils::put_value(&byte2, 2, len_subject, 2);
            bytes.push(byte2);

            // byte 3: bits 0-1 = high 2 bits of len_subject, bits 2-7 = from_id
            let mut byte3 = bit_utils::get_bits(&len_subject, 6, 7);
            byte3 = bit_utils::put_value(&byte3, 2, self.from_id, 2);
            bytes.push(byte3);
        } else {
            // byte 2: bits 0-1 = high len_body, bits 2-7 = from_id (no len_subject bytes needed)
            let mut byte2 = bit_utils::get_bits(&len_body_bytes[1], 0, 1);
            byte2 = bit_utils::put_value(&byte2, 2, self.from_id, 2);
            bytes.push(byte2);
            // no byte 3 — from_id fits in byte 2, saving one byte
        }

        bytes.extend_from_slice(self.to.as_bytes());
        bytes.extend_from_slice(self.body.as_bytes());
        if self.i_subject {
            bytes.extend_from_slice(subject_bytes);
        }
        Ok(bytes)
    }

    pub fn deserialize(data: &[u8]) -> Result<Email> {
        let indicator = data[0];
        let i_subject = bit_utils::is_bit_on(&indicator, 0);
        let len_to = bit_utils::get_bits(&indicator, 1, 7);
        let len_body_high = bit_utils::get_bits(&data[2], 0, 1);
        // let len_body = LittleEndian::read_u16(&[data[1], len_body_high]);
        let len_body = u16::from_le_bytes([data[1], len_body_high]);

        let mut current_index: usize = 2;
        let len_subject: u8 = if i_subject {
            match bit_utils::bit_wrap(
                &data[2],
                2,
                &data[3],
                1
            ) {
                Ok(n) =>  {
                    current_index += 1;
                    n
                },
                Err(e) => {
                    println!("{:?}", e);
                    return Err(ContentError::BitParsingError);
                }
            }
        } else { 0 };
        let from_id = bit_utils::get_bits(&data[current_index], 2, 7);
        current_index += 1;

        let to = data[current_index..current_index + len_to as usize].to_vec();
        current_index += len_to as usize;

        let body = data[current_index..current_index + len_body as usize].to_vec();
        current_index += len_body as usize;

        let subject = if i_subject {
            let slice = &data[current_index..current_index + len_subject as usize];
            // current_index += len_subject as usize;
            Some(slice)
        } else {
            None
        };

        Ok(Email {
            i_subject,
            len_to,
            len_subject,
            len_body,
            from_id,
            to: String::from_utf8(to).unwrap(),
            body: String::from_utf8(body).unwrap(),
            subject: subject.map(|b| String::from_utf8(b.to_vec()).unwrap()),
        })
    }
}