use std::sync::Arc;
use crate::{bit_utils, utils};
use crate::contents::{deserialize_for_content, Contents};
use crate::contents::email::Emails;
use crate::payloads::{Payloads, PayloadsError};
use crate::payloads::payload_without_attachment::PayloadWithoutAttachment;
use crate::payloads::PayloadsError::{CategoryIdTooLarge, ContentDeserializationError, DeviceIdTooLarge, EncryptionIdTooLarge, HeaderNWithDeviceIdTooLarge, HeaderWithDeviceIdTooLarge, HeaderWithoutDeviceIdTooLarge, KeyIdTooLarge, PayloadTooLarge, SessionIdTooLarge, VersionTooLarge};

const MAX_SPLIT_0_WITHOUT_DID: u8 = 132;
const MAX_SPLIT_0_WITH_DID: u8 = 116;
const MAX_SPLIT_N: u8 = 136;
const SEG_0_HEADER_SIZE_WITHOUT_DID: u8 = 6;
const SEG_0_HEADER_SIZE_WITH_DID: u8 = 22;
const SEG_N_HEADER_SIZE: u8 = 2;

#[derive(Debug, uniffi::Object)]
pub struct PayloadWithAttachments {
    i_did: bool,
    i_att: bool,
    i_end: bool,
    version: u8,
    seg_num: u8,
    sess_id: u8,
    k_id: u8,
    e_id: u8,
    cat_id: u8,
    len_att: u16,
    device_id: Option<Vec<u8>>,
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
    pub fn get_i_did(&self) -> bool { self.i_did }
    pub fn get_i_att(&self) -> bool { self.i_att }
    pub fn get_i_cont(&self) -> bool { self.i_end }
    pub fn get_version(&self) -> u8 { self.version }
    pub fn get_seg_num(&self) -> u8 { self.seg_num }
    pub fn get_sess_id(&self) -> u8 { self.sess_id }
    pub fn get_e_id(&self) -> u8 { self.e_id }
    pub fn get_k_id(&self) -> u8 { self.k_id }
    pub fn get_cat_id(&self) -> u8 { self.cat_id }
    pub fn get_len_att(&self) -> u16 { self.len_att }
    pub fn get_device_id(&self) -> Option<Vec<u8>> { self.device_id.clone() }
    pub fn get_payload_content(&self) -> Vec<u8> { self.payload.clone() }

    pub fn calculate_segments(
        &self,
        is_did: bool,
    ) -> u32 {
        let payload_len = self.payload.len() as u32;
        let seg0_header_size = if is_did { SEG_0_HEADER_SIZE_WITH_DID }
        else { SEG_0_HEADER_SIZE_WITHOUT_DID} as u32;
        let max_value = if is_did { MAX_SPLIT_0_WITH_DID + SEG_0_HEADER_SIZE_WITH_DID }
        else { MAX_SPLIT_0_WITHOUT_DID + SEG_0_HEADER_SIZE_WITHOUT_DID } as u32;
        // (x + seg0_header_size + (SEG_N_HEADER_SIZE as u32 * (x.div_ceil(max_value) - 1)));
        (payload_len + seg0_header_size +
            (SEG_N_HEADER_SIZE as u32 * (payload_len.div_ceil(max_value) - 1)))
            .div_ceil(max_value)
        // 1 + 22 + (2 * (1 - 1))
    }



    #[uniffi::constructor]
    pub fn instance() -> Result<Arc<Self>, PayloadsError> {
        Ok(Arc::new(Self {
            i_did: false,
            i_att: false,
            i_end: false,
            version: 255,
            seg_num: 255,
            sess_id: 255,
            k_id: 255,
            e_id: 255,
            cat_id: 255,
            len_att: 0,
            device_id: None,
            payload: Vec::new(),
        }))
    }

    #[uniffi::constructor]
    pub fn new(
        version: u8,
        sess_id: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        len_att: u16,
        device_id: Option<Vec<u8>>,
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
        if cat_id > (2u8.pow(4) - 1) {
            return Err(CategoryIdTooLarge);
        }

        let i_did = device_id.as_ref().map_or(false, |v| !v.is_empty());
        if i_did && (device_id.clone().unwrap().len() > u16::MAX as usize) {
            return Err(DeviceIdTooLarge);
        }
        let i_att = !payload.is_empty();
        let i_end = false; // TODO: change value after parsing

        Ok(Arc::new(Self {
            i_did,
            i_att,
            i_end,
            version,
            seg_num: 0,
            sess_id,
            e_id,
            k_id,
            cat_id,
            len_att,
            device_id,
            payload,
        }))
    }


