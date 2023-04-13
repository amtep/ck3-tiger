use crate::errorkey::ErrorKey;
use crate::errors::warn2;
use crate::token::Token;

/// Warns about a redefinition of a database item
pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn2(
        other,
        ErrorKey::Duplicate,
        &format!("{id} is redefined by another {id}"),
        key,
        &format!("the other {id} is here"),
    );
}

/// Warns about a duplicate `key = value` in a database item
pub fn dup_assign_error(key: &Token, other: &Token) {
    warn2(
        other,
        ErrorKey::Duplicate,
        &format!("`{other}` is redefined in a following line"),
        key,
        "the other one is here",
    );
}
