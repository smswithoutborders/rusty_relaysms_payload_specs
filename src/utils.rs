pub fn take_n_from<T: Clone>(items: &[T], start: usize, n: usize) -> Vec<T> {
    items[start.min(items.len())..]
        .iter()
        .take(n)
        .cloned()
        .collect()
}
