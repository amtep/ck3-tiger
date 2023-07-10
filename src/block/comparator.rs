use std::fmt::{Display, Error, Formatter};

use crate::token::Token;

use crate::block::comparator::Eq::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Comparator {
    /// =, ?=, ==,
    Equals(Eq),
    /// !=
    NotEquals,
    /// <
    LessThan,
    /// >
    GreaterThan,
    /// <=
    AtMost,
    /// >=
    AtLeast,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Eq {
    /// Notation: =
    /// Valid as an equality comparison operator, assignment operator and scope opener.
    Single,
    /// Notation: ==
    /// Only valid as an equality comparison operator.
    Double,
    /// Notation: ?=
    /// Valid as a conditional equality comparison operator and condition scope opener.
    Question,
}

impl Comparator {
    pub fn from_str(s: &str) -> Option<Self> {
        if s == "=" {
            Some(Comparator::Equals(Single))
        } else if s == "==" {
            Some(Comparator::Equals(Double))
        } else if s == "?=" {
            Some(Comparator::Equals(Question))
        } else if s == "<" {
            Some(Comparator::LessThan)
        } else if s == ">" {
            Some(Comparator::GreaterThan)
        } else if s == "<=" {
            Some(Comparator::AtMost)
        } else if s == ">=" {
            Some(Comparator::AtLeast)
        } else if s == "!=" {
            Some(Comparator::NotEquals)
        } else {
            None
        }
    }
    pub fn from_token(token: &Token) -> Option<Self> {
        Self::from_str(token.as_str())
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            Comparator::Equals(Single) => write!(f, "="),
            Comparator::Equals(Double) => write!(f, "=="),
            Comparator::Equals(Question) => write!(f, "?="),
            Comparator::LessThan => write!(f, "<"),
            Comparator::GreaterThan => write!(f, ">"),
            Comparator::AtMost => write!(f, "<="),
            Comparator::AtLeast => write!(f, ">="),
            Comparator::NotEquals => write!(f, "!="),
        }
    }
}
