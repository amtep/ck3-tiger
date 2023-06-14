use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use crate::block::{Block, BlockOrValue, Comparator, Date, Token};
use crate::context::ScopeContext;
use crate::data::scriptvalues::ScriptValue;
use crate::errorkey::ErrorKey;
use crate::errors::{advice, error, warn};
use crate::everything::Everything;
use crate::helpers::dup_assign_error;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::trigger::validate_target;

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
    // Whether key comparisons should be done case-sensitively
    case_sensitive: bool,
}

impl<'a> Debug for Validator<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("Validator")
            .field("block", &self.block)
            .field("known_fields", &self.known_fields)
            .field("accepted_tokens", &self.accepted_tokens)
            .field("accepted_blocks", &self.accepted_blocks)
            .field("accepted_keys", &self.accepted_keys)
            .field("case_sensitive", &self.case_sensitive)
            .finish()
    }
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
            case_sensitive: true,
        }
    }

    pub fn set_case_sensitive(&mut self, cs: bool) {
        self.case_sensitive = cs;
    }

    pub fn req_field(&mut self, name: &str) -> bool {
        let found = self.check_key(name);
        if !found {
            let msg = format!("required field `{name}` missing");
            error(self.block, ErrorKey::FieldMissing, &msg);
        }
        found
    }

    pub fn req_field_one_of(&mut self, names: &[&str]) -> bool {
        let mut count = 0;
        for name in names {
            if self.check_key(name) {
                count += 1;
            }
        }
        if count != 1 {
            let msg = format!("expected exactly 1 of {}", names.join(", "));
            let key = if count == 0 {
                ErrorKey::FieldMissing
            } else {
                ErrorKey::Validation
            };
            error(self.block, key, &msg);
        }
        count == 1
    }

    pub fn req_field_warn(&mut self, name: &str) -> bool {
        let found = self.check_key(name);
        if !found {
            let msg = format!("required field `{name}` missing");
            warn(self.block, ErrorKey::FieldMissing, &msg);
        }
        found
    }

    pub fn ban_field<F, S>(&mut self, name: &str, only_for: F)
    where
        F: Fn() -> S,
        S: Borrow<str> + Display,
    {
        self.fields_check(name, |key, _| {
            let msg = format!("`{name} = ` is only for {}", only_for());
            error(key, ErrorKey::Validation, &msg);
        });
    }

    pub fn replaced_field(&mut self, name: &str, replaced_by: &str) {
        self.fields_check(name, |key, _| {
            let msg = format!("`{name}` has been replaced by {replaced_by}");
            error(key, ErrorKey::Validation, &msg);
        });
    }

    fn check_key(&mut self, name: &str) -> bool {
        for (k, _, _) in &self.block.v {
            if let Some(key) = k {
                if (self.case_sensitive && key.is(name))
                    || (!self.case_sensitive && key.lowercase_is(name))
                {
                    self.known_fields.push(key.as_str());
                    return true;
                }
            }
        }
        false
    }

    fn field_check<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BlockOrValue),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if (self.case_sensitive && key.is(name))
                    || (!self.case_sensitive && key.lowercase_is(name))
                {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    f(key, bv);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn fields_check<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BlockOrValue),
    {
        let mut found = false;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if (self.case_sensitive && key.is(name))
                    || (!self.case_sensitive && key.lowercase_is(name))
                {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    f(key, bv);
                    found = true;
                }
            }
        }
        found
    }

    pub fn field(&mut self, name: &str) -> bool {
        self.field_check(name, |_, _| ())
    }

    pub fn field_any_cmp(&mut self, name: &str) -> Option<&BlockOrValue> {
        let mut found = None;
        for (k, _, bv) in &self.block.v {
            if let Some(key) = k {
                if (self.case_sensitive && key.is(name))
                    || (!self.case_sensitive && key.lowercase_is(name))
                {
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
        let mut found = None;
        let mut result = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if (self.case_sensitive && key.is(name))
                    || (!self.case_sensitive && key.lowercase_is(name))
                {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    if let Some(token) = bv.expect_value() {
                        result = Some(token);
                    }
                    found = Some(key);
                }
            }
        }
        result
    }

    pub fn field_validated_value<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Token, &Everything),
    {
        self.field_check(name, |k, bv| {
            if let Some(token) = bv.expect_value() {
                f(k, token, self.data)
            }
        })
    }

    pub fn field_item(&mut self, name: &str, itype: Item) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                self.data.verify_exists(itype, token);
            }
        })
    }

    pub fn field_target(&mut self, name: &str, sc: &mut ScopeContext, outscopes: Scopes) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                validate_target(token, self.data, sc, outscopes);
            }
        })
    }

    pub fn field_item_or_target(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        itype: Item,
        outscopes: Scopes,
    ) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !self.data.item_exists(itype, token.as_str()) {
                    validate_target(token, self.data, sc, outscopes);
                }
            }
        })
    }

    pub fn field_block(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| _ = bv.expect_block())
    }

    pub fn field_bool(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !token.is("yes") && !token.is("no") {
                    error(token, ErrorKey::Validation, "expected yes or no");
                }
            }
        })
    }

    pub fn field_integer(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                token.expect_integer();
            }
        })
    }

    pub fn field_numeric(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                token.expect_number();
            }
        })
    }

    pub fn field_date(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if Date::from_str(token.as_str()).is_err() {
                    warn(token, ErrorKey::Validation, "expected date value");
                }
            }
        })
    }

    pub fn field_script_value(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.field_check(name, |_, bv| {
            ScriptValue::validate_bv(bv, self.data, sc);
        })
    }

    pub fn field_script_value_rooted(&mut self, name: &str, scopes: Scopes) -> bool {
        self.field_check(name, |_, bv| {
            let mut sc = ScopeContext::new_root(scopes, self.block.get_key(name).unwrap().clone());
            ScriptValue::validate_bv(bv, self.data, &mut sc);
        })
    }

    pub fn field_script_value_or_flag(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.get_value() {
                validate_target(
                    token,
                    self.data,
                    sc,
                    Scopes::Value | Scopes::Bool | Scopes::Flag,
                );
            } else {
                ScriptValue::validate_bv(bv, self.data, sc);
            }
        })
    }

    pub fn fields_script_value(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.fields_check(name, |_, bv| {
            ScriptValue::validate_bv(bv, self.data, sc);
        })
    }

    pub fn field_choice(&mut self, name: &str, choices: &[&str]) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !choices.contains(&token.as_str()) {
                    let msg = format!("expected one of {}", choices.join(", "));
                    error(token, ErrorKey::Validation, &msg);
                }
            }
        })
    }

    pub fn field_validated_list<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Everything),
    {
        self.field_check(name, |_, bv| {
            if let Some(block) = bv.expect_block() {
                for (k, _, bv) in &block.v {
                    if let Some(key) = k {
                        let msg = format!("found key `{key}`, expected only values");
                        warn(key, ErrorKey::Validation, &msg);
                    } else if let Some(token) = bv.expect_value() {
                        f(token, self.data);
                    }
                }
            }
        })
    }

    pub fn field_list(&mut self, name: &str) -> bool {
        self.field_validated_list(name, |_, _| ())
    }

    pub fn field_list_items(&mut self, name: &str, item: Item) -> bool {
        self.field_validated_list(name, |token, data| {
            data.verify_exists(item, token);
        })
    }

    pub fn fields_validated_list<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Everything),
    {
        self.fields_check(name, |_, bv| {
            if let Some(block) = bv.expect_block() {
                for (k, _, bv) in &block.v {
                    if let Some(key) = k {
                        let msg = format!("found key `{key}`, expected only values");
                        warn(key, ErrorKey::Validation, &msg);
                    } else if let Some(token) = bv.expect_value() {
                        f(token, self.data);
                    }
                }
            }
        })
    }

    pub fn fields_list(&mut self, name: &str) -> bool {
        self.fields_validated_list(name, |_, _| ())
    }

    pub fn fields_list_items(&mut self, name: &str, item: Item) -> bool {
        self.fields_validated_list(name, |token, data| {
            data.verify_exists(item, token);
        })
    }

    pub fn field_values(&mut self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(token) = bv.expect_value() {
                        vec.push(token);
                    }
                }
            }
        }
        vec
    }

    pub fn field_items(&mut self, name: &str, itype: Item) {
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(token) = bv.expect_value() {
                        self.data.verify_exists(itype, token);
                    }
                }
            }
        }
    }

    pub fn fields_any_cmp(&mut self, name: &str) -> bool {
        let mut found = false;
        for (k, _cmp, _) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
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
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    f(bv, self.data);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_key<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BlockOrValue, &Everything),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    f(key, bv, self.data);
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_sc<F>(&mut self, name: &str, sc: &mut ScopeContext, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything, &mut ScopeContext),
    {
        self.field_validated(name, |bv, data| f(bv, data, sc))
    }

    pub fn field_validated_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything, &mut ScopeContext),
    {
        self.field_validated_key(name, |key, bv, data| {
            let mut sc = ScopeContext::new_root(scopes, key);
            f(bv, data, &mut sc);
        })
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
                    expect_eq_qeq(key, cmp);
                    f(v, self.data);
                    found = true;
                }
            }
        }
        found
    }

    pub fn field_validated_bvs_sc<F>(&mut self, name: &str, sc: &mut ScopeContext, mut f: F) -> bool
    where
        F: FnMut(&BlockOrValue, &Everything, &mut ScopeContext),
    {
        self.field_validated_bvs(name, |b, data| f(b, data, sc))
    }

    pub fn field_validated_blocks<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = false;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        f(block, self.data);
                    }
                    found = true;
                }
            }
        }
        found
    }

    pub fn field_validated_blocks_sc<F>(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        self.field_validated_blocks(name, |b, data| f(b, data, sc))
    }

    pub fn field_validated_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        f(block, self.data);
                    }
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_key_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Block, &Everything),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        f(key, block, self.data);
                    }
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_block_sc<F>(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        self.field_validated_block(name, |b, data| f(b, data, sc))
    }

    pub fn field_validated_block_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        let mut sc = ScopeContext::new_root(scopes, key);
                        f(block, self.data, &mut sc);
                    }
                    found = Some(key);
                }
            }
        }
        found.is_some()
    }

    pub fn field_validated_blocks_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F)
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        let mut sc = ScopeContext::new_root(scopes, key);
                        f(block, self.data, &mut sc);
                    }
                }
            }
        }
    }

    pub fn field_validated_block_rerooted<F>(
        &mut self,
        name: &str,
        sc: &ScopeContext,
        scopes: Scopes,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        let mut found = None;
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is(name) {
                    self.known_fields.push(key.as_str());
                    if let Some(other) = found {
                        dup_assign_error(key, other);
                    }
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        let mut sc = sc.clone();
                        sc.change_root(scopes, key);
                        f(block, self.data, &mut sc);
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
                    expect_eq_qeq(key, cmp);
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
        for (k, _, bv) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Value(t) = bv {
                    if let Some(_) = t.expect_integer() {
                        found += 1;
                    }
                }
            }
        }
        if found != expect {
            let msg = format!("expected {expect} integers");
            error(self.block, ErrorKey::Validation, &msg);
        }
    }

    pub fn req_tokens_numbers_exactly(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for (k, _, bv) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Value(t) = bv {
                    if let Some(_) = t.expect_number() {
                        found += 1;
                    }
                }
            }
        }
        if found != expect {
            let msg = format!("expected {expect} numbers");
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
        for (k, _, bv) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Value(t) = bv {
                    vec.push(t);
                }
            }
        }
        vec
    }

    pub fn blocks(&mut self) -> Vec<&Block> {
        self.accepted_blocks = true;
        let mut vec = Vec::new();
        for (k, _, bv) in &self.block.v {
            if k.is_none() {
                if let BlockOrValue::Block(b) = bv {
                    vec.push(b);
                }
            }
        }
        vec
    }

    pub fn integer_blocks(&mut self) -> Vec<(&Token, &Block)> {
        let mut vec = Vec::new();
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is_integer() {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        vec.push((key, block));
                    }
                }
            }
        }
        vec
    }

    pub fn integer_values(&mut self) -> Vec<(&Token, &Token)> {
        let mut vec = Vec::new();
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if key.is_integer() {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(token) = bv.expect_value() {
                        vec.push((key, token));
                    }
                }
            }
        }
        vec
    }

    pub fn validate_history_blocks<F>(&mut self, mut f: F)
    where
        F: FnMut(Date, &Block, &Everything),
    {
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if let Ok(date) = Date::try_from(key) {
                    self.known_fields.push(key.as_str());
                    expect_eq_qeq(key, cmp);
                    if let Some(block) = bv.expect_block() {
                        f(date, block, self.data);
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
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                expect_eq_qeq(key, cmp);
                if !self.known_fields.contains(&key.as_str()) {
                    vec.push((key, bv));
                }
            }
        }
        vec
    }

    pub fn unknown_keys_any_cmp(&mut self) -> Vec<(&Token, Comparator, &BlockOrValue)> {
        self.accepted_keys = true;
        let mut vec = Vec::new();
        for (k, cmp, bv) in &self.block.v {
            if let Some(key) = k {
                if !self.known_fields.contains(&key.as_str()) {
                    vec.push((key, *cmp, bv));
                }
            }
        }
        vec
    }

    pub fn no_warn_remaining(&mut self) {
        self.accepted_keys = true;
        self.accepted_tokens = true;
        self.accepted_blocks = true;
    }

    pub fn warn_remaining(&mut self) -> bool {
        let mut warned = false;
        for (k, _, v) in &self.block.v {
            match k {
                Some(key) => {
                    if !self.accepted_keys && !self.known_fields.contains(&key.as_str()) {
                        warn(key, ErrorKey::Validation, &format!("unknown field `{key}`"));
                        warned = true;
                    }
                }
                None => match v {
                    BlockOrValue::Value(t) => {
                        if !self.accepted_tokens {
                            warn(
                                t,
                                ErrorKey::Validation,
                                "found loose value, expected only `key =`",
                            );
                            warned = true;
                        }
                    }
                    BlockOrValue::Block(s) => {
                        if !self.accepted_blocks {
                            warn(
                                s,
                                ErrorKey::Validation,
                                "found sub-block, expected only `key =`",
                            );
                            warned = true;
                        }
                    }
                },
            }
        }
        self.no_warn_remaining();
        warned
    }
}

impl<'a> Drop for Validator<'a> {
    fn drop(&mut self) {
        self.warn_remaining();
    }
}

fn expect_eq_qeq(key: &Token, cmp: &Comparator) {
    if !matches!(cmp, Comparator::Eq | Comparator::QEq) {
        error(
            key,
            ErrorKey::Validation,
            &format!("expected `{key} =` or `?=`, found `{cmp}`"),
        );
    }
}
