use crate::block::{Block, BlockOrValue, Comparator, Token};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::validate::{Validate, ValidationError};

#[derive(Debug)]
pub struct Validator<'a> {
    // The block being validated
    block: &'a Block,
    // Identifier used for error messages
    id: &'a str,
    // Fields that have been requested so far
    known_fields: Vec<&'a str>,
    // Fatal error, if any
    pub err: Option<ValidationError>,
}

impl<'a> Validator<'a> {
    pub fn new(block: &'a Block, id: &'a str) -> Self {
        Validator {
            block,
            id,
            known_fields: Vec::new(),
            err: None,
        }
    }

    pub fn err(&mut self, e: ValidationError) {
        if self.err.is_none() {
            self.err = Some(e);
        }
    }

    pub fn require_unique_field_value(&mut self, name: &'a str) -> Result<Token, ValidationError> {
        self.known_fields.push(name);

        let mut found = false;
        let mut value = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if found {
                        warn(
                            key,
                            ErrorKey::Validation,
                            &format!("multiple definitions of `{}`, expected only one.", key),
                        );
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => value = Some(t.clone()),
                        BlockOrValue::Block(s) => {
                            self.err(ValidationError::RequiredFieldInvalid(name.to_string()));
                            error(s, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                    found = true;
                }
            }
        }
        if !found || value.is_none() {
            let err = ValidationError::RequiredFieldMissing(name.to_string());
            self.err(err.clone());
            error(
                &self.block.loc,
                ErrorKey::Validation,
                &format!("required field `{}` missing", name),
            );
            Err(err)
        } else {
            Ok(value.unwrap())
        }
    }

    pub fn allow_unique_field_check<F, T>(&mut self, name: &'a str, mut f: F) -> Option<T>
    where
        F: FnMut(BlockOrValue) -> Option<T>,
    {
        self.known_fields.push(name);

        let mut found = false;
        let mut value = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if found {
                        warn(
                            key,
                            ErrorKey::Validation,
                            &format!("multiple definitions of `{}`, expected only one.", key),
                        );
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    value = f(v.clone());
                    found = true;
                }
            }
        }
        value
    }

    pub fn allow_unique_field(&mut self, name: &'a str) -> Option<BlockOrValue> {
        self.allow_unique_field_check(name, Some)
    }

    pub fn allow_unique_field_value(&mut self, name: &'a str) -> Option<Token> {
        self.allow_unique_field_check(name, |v| match v {
            BlockOrValue::Token(t) => Some(t),
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
                None
            }
        })
    }

    pub fn allow_unique_field_block(&mut self, name: &'a str) -> Option<Block> {
        self.allow_unique_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected block, found value");
                None
            }
            BlockOrValue::Block(s) => Some(s),
        })
    }

    pub fn allow_unique_field_boolean(&mut self, name: &'a str) -> Option<bool> {
        self.allow_unique_field_check(name, |v| match v {
            BlockOrValue::Token(t) if t.is("yes") => Some(true),
            BlockOrValue::Token(t) if t.is("no") => Some(false),
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected yes or no");
                None
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
                None
            }
        })
    }

    pub fn allow_unique_field_integer(&mut self, name: &'a str) -> Option<i64> {
        self.allow_unique_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                if let Ok(i) = t.as_str().parse() {
                    Some(i)
                } else {
                    error(t, ErrorKey::Validation, "expected integer");
                    None
                }
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
                None
            }
        })
    }

    pub fn allow_unique_field_list(&mut self, name: &'a str) -> Option<Vec<Token>> {
        self.allow_unique_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected block, found value");
                None
            }
            BlockOrValue::Block(s) => {
                let mut vec = Vec::new();
                for (k, _, v) in &s.v {
                    if let Some(key) = k {
                        warn(
                            key,
                            ErrorKey::Validation,
                            &format!("found key `{}`, expected only values", key),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => vec.push(t.clone()),
                        BlockOrValue::Block(s) => {
                            error(s, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                }
                Some(vec)
            }
        })
    }

    pub fn allow_field_values(&mut self, name: &'a str) -> Vec<Token> {
        self.known_fields.push(name);

        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => vec.push(t.clone()),
                        BlockOrValue::Block(s) => {
                            error(s, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                }
            }
        }
        vec
    }

    pub fn allow_field_validated_blocks<V: Validate>(
        &mut self,
        name: &'a str,
    ) -> Result<Vec<V>, ValidationError> {
        self.known_fields.push(name);
        let id = format!("{}.{}", self.id, name);

        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => {
                            error(t, ErrorKey::Validation, "expected block, found value");
                        }
                        BlockOrValue::Block(s) => vec.push(V::from_block(s.clone(), &id)?),
                    }
                }
            }
        }
        Ok(vec)
    }

    pub fn allow_field_blocks(&mut self, name: &'a str) -> Vec<Block> {
        self.known_fields.push(name);

        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => {
                            error(t, ErrorKey::Validation, "expected block, found value");
                        }
                        BlockOrValue::Block(s) => vec.push(s.clone()),
                    }
                }
            }
        }
        vec
    }

    pub fn warn_unused_entries(&mut self) {
        for (k, _, v) in &self.block.v {
            match k {
                Some(key) => {
                    if !self.known_fields.contains(&key.as_str()) {
                        warn(
                            key,
                            ErrorKey::Validation,
                            &format!("unknown field `{}` for {}", key, self.id),
                        );
                    }
                }
                None => match v {
                    BlockOrValue::Token(t) => {
                        warn(
                            t,
                            ErrorKey::Validation,
                            "found loose value, expected only `key =`",
                        );
                    }
                    BlockOrValue::Block(s) => {
                        warn(
                            s,
                            ErrorKey::Validation,
                            "found sub-block, expected only `key =`",
                        );
                    }
                },
            }
        }
    }
}
