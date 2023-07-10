use crate::block::{Block, Comparator, Eq::*, BV};
use crate::report::{error, ErrorKey};
use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Field(pub Token, pub Comparator, pub BV);

impl Field {
    pub fn into_key(self) -> Token {
        self.0
    }

    pub fn key(&self) -> &Token {
        &self.0
    }

    pub fn cmp(&self) -> Comparator {
        self.1
    }

    pub fn into_bv(self) -> BV {
        self.2
    }

    pub fn bv(&self) -> &BV {
        &self.2
    }

    pub fn is_eq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single))
    }

    pub fn is_eq_qeq(&self) -> bool {
        matches!(self.1, Comparator::Equals(Single | Question))
    }

    pub fn expect_eq(&self) -> bool {
        let Self(key, cmp, _) = self;
        if matches!(cmp, Comparator::Equals(Single)) {
            true
        } else {
            error(self, ErrorKey::Validation, &format!("expected `{key} =`, found `{cmp}`"));
            false
        }
    }

    pub fn describe(&self) -> &'static str {
        if self.is_eq_qeq() {
            match self.2 {
                BV::Value(_) => "assignment",
                BV::Block(_) => "definition",
            }
        } else {
            "comparison"
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2.equivalent(&other.2)
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                return Some((key, block))
            }
            _ => {
                let msg = format!("expected definition, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }

    pub fn get_into_definition(self) -> Option<(Token, Block)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Block(block)) => {
                Some((key, block))
            }
            _ => None,
        }
    }
    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        match self {
            Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                return Some((key, token))
            }
            _ => {
                let msg = format!("expected assignment, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
            }
        }
        None
    }
}
