use crate::bit_utils;

type Result<T> = std::result::Result<T, ContentError>;

pub enum ContentError {
    BitParsingError,
}

pub struct Email {
    i_subject: bool,
    i_from: bool,
    len_to: u8,
    len_body: u16,
    len_subject: u8,
    len_from: u8,

    to: String,
    body: String,
    subject: Option<String>,
    from: Option<String>,
}

impl Email {
    pub fn new(
        to: &str,
        body: &str,
        subject: &str,
        from: &str,
    ) -> Email {
        let len_subject = subject.len() as u8;
        let len_from = from.len() as u8;

        Email {
            i_subject: len_subject > 0,
            i_from: len_from > 0,
            len_to: to.len() as u8,
            len_body: body.len() as u16,
            len_subject,
            len_from,
            to: to.to_string(),
            body: body.to_string(),
            subject: Option::from(subject.to_string()),
            from: Option::from(from.to_string()),
        }
    }

    pub fn serialize(&self) -> *mut [u8] {
        let mut indicator: u8 = 0;

        if self.i_subject { indicator = bit_utils::turn_bit_on( &indicator, 0 ) }
        if self.i_from { indicator = bit_utils::turn_bit_on( &indicator, 1 ) }

        let low_len_to = bit_utils::get_bits(&self.len_to, 0, 5);
        indicator = bit_utils::put_value(&indicator, 2, low_len_to, 0);
        let mut byte1 = bit_utils::get_bits(&self.len_to, 5, 7);

        let len_body_bytes = self.len_body.to_le_bytes();
        let low_len_body = len_body_bytes[0];
        let high_len_body = len_body_bytes[1];
        byte1 = bit_utils::put_value(&byte1, 2, low_len_body, 2);

        todo!()
    }

    pub fn deserialize(data: &[u8]) -> Result<Email> {
        let indicator = data[0];
        let i_subject = bit_utils::is_bit_on(&indicator, 0);
        let i_from = bit_utils::is_bit_on(&indicator, 1);

        let len_to = match bit_utils::bit_wrap(
            &data[0],
            2,
            &data[1],
            0
        ) {
            Ok(n) =>  n,
            Err(e) => {
                println!("{:?}", e);
                return Err(ContentError::BitParsingError);
            }
        };

        let len_body = match bit_utils::bit_wrap(
            &data[1],
            1,
            &data[2],
            3
        ) {
            Ok(n) =>  n,
            Err(e) => {
                println!("{:?}", e);
                return Err(ContentError::BitParsingError);
            }
        };

        let len_subject: u8 = if i_subject {
            match bit_utils::bit_wrap(
                &data[2],
                3,
                &data[3],
                2
            ) {
                Ok(n) =>  n,
                Err(e) => {
                    println!("{:?}", e);
                    return Err(ContentError::BitParsingError);
                }
            }
        } else { 0 };

        let mut current_index: usize = 2;
        let len_from = if i_from {
            if i_subject {
                current_index += 1;
                bit_utils::get_bits(&data[3], 3, 7)
            } else {
                bit_utils::get_bits(&data[2], 3, 7)
            }
        } else { 0 };

        let to = data[current_index..current_index + len_to as usize].to_vec();
        current_index += len_to as usize;

        let body = data[current_index..current_index + len_body as usize].to_vec();
        current_index += len_body as usize;

        let mut next_slice = |condition: bool, length: u8| -> Option<&[u8]> {
            if condition {
                let len = length as usize;
                let slice = &data[current_index..current_index + len];
                current_index += len;
                Some(slice)
            } else {
                None
            }
        };

        let subject = next_slice(i_subject, len_subject);
        let from = next_slice(i_from, len_from);

        Ok(Email {
            i_subject,
            i_from,
            len_from,
            len_to,
            len_subject,
            len_body,

            to: String::from_utf8(to).unwrap(),
            body: String::from_utf8(body).unwrap(),
            subject: subject.map(|b| String::from_utf8(b.to_vec()).unwrap()),
            // from: if let Some(from) = from {
            //     Some(String::from_utf8(from.to_vec()).unwrap())
            // } else { None },
            from: from.map(|b| String::from_utf8(b.to_vec()).unwrap()),
        })
    }
}