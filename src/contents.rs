use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use crate::AsAny;
use crate::contents::email::Emails;

pub mod email;
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ContentError {
    #[error("Inconsistent subject indicator")]
    InconsistentSubjectIndicator,
    #[error("From ID too large")]
    FromIdTooLarge,
    #[error("Subject length too large")]
    SubjectLenTooLarge,
    #[error("To too large")]
    ToTooLarge,
    #[error("Bit parsing error")]
    BitParsingError,
    #[error("Invalid utf-8")]
    InvalidUtf8
}
#[uniffi::export(with_foreign)]
pub trait Contents: Debug + Send + Sync {
    fn serialize(&self) -> Result<Vec<u8>, ContentError>;
    fn equals(&self, other: Arc<dyn Contents>) -> bool;
}
