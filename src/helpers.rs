use crate::errorkey::ErrorKey;
use crate::errors::warn2;
use crate::token::Token;

/// Warns about a redefinition of a database item
pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn2(
        other,
        ErrorKey::Duplicate,
        &format!("{} is redefined by another {}", id, id),
        key,
        &format!("the other {} is here", id),
    );
}

/// Warns about a duplicate `key = value` in a database item
pub fn dup_assign_error(key: &Token, other: &Token) {
    warn2(
        other,
        ErrorKey::Duplicate,
        &format!("`{}` is redefined in a following line", other),
        key,
        "the other one is here",
    );
}
