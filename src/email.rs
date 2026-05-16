use byteorder::{ByteOrder, LittleEndian};
use crate::bit_utils;

type Result<T> = std::result::Result<T, ContentError>;

pub enum ContentError {
    BitParsingError,
}

pub struct Email {
    i_subject: bool,
    len_to: u8,
    len_body: u16,
    len_subject: u8,
    to: String,
    body: String,
    subject: Option<String>,
    from_id: Option<u8>,
}

impl Email {
    pub fn new(
        to: &str,
        body: &str,
        subject: &str,
        from_id: &u8,
    ) -> Email {
        let len_subject = subject.len() as u8;
        Email {
            i_subject: len_subject > 0,
            len_to: to.len() as u8,
            len_body: body.len() as u16,
            len_subject,
            from_id: Option::from(*from_id),
            to: to.to_string(),
            body: body.to_string(),
            subject: Option::from(subject.to_string()),
        }
    }

    pub fn serialize(&self) -> *mut [u8] {
        let mut bytes : Vec<u8> = Vec::new();
        let mut indicator: u8 = 0;

        if self.i_subject { indicator = bit_utils::turn_bit_on( &indicator, 0 ) }
        // len to
        indicator = bit_utils::put_value(&indicator, 1, self.len_to, 1);
        bytes.push(indicator);

        let len_body_bytes = self.len_body.to_le_bytes();
        let low_len_body = len_body_bytes[0];
        bytes.push(low_len_body);

        let mut high_len_body = bit_utils::get_bits(
            &len_body_bytes[1], 0, 2);
        high_len_body = bit_utils::put_value(&high_len_body, 3, self.len_subject, 2);
        bytes.push(high_len_body);

        let mut high_len_subject = bit_utils::get_bits(
            &self.len_subject, 0, 2);

        if self.from_id.is_some() {
            high_len_subject = bit_utils::put_value(
                &high_len_subject, 3, self.from_id.unwrap(), 2);
        }
        bytes.push(high_len_subject);

        todo!()
    }

    pub fn deserialize(data: &[u8]) -> Result<Email> {
        let indicator = data[0];
        let i_subject = bit_utils::is_bit_on(&indicator, 0);
        let len_to = bit_utils::get_bits(&indicator, 1, 7);
        let len_body_high = bit_utils::get_bits(&data[2], 0, 2);
        let len_body = LittleEndian::read_u16(&[data[1], len_body_high]);

        let mut current_index: usize = 2;
        let len_subject: u8 = if i_subject {
            match bit_utils::bit_wrap(
                &data[2],
                3,
                &data[3],
                2
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
            from_id: Option::from(from_id),
            to: String::from_utf8(to).unwrap(),
            body: String::from_utf8(body).unwrap(),
            subject: subject.map(|b| String::from_utf8(b.to_vec()).unwrap()),
        })
    }
}