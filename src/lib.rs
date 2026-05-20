use std::any::Any;
use std::fmt::Debug;
use crate::contents::email::Emails;

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
pub mod transport;

#[derive(Debug)]
pub enum ContentError {
    InconsistentSubjectIndicator,
    FromIdTooLarge,
    SubjectLenTooLarge,
    ToTooLarge,
    BitParsingError,
    InvalidUtf8
}
pub trait Contents : Debug {
    fn serialize(&self) -> Result<Vec<u8>, ContentError>;
    fn deserialize(data: &[u8]) -> Result<Emails, ContentError>
    where Self: Sized;

    fn equals(&self, other: &dyn Contents) -> bool;

    fn as_any(&self) -> &dyn Any;
}

uniffi::setup_scaffolding!();  // ← replaces the UDL file