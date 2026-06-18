use std::ops::Div;

pub fn take_n_from<T: Clone>(items: &[T], start: usize, n: usize) -> (Vec<T>, usize) {
    let taken = items[start.min(items.len())..]
        .iter()
        .take(n)
        .cloned()
        .collect();
    let left = &items[start + n..].len();
    (taken, *left)
}

pub fn calculate_b64_min_size(data_len: usize) -> usize {
    (data_len * 3).div_ceil(4)
}

