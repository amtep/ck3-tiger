use std::fmt::{Display, Formatter};

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

pub fn display_choices(f: &mut Formatter, v: &[&str]) -> Result<(), std::fmt::Error> {
    for i in 0..v.len() {
        write!(f, "{}", v[i])?;
        if i + 1 == v.len() {
        } else if i + 2 == v.len() {
            write!(f, " or ")?;
        } else {
            write!(f, ", ")?;
        }
    }
    Ok(())
}

/// The Choices enum exists to hook into the Display logic of printing to a string
enum Choices<'a> {
    Choices(&'a [&'a str]),
}

impl<'a> Display for Choices<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Choices::Choices(cs) => display_choices(f, cs),
        }
    }
}

pub fn stringify_choices(v: &[&str]) -> String {
    format!("{}", Choices::Choices(v))
}
