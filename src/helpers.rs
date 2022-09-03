use crate::errorkey::ErrorKey;
use crate::errors::warn2;
use crate::token::Token;

pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn2(
        other,
        ErrorKey::Duplicate,
        &format!("{} is redefined by another {}", id, id),
        key,
        &format!("the other {} is here", id),
    );
}
