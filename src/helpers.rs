use std::fmt::{Display, Formatter};

use crate::report::{
    advice2, log, warn2, Confidence, ErrorKey, LogLevel, LogReport, PointedMessage, Severity,
};
use crate::token::Token;

/// Warns about a redefinition of a database item
pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn2(
        other,
        ErrorKey::DuplicateItem,
        &format!("{id} is redefined by another {id}"),
        key,
        &format!("the other {id} is here"),
    );
}

/// Warns about an exact redefinition of a database item
pub fn exact_dup_error(key: &Token, other: &Token, id: &str) {
    warn2(
        other,
        ErrorKey::ExactDuplicateItem,
        &format!("{id} is redefined by an identical {id}"),
        key,
        &format!("the other {id} is here"),
    );
}

/// Warns about a redefinition of a database item, but only at "advice" level
pub fn dup_advice(key: &Token, other: &Token, id: &str) {
    advice2(
        other,
        ErrorKey::DuplicateItem,
        &format!("{id} is redefined by another {id}, which may cause problems if one of them is later changed"),
        key,
        &format!("the other {id} is here"),
    );
}

/// Warns about a duplicate `key = value` in a database item
pub fn dup_assign_error(key: &Token, other: &Token) {
    // Don't trace back macro invocations for duplicate field errors,
    // because they're just confusing.
    let mut key = key.clone();
    key.loc.link = None;
    let mut other = other.clone();
    other.loc.link = None;

    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key: ErrorKey::DuplicateField,
        msg: format!("`{other}` is redefined in a following line").as_str(),
        info: None,
        pointers: vec![
            PointedMessage {
                location: other.loc.clone(),
                length: other.as_str().len(),
                msg: None,
            },
            PointedMessage {
                location: key.loc.clone(),
                length: key.as_str().len(),
                msg: Some("the other one is here"),
            },
        ],
    });
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
