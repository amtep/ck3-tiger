//! Miscellaneous convenience functions.

use std::fmt::{Display, Formatter};

use crate::report::{tips, warn, ErrorKey};
use crate::token::Token;

/// Warns about a redefinition of a database item
pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn(ErrorKey::DuplicateItem)
        .msg(format!("{id} is redefined by another {id}"))
        .loc(other)
        .loc(key, format!("the other {id} is here"))
        .push();
}

/// Warns about an exact redefinition of a database item
pub fn exact_dup_error(key: &Token, other: &Token, id: &str) {
    warn(ErrorKey::ExactDuplicateItem)
        .msg(format!("{id} is redefined by an identical {id}"))
        .loc(other)
        .loc(key, format!("the other {id} is here"))
        .push();
}

/// Warns about a redefinition of a database item, but only at "advice" level
pub fn exact_dup_advice(key: &Token, other: &Token, id: &str) {
    tips(ErrorKey::DuplicateItem)
        .msg(format!("{id} is redefined by an identical {id}, which may cause problems if one of them is later changed"))
        .loc(other)
        .loc(key, format!("the other {id} is here"))
        .push();
}

/// Warns about a duplicate `key = value` in a database item
pub fn dup_assign_error(key: &Token, other: &Token) {
    // Don't trace back macro invocations for duplicate field errors,
    // because they're just confusing.
    let mut key = key.clone();
    key.loc.link_idx = None;
    let mut other = other.clone();
    other.loc.link_idx = None;

    warn(ErrorKey::DuplicateField)
        .msg(format!("`{other}` is redefined in a following line").as_str())
        .loc(other.loc)
        .loc(key.loc, "the other one is here")
        .push();
}

pub fn display_choices(f: &mut Formatter, v: &[&str], joiner: &str) -> Result<(), std::fmt::Error> {
    for i in 0..v.len() {
        write!(f, "{}", v[i])?;
        if i + 1 == v.len() {
        } else if i + 2 == v.len() {
            write!(f, " {joiner} ")?;
        } else {
            write!(f, ", ")?;
        }
    }
    Ok(())
}

/// The Choices enum exists to hook into the Display logic of printing to a string
enum Choices<'a> {
    OrChoices(&'a [&'a str]),
    AndChoices(&'a [&'a str]),
}

impl<'a> Display for Choices<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Choices::OrChoices(cs) => display_choices(f, cs, "or"),
            Choices::AndChoices(cs) => display_choices(f, cs, "and"),
        }
    }
}

pub fn stringify_choices(v: &[&str]) -> String {
    format!("{}", Choices::OrChoices(v))
}

pub fn stringify_list(v: &[&str]) -> String {
    format!("{}", Choices::AndChoices(v))
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TriBool {
    True,
    False,
    Maybe,
}

/// Warn if a scripted item has one of these names, and ignore it when validating.
/// This avoids tons of errors from for example a scripted effect named `if`.
/// Such an effect can happen accidentally with a misplaced brace or two.
pub const BANNED_NAMES: &[&str] = &[
    "if",
    "else",
    "else_if",
    "trigger_if",
    "trigger_else",
    "trigger_else_if",
    "while",
    "limit",
    "filter",
    "switch",
    "take_hostage", // actually used by vanilla CK3
];
