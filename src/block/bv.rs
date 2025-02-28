use crate::block::Block;
use crate::report::{ErrorKey, err};
use crate::token::Token;

/// `BV` is an component a `Field`, which represents keyed items in `Block`.
/// It is itself either a `Block` or a single-token `Value`.
#[derive(Clone, Debug)]
pub enum BV {
    Value(Token),
    Block(Block),
}

impl BV {
    pub fn get_block(&self) -> Option<&Block> {
        match self {
            BV::Value(_) => None,
            BV::Block(b) => Some(b),
        }
    }

    pub fn get_value(&self) -> Option<&Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => None,
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        match self {
            BV::Value(_) => {
                err(ErrorKey::Structure).msg("expected block, found value").loc(self).push();
                None
            }
            BV::Block(b) => Some(b),
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => {
                err(ErrorKey::Structure).msg("expected value, found block").loc(self).push();
                None
            }
        }
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn into_value(self) -> Option<Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => None,
        }
    }

    pub fn expect_into_value(self) -> Option<Token> {
        match self {
            BV::Value(t) => Some(t),
            BV::Block(_) => {
                err(ErrorKey::Structure).msg("expected value, found block").loc(self).push();
                None
            }
        }
    }

    pub fn into_block(self) -> Option<Block> {
        match self {
            BV::Value(_) => None,
            BV::Block(b) => Some(b),
        }
    }

    #[allow(dead_code)] // It's here for symmetry
    pub fn expect_into_block(self) -> Option<Block> {
        match self {
            BV::Value(_) => {
                err(ErrorKey::Structure).msg("expected block, found value").loc(self).push();
                None
            }
            BV::Block(b) => Some(b),
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            BV::Value(t1) => {
                if let Some(t2) = other.get_value() {
                    t1.is(t2.as_str())
                } else {
                    false
                }
            }
            BV::Block(b1) => {
                if let Some(b2) = other.get_block() {
                    b1.equivalent(b2)
                } else {
                    false
                }
            }
        }
    }
}