    #[uniffi::constructor]
    pub fn new_segment(
        version: u8,
        seg_num: u8,
        sess_id: u8,
        e_id: u8,
        k_id: u8,
        cat_id: u8,
        len_att: u16,
        device_id: Option<Vec<u8>>,
        payload: Vec<u8>,
    ) -> Result<Arc<Self>, PayloadsError> {
        let i_did = device_id.as_ref().map_or(false, |v| !v.is_empty());

        let max_value = if i_did { MAX_SPLIT_0_WITH_DID } else { MAX_SPLIT_0_WITHOUT_DID };
        if payload.len() as u32 > max_value as u32 {
            return Err(PayloadTooLarge {
                current: payload.len() as i32,
                max: max_value,
            })
        }

        if i_did && (device_id.clone().unwrap().len() > u16::MAX as usize) {
            return Err(DeviceIdTooLarge);
        }
        let i_att = !payload.is_empty();
        let i_end = false; // TODO: change value after parsing

        Ok(Arc::new(Self {
            i_did,
            i_att,
            i_end,
            version,
            seg_num,
            sess_id,
            e_id,
            k_id,
            cat_id,
            len_att,
            device_id,
            payload,
        }))
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<Arc<Self>, PayloadsError> {
        let i_did = bit_utils::is_bit_on(&data[0], 0);
        let i_att = bit_utils::is_bit_on(&data[0], 1);
        let i_end = bit_utils::is_bit_on(&data[0], 2);
        let version = bit_utils::get_bits(&data[0], 3, 7);
        let seg_num = data[1];
        let sess_id = bit_utils::get_bits(&data[2], 0, 3);
        let k_id = bit_utils::get_bits(&data[2], 4, 7);
        let e_id = bit_utils::get_bits(&data[3], 0, 2);
        let cat_id = bit_utils::get_bits(&data[3], 3, 6);
        let len_att = u16::from_le_bytes([data[4], data[5]]);

        let mut current_index: usize = 6;
        let device_id = if i_did {
            let device_id = Option::from(data[6..(6 + 16)].to_vec());
            current_index += 16;
            device_id
        } else { None };

        let payload = data[current_index..].to_vec();

        Ok(Arc::new(Self {
            i_did,
            i_att,
            i_end,
            version,
            seg_num,
            sess_id,
            k_id,
            e_id,
            cat_id,
            len_att,
            device_id,
            payload
        }))
    }

    /**
    Assumption, payload already processed just needs splitting for transmission
    **/
    pub fn split(&self) -> crate::payloads::Result<Vec<Arc<dyn Payloads>>> {
        let mut splits: Vec<Arc<dyn Payloads>> = Vec::new();

        let mut seg_num :u8 = 0;

        let mut max_value = if self.i_did { MAX_SPLIT_0_WITH_DID } else { MAX_SPLIT_0_WITHOUT_DID };
        let items = utils::take_n_from(&self.payload, 0, max_value as usize);
        let mut start_index: usize = items.len();
        let transport = match PayloadWithAttachments::new_segment(
            self.version,
            seg_num,
            self.sess_id,
            self.k_id,
            self.e_id,
            self.cat_id,
            self.payload.len() as u16,
            self.device_id.clone(),
            items,
        ) {
            Ok(transport) => transport,
            Err(e) => { return Err(PayloadsError::from(e)); }
        };
        splits.push(transport);

        while start_index < self.payload.len() {
            let items = utils::take_n_from(&self.payload, start_index, MAX_SPLIT_N as usize);
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

        let mut byte1 : u8 = if self.i_did { 1 } else { 0 };
        if self.i_att { byte1 = bit_utils::turn_bit_on(&byte1, 1); }
        if self.i_end { byte1 = bit_utils::turn_bit_on(&byte1, 2); }
        byte1 = bit_utils::put_value(&byte1, 3, self.version, 4);
        bytes.push(byte1);

        bytes.push(self.seg_num);

        let byte3 = bit_utils::put_value(&self.sess_id, 4, self.k_id, 4);
        bytes.push(byte3);

        let byte4 = bit_utils::put_value(&self.e_id, 3, self.cat_id, 4);
        bytes.push(byte4);

        let len_att = u16::to_le_bytes(self.len_att);
        bytes.push(len_att[0]);
        bytes.push(len_att[1]);

        if bytes.len() > SEG_0_HEADER_SIZE_WITHOUT_DID as usize {
            return Err(HeaderWithoutDeviceIdTooLarge {
                current: bytes.len() as i32,
                max: SEG_0_HEADER_SIZE_WITHOUT_DID,
            })
        }

        if self.device_id.is_some() {
            bytes.extend(self.device_id.clone().unwrap());
        }

        if bytes.len() > SEG_0_HEADER_SIZE_WITH_DID as usize {
            return Err(HeaderWithDeviceIdTooLarge {
                current: bytes.len() as i32,
                max: SEG_0_HEADER_SIZE_WITHOUT_DID,
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
            return Err(HeaderNWithDeviceIdTooLarge {
                segment: self.seg_num,
                current: payload.len() as i32,
                max: SEG_0_HEADER_SIZE_WITHOUT_DID,
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
        self.i_did == other.i_did
            && self.i_att == other.i_att
            && self.i_end == other.i_end
            && self.version == other.version
            && self.seg_num == other.seg_num
            && self.sess_id == other.sess_id
            && self.e_id == other.e_id
            && self.k_id == other.k_id
            && self.cat_id == other.cat_id
            && self.len_att == other.len_att
            && self.device_id == other.device_id
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
    let from_id: u8 = 7; // 1
    let email = Emails::new(
        to,
        body,
        Option::from(subject.to_string()),
        &from_id
    ).unwrap();

    let version: u8 = 1;
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let cat_id: u8 = email.get_cat_id();
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());

    let mut payload = email.serialize().unwrap();
    let att = rand::random::<[u8; (140*10)]>().to_vec();
    let len_att = att.len() as u16;
    payload.extend(att);

    let payload_with_attachment = PayloadWithAttachments::new_segment(
        version,
        seg_num,
        sess_num,
        e_id,
        k_id,
        cat_id,
        len_att,
        device_id,
        payload.clone(),
    ).unwrap();

    let serialized = payload_with_attachment.serialize().unwrap();
    let deserialized = PayloadWithAttachments::instance().unwrap()
        .deserialize(&serialized).unwrap();
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
    let mut ins = PayloadWithAttachments::instance().unwrap();
    if let Some(val) = Arc::get_mut(&mut ins) {
        val.payload = rand::random::<[u8; 1]>().to_vec();

        let expected = 1;
        assert_eq!(expected, val.calculate_segments(true));

        val.payload = rand::random::<[u8; 300]>().to_vec();
        let expected = 3;
        assert_eq!(expected, val.calculate_segments(true));

        val.payload = rand::random::<[u8; 1]>().to_vec();
        let expected = 1;
        assert_eq!(expected, val.calculate_segments(false));

        val.payload = rand::random::<[u8; 300]>().to_vec();
        let expected = 3;
        assert_eq!(expected, val.calculate_segments(false));
    }


}

#[test]
fn att_split() {
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

    let version: u8 = 1;
    let seg_num: u8 = 1;
    let sess_num: u8 = 1;
    let e_id: u8 = 1;
    let k_id: u8 = 1;
    let cat_id: u8 = email.get_cat_id();
    let device_id: Option<Vec<u8>> = Some(rand::random::<[u8; 16]>().to_vec());

    // let mut payload = email.serialize().unwrap();
    // const len_att: usize = MAX_SPLIT_0_WITH_DID as usize + 1;
    const LEN_ATT: usize = MAX_SPLIT_0_WITH_DID as usize * 50;
    let payload = rand::random::<[u8; LEN_ATT]>().to_vec();

    let payload_with_attachment = PayloadWithAttachments::new(
        version,
        sess_num,
        e_id,
        k_id,
        cat_id,
        LEN_ATT as u16,
        device_id,
        payload.clone(),
    ).unwrap();

    let split = payload_with_attachment.split().unwrap();
    let expected = payload_with_attachment.calculate_segments(true);
    assert_eq!(expected, split.len() as u32);
    let mut serialized = split[0].serialize().unwrap();
    assert_eq!(138, serialized.len());

    serialized = split[1].serialize().unwrap();
    assert_eq!(138, serialized.len());
}
