//! Miscellaneous convenience functions.
use ahash::{HashMap, HashSet, RandomState};
use bimap::BiHashMap;

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::report::{ErrorKey, tips, warn};
use crate::token::Token;

pub type TigerHashMap<K, V> = HashMap<K, V>;
pub type TigerHashSet<T> = HashSet<T>;

/// Warns about a redefinition of a database item
pub fn dup_error(key: &Token, other: &Token, id: &str) {
    warn(ErrorKey::DuplicateItem)
        .msg(format!("{id} is redefined by another {id}"))
        .loc(other)
        .loc_msg(key, format!("the other {id} is here"))
        .push();
}

/// Warns about an exact redefinition of a database item
pub fn exact_dup_error(key: &Token, other: &Token, id: &str) {
    warn(ErrorKey::ExactDuplicateItem)
        .msg(format!("{id} is redefined by an identical {id}"))
        .loc(other)
        .loc_msg(key, format!("the other {id} is here"))
        .push();
}

/// Warns about a redefinition of a database item, but only at "advice" level
pub fn exact_dup_advice(key: &Token, other: &Token, id: &str) {
    tips(ErrorKey::DuplicateItem)
        .msg(format!("{id} is redefined by an identical {id}, which may cause problems if one of them is later changed"))
        .loc(other)
        .loc_msg(key, format!("the other {id} is here"))
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
        .loc_msg(key.loc, "the other one is here")
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

impl Display for Choices<'_> {
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

pub(crate) type BiTigerHashMap<L, R> = BiHashMap<L, R, RandomState, RandomState>;

#[derive(Debug, Clone)]
pub(crate) enum ActionOrEvent {
    Action(Token),
    Event(Token, &'static str, usize),
}

impl ActionOrEvent {
    pub(crate) fn new_action(key: Token) -> Self {
        Self::Action(key)
    }

    pub(crate) fn new_event(key: Token) -> Self {
        if let Some((namespace, nr)) = key.as_str().split_once('.') {
            if let Ok(nr) = usize::from_str(nr) {
                return Self::Event(key, namespace, nr);
            }
        }
        let namespace = key.as_str();
        Self::Event(key, namespace, 0)
    }

    pub(crate) fn token(&self) -> &Token {
        match self {
            Self::Action(token) | Self::Event(token, _, _) => token,
        }
    }
}

impl PartialEq for ActionOrEvent {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Action(token) => {
                if let Self::Action(other_token) = other {
                    token == other_token
                } else {
                    false
                }
            }
            Self::Event(_, namespace, nr) => {
                if let Self::Event(_, other_namespace, other_nr) = other {
                    namespace == other_namespace && nr == other_nr
                } else {
                    false
                }
            }
        }
    }
}

impl Eq for ActionOrEvent {}
