use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

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

pub struct UnknownComparatorError;

impl FromStr for Comparator {
    type Err = UnknownComparatorError;

    fn from_str(s: &str) -> Result<Self, UnknownComparatorError> {
        match s {
            "=" => Ok(Comparator::Equals(Single)),
            "==" => Ok(Comparator::Equals(Double)),
            "?=" => Ok(Comparator::Equals(Question)),
            "<" => Ok(Comparator::LessThan),
            ">" => Ok(Comparator::GreaterThan),
            "<=" => Ok(Comparator::AtMost),
            ">=" => Ok(Comparator::AtLeast),
            "!=" => Ok(Comparator::NotEquals),
            _ => Err(UnknownComparatorError),
        }
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
