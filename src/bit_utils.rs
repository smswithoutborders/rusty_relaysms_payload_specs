use std::fmt::Formatter;

#[derive(Debug)]
pub enum BitParsingError {
    IndexOutOfBounds,
    ExpectedLargerThanOctet,
}

impl std::fmt::Display for BitParsingError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            BitParsingError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            BitParsingError::ExpectedLargerThanOctet => write!(f, "Expected larger than octets"),
        }
    }
}

pub fn bit_wrap<T: From<u8>>(
    byte1: &u8,
    byte1_start_index: i8,
    byte2: &u8,
    byte2_end_index: i8,
) -> Result<T, BitParsingError> {
    if byte1_start_index > 7 || byte2_end_index > 6 {
        return Err(BitParsingError::IndexOutOfBounds);
    }

    if ((8 - byte1_start_index) + byte2_end_index) > 7 {
        return Err(BitParsingError::ExpectedLargerThanOctet);
    }

    let low = byte1 >> byte1_start_index;
    let high = byte2 & ((1 << (byte2_end_index + 1) as u8) - 1);
    Ok(T::from((high << (8 - byte1_start_index)) | low))
}

#[test]
fn test_bit_wrap() {
    let byte1: u8 = 160;
    let byte2: u8 = 27;
    let expected: u8 = 221;
    let output: u8 = bit_wrap(
        &byte1,
        5,
        &byte2,
        4
    ).unwrap();
    assert_eq!(expected, output);

    let byte1: u8 = 234;
    let byte2: u8 = 91;
    let expected: u8 = 183;
    let output1: u8 = bit_wrap(
        &byte1,
        7,
        &byte2,
        6
    ).unwrap();
    assert_eq!(expected, output1);
}

pub fn is_bit_on(data: &u8, index: u8) -> bool {
    ((data >> index) & 1) == 1
}

#[test]
fn test_is_bit_on() {
    assert!(is_bit_on(&7, 1));
    assert!(!is_bit_on(&8, 1));
}

pub fn get_bits(data: &u8, start_index: u8, end_index: u8) -> u8 {
    let offset = ((end_index - (start_index)) + 1) as usize;
    let mask = ((1 << offset) - 1) << start_index;
    (*data & mask) >> start_index
}

#[test]
fn test_get_bits() {
    let value: u8 = 23;
    let expected: u8 = 7;
    let output = get_bits(&value, 0, 2);
    assert_eq!(expected, output);

    let value: u8 = 103;
    let expected: u8 = 39;
    let output = get_bits(&value, 0, 5);
    assert_eq!(expected, output);
}

pub fn turn_bit_on(data: &u8, index: u8) -> u8 {
    data | ((1 << index + 1) - 1)
}

#[test]
fn test_turn_bit_on() {
    let value: u8 = 8;
    let expected: u8 = 11;
    let output = turn_bit_on(&value, 1);
    assert_eq!(expected, output);
}

pub fn put_value(low: &u8, start_index: u8, high: u8, offset: u8) -> u8 {
    let offset_value = if offset > 0 {
        let end_index = 7 - offset;
        get_bits(&high, 0, end_index)
    } else { high };
    low | (offset_value << start_index)
}

#[test]
fn test_put_value() {
    let byte1: u8 = 7;
    let byte2: u8 = 8;
    let expected: u8 = 71;
    let output = put_value(&byte1, 3, byte2, 0);
    assert_eq!(expected, output);

    let byte1: u8 = 3;
    let byte2: u8 = 103;
    let expected: u8 = 159;
    let output = put_value(&byte1, 2, byte2, 2);
    assert_eq!(expected, output);
}

