pub fn bit_wrap<T: From<u8>>(
    byte1: &u8,
    byte1_start_index: i8,
    byte2: &u8,
    byte2_end_index: i8,
) -> T {
    let low = byte1 >> byte1_start_index;
    let high = byte2 & ((1 << byte2_end_index as u8) - 1);
    T::from(high << byte1_start_index | low)
}

pub fn is_bit_on(data: &u8, index: u8) -> bool {
    ((data >> index) & 1) == 1
}

pub fn get_bits(data: &u8, start_index: u8, end_index: u8) -> u8 {
    let mask = ((1 << (end_index - start_index + 1)) - 1) << start_index;
    (*data & mask) >> start_index
}
