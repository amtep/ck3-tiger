use crate::block::{Block, BlockOrValue, Comparator, Date, Token};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice, error, warn};
use crate::everything::Everything;
use crate::helpers::dup_assign_error;
use crate::item::Item;

#[derive(Debug)]
pub struct Validator<'a> {
    // The block being validated
    block: &'a Block,
    // A link to all the loaded and processed CK3 and mod files
    data: &'a Everything,
    // Fields that have been requested so far
    known_fields: Vec<&'a str>,
    // Whether loose tokens are expected
    accepted_tokens: bool,
    // Whether subblocks are expected
    accepted_blocks: bool,
    // Whether unknown keys are expected
    accepted_keys: bool,
}

impl<'a> Validator<'a> {
    pub fn new(block: &'a Block, data: &'a Everything) -> Self {
        Validator {
            block,
            data,
            known_fields: Vec::new(),
            accepted_tokens: false,
            accepted_blocks: false,
            accepted_keys: false,
        }
    }

    pub fn req_field(&mut self, name: &str) -> bool {
        if let Some(key) = self.block.get_key(name) {
            self.known_fields.push(key.as_str());
            true
        } else {
            error(
                self.block,
                ErrorKey::Validation,
                &format!("required field `{}` missing", name),
            );
            false
        }
    }

