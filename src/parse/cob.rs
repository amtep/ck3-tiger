use std::mem::take;

use crate::token::{Loc, Token};

/// Copy on boundary type used for when a token may cross multiple parts of the input.
#[derive(Clone, Debug)]
pub(crate) enum Cob {
    Uninit,
    Borrowed(&'static str, usize, usize, Loc),
    Owned(String, Loc),
}

impl Default for Cob {
    fn default() -> Self {
        Self::Uninit
    }
}

impl Cob {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub(crate) fn set(&mut self, str: &'static str, index: usize, loc: Loc) {
        *self = Self::Borrowed(str, index, index, loc);
    }

    /// **ASSERT**: the char must match the char starting at the end index of the borrowed string (if applicable).
    pub(crate) fn add_char(&mut self, c: char) {
        match *self {
            Self::Uninit => unreachable!(),
            Self::Borrowed(str, start, end, loc) if end == str.len() => {
                let mut string = str[start..].to_owned();
                string.push(c);
                *self = Self::Owned(string, loc);
            }
            Self::Borrowed(_str, _, ref mut end, _) => {
                // ASSERT: _str[*end..].starts_with(c)
                *end += c.len_utf8();
            }
            Self::Owned(ref mut string, _) => string.push(c),
        }
    }

    pub(crate) fn make_owned(&mut self) {
        match *self {
            Self::Uninit => unreachable!(),
            Self::Borrowed(str, start, end, loc) => {
                let string = str[start..end].to_owned();
                *self = Self::Owned(string, loc);
            }
            Self::Owned(_, _) => (),
        }
    }

    pub(crate) fn take_to_token(&mut self) -> Token {
        match take(self) {
            Cob::Uninit => unreachable!(),
            Cob::Borrowed(str, start, end, loc) => Token::from_static_str(&str[start..end], loc),
            Cob::Owned(string, loc) => Token::new(&string, loc),
        }
    }
}
