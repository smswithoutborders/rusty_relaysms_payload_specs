use std::sync::Arc;
use crate::{bit_utils, utils};
use crate::contents::{deserialize_for_content, ContentCategories, Contents};
use crate::contents::contents_container::ContentsContainer;
use crate::contents::email::Emails;
use crate::payloads::{Payloads, PayloadsError};
use crate::payloads::payload_without_attachment::PayloadWithoutAttachments;
use crate::payloads::PayloadsError::{CategoryIdTooLarge, ContentDeserializationError, DeviceIdTooLarge, EncryptionIdTooLarge, HeaderTooLarge, KeyIdTooLarge, NHeaderTooLarge, PayloadTooLarge, SessionIdTooLarge, VersionTooLarge};

const SEG_0_HEADER_SIZE: u8 = 10;
const SEG_N_HEADER_SIZE: u8 = 2;
const MAX_PAYLOAD_SIZE: u8 = 138;

#[derive(Debug, uniffi::Object)]
pub struct PayloadWithAttachments {
    i_att: bool,
    version: u8,
    seg_num: u8,
    sess_id: u8,
    k_id: u8,
    e_id: u8,
    len_att: u16,
    t_id: u32,
    payload: Vec<u8>,
}

#[derive(Debug, uniffi::Object)]
pub struct PayloadWithAttachmentsNoHeader {
    seg_num: u8,
    sess_id: u8,
    payload: Vec<u8>
}


#[uniffi::export]
impl PayloadWithAttachments {
    pub fn get_i_att(&self) -> bool { self.i_att }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_seg_num(&self) -> u8 { self.seg_num }
    pub fn get_sess_id(&self) -> u8 { self.sess_id }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_t_id(&self) -> u32 { self.t_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_payload_content(&self) -> Vec<u8> { self.payload.clone() }

    pub fn calculate_segments(&self) -> u32 {
        let payload_len = self.payload.len() as u32;
        (payload_len + SEG_0_HEADER_SIZE as u32 +
            (SEG_N_HEADER_SIZE as u32 * (payload_len.div_ceil(MAX_PAYLOAD_SIZE as u32) - 1)))
            .div_ceil(MAX_PAYLOAD_SIZE as u32)
    }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        sess_id: u8,
        e_id: u8,
        k_id: u8,
        t_id: u32,
        len_att: u16,
        payload: Vec<u8>,
    ) -> Result<Arc<Self>, PayloadsError> {
        if version > (2u8.pow(4) - 1) {
            return Err(VersionTooLarge);
        }
        if sess_id > (2u8.pow(4) - 1) {
            return Err(SessionIdTooLarge);
        }
        if e_id > (2u8.pow(3) - 1) {
            return Err(EncryptionIdTooLarge);
        }
        if k_id > (2u8.pow(4) - 1) {
            return Err(KeyIdTooLarge);
        }

        let i_att = !payload.is_empty();

        Ok(Arc::new(Self {
            i_att,
            version,
            seg_num: 0,
            sess_id,
            e_id,
            k_id,
            t_id,
            len_att,
            payload,
        }))
    }


    #[uniffi::constructor]
    pub fn new_segment(
        version: u8,
        seg_num: u8,
        sess_id: u8,
        k_id: u8,
        e_id: u8,
        t_id: u32,
        len_att: u16,
        payload: Vec<u8>,
    ) -> Result<Arc<Self>, PayloadsError> {
        if payload.len() as u32 > MAX_PAYLOAD_SIZE as u32 {
            return Err(PayloadTooLarge {
                current: payload.len() as i32,
                max: MAX_PAYLOAD_SIZE,
            })
        }

        let i_att = !payload.is_empty();

        Ok(Arc::new(Self {
            i_att,
            version,
            seg_num,
            sess_id,
            e_id,
            k_id,
            t_id,
            len_att,
            payload,
        }))
    }


