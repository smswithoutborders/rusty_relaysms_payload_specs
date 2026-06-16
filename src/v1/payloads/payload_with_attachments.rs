use std::fmt::{Debug};
use std::sync::Arc;
use crate::{bit_utils, v1, AsAny};
use crate::v1::contents::email::V1Emails;
use crate::v1::contents::{V1ContentCategories, V1ContentVariation};
use crate::v1::payloads::{V1Payloads, V1PayloadsError};
use crate::v1::payloads::V1PayloadsError::{KeyIdTooLarge, SessionIdTooLarge};

type Result<T> = std::result::Result<T, V1PayloadsError>;

pub const ATTACHMENT_SEG_O_HEADER_SIZE: u8 = 9;
pub const ATTACHMENT_SEG_N_HEADER_SIZE: u8 = 3;

// #[derive(Debug, PartialEq, uniffi::Object)]
#[derive(Debug, PartialEq)]
pub struct V1PayloadWithAttachmentsHeader {
    version: u8,
    i_tid: bool,
    i_att: bool,
    seg_num: u8,
    sess_id: u8,
    k_id: u8,
    len_att: u16,
    t_id: Option<u32>,
    content: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct V1PayloadWithAttachmentsNoHeader {
    version: u8,
    i_tid: bool,
    i_att: bool,
    seg_num: u8,
    sess_id: u8,
    content: Vec<u8>
}

impl V1PayloadWithAttachmentsHeader {
    pub fn get_i_tid(&self) -> bool { self.i_tid }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_sess_id(&self) -> u8 { self.sess_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_t_id(&self) -> Option<u32> { self.t_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_content(&self) -> Vec<u8> { self.content.clone() }

    pub fn new(
        sess_id: u8,
        k_id: u8,
        t_id: Option<u32>,
        len_att: u16,
        payload: Vec<u8>,
    ) -> Result<Self> {
        if sess_id > (2u8.pow(7) - 1) {
            return Err(SessionIdTooLarge);
        }
        if k_id > (2u8.pow(4) - 1) {
            return Err(KeyIdTooLarge);
        }

        let i_att = !payload.is_empty();
        let i_tid = t_id.is_some();

        let version = v1::get_version();
        Ok(Self {
            version,
            i_tid,
            i_att,
            sess_id,
            seg_num: 0,
            k_id,
            t_id,
            len_att,
            content: payload,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte = bit_utils::put_value(&0, 0, self.version, 5);
        if self.i_tid { byte = bit_utils::turn_bit_on(&byte, 3) };
        if self.i_att { byte = bit_utils::turn_bit_on(&byte, 4) };
        byte = bit_utils::put_value(&byte, 5, self.sess_id, 5);
        bytes.push(byte);

        let mut byte = bit_utils::get_bits(&self.sess_id, 3, 6);
        byte = bit_utils::put_value(&byte, 4, self.seg_num, 4);
        bytes.push(byte);

        let mut byte = bit_utils::get_bits(&self.seg_num, 4, 7);
        byte = bit_utils::put_value(&byte, 4, self.k_id, 4);
        bytes.push(byte);

        if self.t_id.is_some() {
            bytes.extend(self.t_id.unwrap().to_le_bytes());
        }
        bytes.extend(self.len_att.to_le_bytes());

        bytes.extend(self.content.clone());
        Ok(bytes)
    }


    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let version = bit_utils::get_bits(&data[0], 0, 2);
        let i_tid = bit_utils::is_bit_on(&data[0], 3);
        let i_att = bit_utils::is_bit_on(&data[0], 4);
        let sess_id = match bit_utils::bit_wrap(
            &data[0], 5, &data[1], 3) {
            Ok(s) => s,
            Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
        };
        let seg_num = match bit_utils::bit_wrap(
            &data[1], 4, &data[2], 3) {
            Ok(s) => s,
            Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
        };
        if seg_num > 0 {
            return Err(V1PayloadsError::InvalidStartSegmentNumber {seg_num});
        }
        let k_id = bit_utils::get_bits(&data[2], 4, 7);
        let mut current_index = 3;
        let t_id = if i_tid {
            current_index = 7;
            Some(u32::from_le_bytes([data[3], data[4], data[5], data[6]]))
        } else { None };
        let len_att = u16::from_le_bytes([data[current_index], data[current_index+1]]);
        let payload = data[current_index + 2..].to_vec();

        Ok(V1PayloadWithAttachmentsHeader {
            version,
            i_tid,
            i_att,
            sess_id,
            seg_num,
            k_id,
            t_id,
            len_att,
            content: payload,
        })
    }
}

impl V1PayloadWithAttachmentsNoHeader {
    pub fn get_sess_id(&self) -> u8 { self.sess_id }
    pub fn get_seg_num(&self) -> u8 { self.seg_num }
    pub fn get_payload(&self) -> Vec<u8> { self.content.clone() }

    pub fn new(
        version: u8,
        seg_num: u8,
        sess_id: u8,
        payload: Vec<u8>
    ) -> Result<Self> {
        if sess_id > (2u8.pow(7) - 1) {
            return Err(SessionIdTooLarge);
        }

        Ok(Self {
            version,
            i_tid: false,
            i_att: true,
            sess_id,
            seg_num,
            content: payload,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte = bit_utils::put_value(&0, 0, self.version, 5);
        if self.i_tid { byte = bit_utils::turn_bit_on(&byte, 3) };
        if self.i_att { byte = bit_utils::turn_bit_on(&byte, 4) };
        byte = bit_utils::put_value(&byte, 5, self.sess_id, 5);
        bytes.push(byte);

        let mut byte = bit_utils::get_bits(&self.sess_id, 3, 6);
        byte = bit_utils::put_value(&byte, 4, self.seg_num, 4);
        bytes.push(byte);

        let byte = bit_utils::get_bits(&self.seg_num, 4, 7);
        bytes.push(byte);
        bytes.extend(self.content.clone());

        Ok(bytes)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let version = bit_utils::get_bits(&data[0], 0, 2);
        let i_tid = bit_utils::is_bit_on(&data[0], 3);
        let i_att = bit_utils::is_bit_on(&data[0], 4);
        let sess_id = match bit_utils::bit_wrap(
            &data[0], 5, &data[1], 3) {
            Ok(s) => s,
            Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
        };
        let seg_num = match bit_utils::bit_wrap(
            &data[1], 4, &data[2], 3) {
            Ok(s) => s,
            Err(e) => return Err(V1PayloadsError::ErrorParsingBits{ error: e }),
        };
        let payload = data[3..].to_vec();
        Ok(
            V1PayloadWithAttachmentsNoHeader {
                version,
                i_tid,
                i_att,
                sess_id,
                seg_num,
                content: payload
            }
        )
    }

}

