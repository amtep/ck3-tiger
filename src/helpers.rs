use crate::errorkey::ErrorKey;
use crate::errors::error2;
use crate::token::Token;

pub fn dup_error(key: &Token, other: &Token, id: &str) {
    error2(
        other,
        ErrorKey::Duplicate,
        &format!("{} is redefined by another {}", id, id),
        key,
        &format!("the other {} is here", id),
    );
}
