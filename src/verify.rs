use crate::errors::Errors;
use crate::scope::Scope;

pub use ck3_mod_validator_derive::Verify;

pub trait Verify {
    fn from_scope(scope: Scope, errors: &mut Errors) -> Self;
}