    /**
    Assumption, payload already processed just needs splitting for transmission
    **/
    pub fn split(&self) -> crate::payloads::Result<Vec<Arc<dyn Payloads>>> {
        let mut splits: Vec<Arc<dyn Payloads>> = Vec::new();

        let mut seg_num :u8 = 0;

        let max_value = MAX_PAYLOAD_SIZE - SEG_0_HEADER_SIZE;
        let items = utils::take_n_from(&self.payload, 0, max_value as usize);
        let mut start_index: usize = items.len();
        let transport = match PayloadWithAttachments::new_segment(
            self.version,
            seg_num,
            self.sess_id,
            self.k_id,
            self.e_id,
            self.t_id,
            self.payload.len() as u16,
            items,
        ) {
            Ok(transport) => transport,
            Err(e) => { return Err(PayloadsError::from(e)); }
        };
        splits.push(transport);


        let max_value = MAX_PAYLOAD_SIZE - SEG_N_HEADER_SIZE;
        while start_index < self.payload.len() {
            let items = utils::take_n_from(&self.payload, start_index, max_value as usize);
            start_index += items.len();
            let transport = match PayloadWithAttachmentsNoHeader::new(
                seg_num,
                self.sess_id,
                items,
            ) {
                Ok(transport) => transport,
                Err(e) => { return Err(PayloadsError::from(e)); }
            };
            splits.push(transport);
            seg_num += 1;
        }

        Ok(splits)
    }

}


#[uniffi::export]
pub fn deserialize_payload_with_attachments(
    data: &[u8]
) -> Result<Arc<PayloadWithAttachments>, PayloadsError> {
    let version = bit_utils::get_bits(&data[0], 0, 2);
    let i_att = bit_utils::is_bit_on(&data[0], 3);
    let sess_id = match bit_utils::bit_wrap(
        &data[0], 4, &data[1], 3) {
        Ok(s) => s,
        Err(e) => return Err(PayloadsError::ErrorParsingBits{ error: e }),
    };
    let seg_num = match bit_utils::bit_wrap(
        &data[1], 4, &data[2], 3) {
        Ok(s) => s,
        Err(e) => return Err(PayloadsError::ErrorParsingBits{ error: e }),
    };
    let k_id = bit_utils::get_bits(&data[2], 4, 7);
    let e_id = bit_utils::get_bits(&data[3], 0, 2);
    let t_id = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    let len_att = u16::from_le_bytes([data[8], data[9]]);
    let payload = data[10..].to_vec();

    Ok(Arc::new(PayloadWithAttachments {
        i_att,
        version,
        seg_num,
        sess_id,
        k_id,
        e_id,
        t_id,
        len_att,
        payload
    }))
}


#[uniffi::export]
impl PayloadWithAttachmentsNoHeader {

    fn get_seg_num(&self) -> u8 { self.seg_num }
    fn get_sess_id(&self) -> u8 { self.sess_id }
    fn get_payload(&self) -> Vec<u8> { self.payload.clone() }

    #[uniffi::constructor]
    pub fn instance() -> Result<Arc<Self>, PayloadsError> {
        Ok(Arc::new(Self {
            seg_num: 255,
            sess_id: 255,
            payload: Vec::new(),
        }))
    }