    pub fn field_check<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue),
    {
        let mut found = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    f(v);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field(&mut self, name: &str) -> Option<&BlockOrValue> {
        if self.field_check(name, |_| ()) {
            self.block.get_field(name)
        } else {
            None
        }
    }

    pub fn field_any_cmp(&mut self, name: &str) -> Option<&BlockOrValue> {
        let mut found = None;
        for (k, _, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some((other, _)) = found {
                        dup_assign_error(key, other);
                    }
                    found = Some((key, bv));
                }
            }
        }
        if let Some((_, bv)) = found {
            Some(bv)
        } else {
            None
        }
    }

    pub fn field_value(&mut self, name: &str) -> Option<&Token> {
        if self.field_check(name, |bv| _ = bv.expect_value()) {
            self.block.get_field_value(name)
        } else {
            None
        }
    }

    pub fn field_value_item(&mut self, name: &str, itype: Item) {
        self.field_check(name, |bv| {
            if let Some(token) = bv.expect_value() {
                self.data.verify_exists(itype, token);
            }
        });
    }

    pub fn field_block(&mut self, name: &str) -> Option<&Block> {
        if self.field_check(name, |bv| _ = bv.expect_block()) {
            self.block.get_field_block(name)
        } else {
            None
        }
    }

    pub fn field_bool(&mut self, name: &str) -> bool {
        self.field_check(name, |v| match v {
            BlockOrValue::Token(t) if t.is("yes") || t.is("no") => (),
            BlockOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected yes or no");
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        })
    }

    pub fn field_integer(&mut self, name: &str) -> bool {
        self.field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                if t.as_str().parse::<i32>().is_err() {
                    error(t, ErrorKey::Validation, "expected integer");
                }
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        })
    }

    pub fn field_numeric(&mut self, name: &str) -> bool {
        self.field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                if t.as_str().parse::<f64>().is_err() {
                    error(t, ErrorKey::Validation, "expected number");
                }
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        })
    }

    pub fn field_script_value(&mut self, name: &str, sc: &mut ScopeContext) {
        self.field_check(name, |bv| {
            ScriptValue::validate_bv(bv, self.data, sc);
        });
    }

    pub fn field_choice(&mut self, name: &str, choices: &[&str]) -> bool {
        self.field_check(name, |v| match v {
            BlockOrValue::Token(t) => {
                if !choices.contains(&t.as_str()) {
                    let msg = format!("expected one of {}", choices.join(", "));
                    error(t, ErrorKey::Validation, &msg);
                }
            }
            BlockOrValue::Block(s) => {
                error(s, ErrorKey::Validation, "expected value, found block");
            }
        })
    }

    pub fn field_list(&mut self, name: &str) -> bool {
        self.field_check(name, |v| match v {
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
        })
    }

    pub fn field_values(&mut self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => vec.push(t),
                        BlockOrValue::Block(s) => {
                            error(s, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                }
            }
        }
        vec
    }

    pub fn field_values_items(&mut self, name: &str, itype: Item) {
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    if let Some(token) = bv.expect_value() {
                        self.data.verify_exists(itype, token);
                    }
                }
            }
        }
    }

    pub fn fields(&mut self, name: &str) -> bool {
        let mut found = false;
        for (k, cmp, _) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    found = true;
                }
            }
        }
        found
    }

    pub fn field_validated<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything),
    {
        let mut found = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    f(v, self.data);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_bv<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything),
    {
        let mut found = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    f(v, self.data);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_bvs<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything),
    {
        let mut found = false;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    f(v, self.data);
                    found = true;
                }
            }
        }
        found
    }

    pub fn field_validated_blocks<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = false;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
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
                        BlockOrValue::Block(s) => f(s, self.data),
                    }
                    found = true;
                }
            }
        }
        found
    }

    pub fn field_validated_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = None;
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
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
                        BlockOrValue::Block(s) => f(s, self.data),
                    }
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_blocks(&mut self, name: &str) -> bool {
        let mut found = false;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    bv.expect_block();
                    found = true;
                }
            }
        }
        found
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
            self.known_fields.push(key.as_str());
            advice(key, ErrorKey::Unneeded, msg);
        }
    }

    pub fn values(&mut self) -> Vec<&Token> {
        self.accepted_tokens = true;
        let mut vec = Vec::new();
        for (k, _, v) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Token(t) = v {
                    vec.push(t);
                }
            }
        }
        vec
    }

    pub fn blocks(&mut self) -> Vec<&Block> {
        self.accepted_blocks = true;
        let mut vec = Vec::new();
        for (k, _, v) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Block(b) = v {
                    vec.push(b);
                }
            }
        }
        vec
    }

    pub fn integer_blocks(&mut self) -> Vec<(&Token, &Block)> {
        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.as_str().parse::<i32>().is_ok() {
                    self.known_fields.push(key.as_str());
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
                        BlockOrValue::Block(s) => vec.push((key, s)),
                    }
                }
            }
        }
        vec
    }

    pub fn integer_values(&mut self) -> Vec<(&Token, &Token)> {
        let mut vec = Vec::new();
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if key.as_str().parse::<i32>().is_ok() {
                    self.known_fields.push(key.as_str());
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        BlockOrValue::Token(t) => vec.push((key, t)),
                        BlockOrValue::Block(b) => {
                            error(b, ErrorKey::Validation, "expected value, found block");
                        }
                    }
                }
            }
        }
        vec
    }

    pub fn validate_history_blocks<F>(&mut self, mut f: F)
    where
        F: FnMut(&Block, &Everything),
    {
        for (k, cmp, v) in &self.block.v {
            if let Some(key) = k {
                if Date::try_from(key).is_ok() {
                    self.known_fields.push(key.as_str());
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
                        BlockOrValue::Block(s) => f(s, self.data),
                    }
                }
            }
        }
    }

    pub fn warn_past_known(&mut self, name: &str, msg: &str) {
        let mut past_known = false;
        for (k, _, _) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    if past_known {
                        warn(key, ErrorKey::Validation, msg);
                    }
                } else if !self.known_fields.contains(&key.as_str()) {
                    past_known = true;
                }
            }
        }
    }

    pub fn unknown_keys(&mut self) -> Vec<(&Token, &BlockOrValue)> {
        self.accepted_keys = true;
        let mut vec = Vec::new();
        for (k, _, bv) in &self.block.v {
            if let Some(key) = k {
                if !self.known_fields.contains(&key.as_str()) {
                    vec.push((key, bv));
                }
            }
        }
        vec
    }

    // TODO: make this execute on drop, and provide another function
    // to tell it not to warn. This way, there's less risk of a function
    // just forgetting to call warn_remaining.
    pub fn warn_remaining(&self) {
        for (k, _, v) in &self.block.v {
            match k {
                Some(key) => {
                    if !self.accepted_keys && !self.known_fields.contains(&key.as_str()) {
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
