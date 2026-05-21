use std::any::Any;

#[uniffi::export]
pub fn add_rust(left: u64, right: u64) -> u64 {
    left + right
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add_rust(2, 2);
        assert_eq!(result, 4);
    }
}

pub mod contents;
pub mod bit_utils;
pub mod transports;
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

// Blanket impl so everything gets it for free
impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

uniffi::setup_scaffolding!();  // ← replaces the UDL file