    #[uniffi::constructor]
    pub fn new(
        seg_num: u8,
        sess_id: u8,
        payload: Vec<u8>
    ) -> Result<Arc<Self>, PayloadsError> {
        if sess_id > (2u8.pow(4) - 1) {
            return Err(SessionIdTooLarge);
        }
        Ok(Arc::new(Self {
            seg_num,
            sess_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, PayloadsError> {
        let seg_num = data[0];
        let sess_id = data[1];
        let payload = data[2..].to_vec();

        Ok(Arc::new(Self {
            seg_num,
            sess_id,
            payload
        }))
    }
}


#[uniffi::export]
impl Payloads for PayloadWithAttachments {
    fn serialize(&self) -> crate::payloads::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();

        let mut byte = bit_utils::put_value(&self.version, 4, self.sess_id, 4);
        if self.i_att { byte = bit_utils::turn_bit_on(&byte, 3) };
        bytes.push(byte);

        let mut byte = bit_utils::get_bits(&self.sess_id, 4, 7);
        byte = bit_utils::put_value(&byte, 4, self.seg_num, 4);
        bytes.push(byte);

        let mut byte = bit_utils::get_bits(&self.seg_num, 4, 7);
        byte = bit_utils::put_value(&byte, 4, self.k_id, 4);
        bytes.push(byte);

        bytes.push(self.e_id);
        bytes.extend(self.t_id.to_le_bytes());
        bytes.extend(self.len_att.to_le_bytes());

        if bytes.len() > SEG_0_HEADER_SIZE as usize {
            return Err(HeaderTooLarge {
                current: bytes.len() as i32,
                max: SEG_0_HEADER_SIZE,
            })
        }

        bytes.extend(self.payload.clone());

        Ok(bytes)
    }

    fn equals(&self, other: Arc<dyn Payloads>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

}


#[uniffi::export]
impl Payloads for PayloadWithAttachmentsNoHeader {
    fn serialize(&self) -> crate::payloads::Result<Vec<u8>> {
        let mut payload = vec![self.sess_id, self.seg_num];
        if payload.len() > SEG_N_HEADER_SIZE as usize {
            return Err(NHeaderTooLarge {
                segment: self.seg_num,
                current: payload.len() as i32,
                max: SEG_N_HEADER_SIZE,
            })
        }
        payload.extend(self.payload.clone());
        Ok(payload)
    }

    fn equals(&self, other: Arc<dyn Payloads>) -> bool {
        match (self.serialize(), other.serialize()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }
}


impl PartialEq for PayloadWithAttachments {
    fn eq(&self, other: &Self) -> bool {
        self.i_att == other.i_att
            && self.version == other.version
            && self.seg_num == other.seg_num
            && self.sess_id == other.sess_id
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.t_id == other.t_id
            && self.len_att == other.len_att
            && self.payload == other.payload
    }
}

impl PartialEq for PayloadWithAttachmentsNoHeader {
    fn eq(&self, other: &Self) -> bool {
        self.seg_num == other.seg_num
            && self.sess_id == other.sess_id
            && self.payload == other.payload
    }
}


#[test]
fn att_true_n_serialize() {
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let email = Emails::new(
        to,
        body,
        Option::from(subject.to_string()),
    ).unwrap();

    let version: u8 = 1;
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let f_id: u32 = 255;

    let mut payload = email.serialize().unwrap();
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;
    payload.extend(att);

    let payload_with_attachment = PayloadWithAttachments::new(
        version,
        sess_num,
        k_id,
        e_id,
        f_id,
        len_att,
        payload.clone(),
    ).unwrap();

    let serialized = payload_with_attachment.serialize().unwrap();
    let deserialized = deserialize_payload_with_attachments(&serialized).unwrap();
    assert_eq!(payload_with_attachment, deserialized);

    let payload_with_n_attachment = PayloadWithAttachmentsNoHeader::new(
        seg_num,
        sess_num,
        payload,
    ).unwrap();

    let serialized = payload_with_n_attachment.serialize().unwrap();
    let deserialized = PayloadWithAttachmentsNoHeader::instance().unwrap()
        .deserialize(&serialized).unwrap();
    assert_eq!(payload_with_n_attachment, deserialized);
}


#[test]
fn test_calculate_segments() {
    let version: u8 = 1;
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let f_id: u32 = 255;
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());
    const LEN_ATT: usize = SEG_0_HEADER_SIZE as usize * 50;
    let payload = rand::random::<[u8; LEN_ATT]>().to_vec();
    let mut ins = PayloadWithAttachments::new(
        version,
        seg_num,
        sess_num,
        e_id,
        f_id,
        LEN_ATT as u16,
        payload.clone(),
    ).unwrap();
    if let Some(val) = Arc::get_mut(&mut ins) {
        val.payload = rand::random::<[u8; 1]>().to_vec();

        let expected = 1;
        assert_eq!(expected, val.calculate_segments());

        val.payload = rand::random::<[u8; 300]>().to_vec();
        let expected = 3;
        assert_eq!(expected, val.calculate_segments());

        val.payload = rand::random::<[u8; 1]>().to_vec();
        let expected = 1;
        assert_eq!(expected, val.calculate_segments());

        val.payload = rand::random::<[u8; 300]>().to_vec();
        let expected = 3;
        assert_eq!(expected, val.calculate_segments());
    }
}

#[test]
fn att_split() {
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let from_id: u8 = 7; // 1

    let container = ContentsContainer::new(
        ContentCategories::Email,
        body.to_string(),
        Some(to.to_string()),
        Some(subject.to_string())
    );
    let email = container.instance().unwrap();

    let version: u8 = 1;
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let f_id: u32 = 255;
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());

    // let mut payload = email.serialize().unwrap();
    // const len_att: usize = MAX_SPLIT_0_WITH_DID as usize + 1;
    const LEN_ATT: usize = SEG_0_HEADER_SIZE as usize * 50;
    let payload = rand::random::<[u8; LEN_ATT]>().to_vec();

    let payload_with_attachment = PayloadWithAttachments::new(
        version,
        sess_num,
        k_id,
        e_id,
        f_id,
        LEN_ATT as u16,
        payload.clone(),
    ).unwrap();

    let split = payload_with_attachment.split().unwrap();
    let expected = payload_with_attachment.calculate_segments();
    assert_eq!(expected, split.len() as u32);
    let mut serialized = split[0].serialize().unwrap();
    assert_eq!(138, serialized.len());

    serialized = split[1].serialize().unwrap();
    assert_eq!(138, serialized.len());
}
