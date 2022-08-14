use thiserror::Error;

use crate::scope::Scope;

#[derive(Clone, Debug, Error)]
pub enum ValidationError {
    #[error("Required field missing ({0})")]
    RequiredFieldMissing(String),
    #[error("Required field was invalid ({0})")]
    RequiredFieldInvalid(String),
}

pub trait Validate {
    fn from_scope(scope: Scope) -> Result<Self, ValidationError>
    where
        Self: Sized;
}
