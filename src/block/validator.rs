use crate::block::{Block, BlockOrValue, Comparator};
use crate::errorkey::ErrorKey;
use crate::errors::{advice, error, warn};

#[derive(Debug)]
pub struct Validator<'a> {
    // The block being validated
    block: &'a Block,
    // Fields that have been requested so far
    known_fields: Vec<&'a str>,
    // Whether loose tokens are expected
    accepted_tokens: bool,
    // Whether subblocks are expected
    accepted_blocks: bool,
}

impl<'a> Validator<'a> {
    pub fn new(block: &'a Block) -> Self {
        Validator {
            block,
            known_fields: Vec::new(),
            accepted_tokens: false,
            accepted_blocks: false,
        }
    }

    pub fn req_field_value(&mut self, name: &'a str) {
        self.known_fields.push(name);

        let mut found = false;
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
                        BlockOrValue::Token(_) => (),
                        BlockOrValue::Block(s) => {
                            error(s, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                    found = true;
                }
            }
        }
        if !found {
            error(
                &self.block.loc,
                ErrorKey::Validation,
                &format!("required field `{}` missing", name),
            );
        }
    }

    pub fn opt_field_check<F>(&mut self, name: &'a str, mut f: F)
    where
        F: FnMut(BlockOrValue),
    {
        self.known_fields.push(name);

        let mut found = false;
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
                    f(v.clone());
                    found = true;
                }
            }
        }
    }

    pub fn opt_field(&mut self, name: &'a str) {
        self.opt_field_check(name, |_| ());
    }

    pub fn opt_field_value(&mut self, name: &'a str) {
        self.opt_field_check(name, |v| match v {
            BlockOrValue::Token(_) => (),
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        });
    }

    pub fn opt_field_block(&mut self, name: &'a str) {
        self.opt_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected block, found value");
            }
            BlockOrValue::Block(_) => (),
        });
    }

    pub fn opt_field_bool(&mut self, name: &'a str) {
        self.opt_field_check(name, |v| match v {
            BlockOrValue::Token(t) if t.is("yes") || t.is("no") => (),
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected yes or no");
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        });
    }

    pub fn opt_field_integer(&mut self, name: &'a str) {
        self.opt_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                if t.as_str().parse::<i32>().is_err() {
                    error(t, ErrorKey::Validation, "expected integer");
                }
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        });
    }

    pub fn opt_field_list(&mut self, name: &'a str) {
        self.opt_field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected block, found value");
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
            }
        });
    }

    pub fn opt_field_values(&mut self, name: &'a str) {
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
    }

    pub fn opt_field_validated_blocks<F>(&mut self, name: &'a str, f: F)
    where
        F: Fn(&Block),
    {
        self.known_fields.push(name);

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
                        BlockOrValue::Block(s) => f(s),
                    }
                }
            }
        }
    }

    pub fn opt_field_validated_block<F>(&mut self, name: &'a str, f: F)
    where
        F: Fn(&Block),
    {
        self.known_fields.push(name);
        let mut found = false;

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
                        BlockOrValue::Token(t) => {
                            error(t, ErrorKey::Validation, "expected block, found value");
                        }
                        BlockOrValue::Block(s) => f(s),
                    }
                    found = true;
                }
            }
        }
    }

    pub fn opt_field_blocks(&mut self, name: &'a str) {
        self.known_fields.push(name);

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
                        BlockOrValue::Block(_) => (),
                    }
                }
            }
        }
    }

    pub fn req_tokens_integers_exactly(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for (k, _, v) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Token(t) = v {
                    if t.as_str().parse::<i32>().is_ok() {
                        found += 1;
                    } else {
                        error(t, ErrorKey::Validation, "expected integer");
                    }
                }
            }
        }
        if found != expect {
            let msg = format!("expected {} integers", expect);
            error(self.block, ErrorKey::Validation, &msg);
        }
    }

    pub fn advice_field(&mut self, name: &str, msg: &str) {
        if let Some(key) = self.block.get_key(name) {
            advice(key, ErrorKey::Unneeded, msg);
        }
    }

    pub fn warn_remaining(&mut self) {
        for (k, _, v) in &self.block.v {
            match k {
                Some(key) => {
                    if !self.known_fields.contains(&key.as_str()) {
                        warn(
                            key,
                            ErrorKey::Validation,
                            &format!("unknown field `{}`", key),
                        );
                    }
                }
                None => match v {
                    BlockOrValue::Token(t) => {
                        if !self.accepted_tokens {
                            warn(
                                t,
                                ErrorKey::Validation,
                                "found loose value, expected only `key =`",
                            );
                        }
                    }
                    BlockOrValue::Block(s) => {
                        if !self.accepted_blocks {
                            warn(
                                s,
                                ErrorKey::Validation,
                                "found sub-block, expected only `key =`",
                            );
                        }
                    }
                },
            }
        }
    }
}