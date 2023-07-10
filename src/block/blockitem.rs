use crate::block::{Block, Comparator, Eq::*, Field, BV};
use crate::report::{error, error_info, ErrorKey};
use crate::token::Token;

#[derive(Debug, Clone)]
pub enum BlockItem {
    Value(Token),
    Block(Block),
    Field(Field),
}

impl BlockItem {
    pub fn expect_field(&self) -> Option<&Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => {
                let msg = format!("unexpected {}", self.describe());
                error_info(self, ErrorKey::Validation, &msg, "Did you forget an = ?");
                None
            }
        }
    }

    pub fn expect_into_field(self) -> Option<Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => {
                let msg = format!("unexpected {}", self.describe());
                error_info(self, ErrorKey::Validation, &msg, "Did you forget an = ?");
                None
            }
        }
    }

    pub fn get_field(&self) -> Option<&Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn get_into_field(self) -> Option<Field> {
        match self {
            BlockItem::Field(field) => Some(field),
            _ => None,
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, BlockItem::Field(_))
    }

    pub fn get_value(&self) -> Option<&Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => None,
        }
    }

    pub fn expect_value(&self) -> Option<&Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => {
                let msg = format!("expected value, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn expect_into_value(self) -> Option<Token> {
        match self {
            BlockItem::Value(token) => Some(token),
            _ => {
                let msg = format!("expected value, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn get_block(&self) -> Option<&Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn expect_block(&self) -> Option<&Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => {
                let msg = format!("expected block, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn expect_into_block(self) -> Option<Block> {
        match self {
            BlockItem::Block(block) => Some(block),
            _ => {
                let msg = format!("expected block, found {}", self.describe());
                error(self, ErrorKey::Validation, &msg);
                None
            }
        }
    }

    pub fn get_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.get_field() {
            field.get_definition()
        } else {
            None
        }
    }

    pub fn expect_into_definition(self) -> Option<(Token, Block)> {
        if let Some(field) = self.expect_into_field() {
            field.expect_into_definition()
        } else {
            None
        }
    }

    pub fn expect_definition(&self) -> Option<(&Token, &Block)> {
        if let Some(field) = self.expect_field() {
            field.expect_definition()
        } else {
            None
        }
    }

    pub fn get_into_definition(self) -> Option<(Token, Block)> {
        if let Some(field) = self.get_into_field() {
            field.get_into_definition()
        } else {
            None
        }
    }

    pub fn expect_assignment(&self) -> Option<(&Token, &Token)> {
        if let Some(field) = self.expect_field() {
            match field {
                Field(key, Comparator::Equals(Single | Question), BV::Value(token)) => {
                    return Some((key, token))
                }
                _ => {
                    let msg = format!("expected assignment, found {}", field.describe());
                    error(self, ErrorKey::Validation, &msg);
                }
            }
        }
        None
    }

    pub fn get_assignment(&self) -> Option<(&Token, &Token)> {
        match self {
            BlockItem::Field(Field(
                key,
                Comparator::Equals(Single | Question),
                BV::Value(token),
            )) => Some((key, token)),
            _ => None,
        }
    }

    pub fn describe(&self) -> &'static str {
        match self {
            BlockItem::Value(_) => "value",
            BlockItem::Block(_) => "block",
            BlockItem::Field(field) => field.describe(),
        }
    }

    pub fn equivalent(&self, other: &Self) -> bool {
        match self {
            BlockItem::Value(token) => {
                if let BlockItem::Value(t) = other {
                    token == t
                } else {
                    false
                }
            }
            BlockItem::Block(block) => {
                if let BlockItem::Block(b) = other {
                    block.equivalent(b)
                } else {
                    false
                }
            }
            BlockItem::Field(field) => {
                if let BlockItem::Field(f) = other {
                    field.equivalent(f)
                } else {
                    false
                }
            }
        }
    }
}
