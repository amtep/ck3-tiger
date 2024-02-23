use std::borrow::Borrow;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use crate::block::{Block, BlockItem, Comparator, Eq::*, Field, BV};
use crate::context::ScopeContext;
use crate::date::Date;
use crate::everything::Everything;
use crate::helpers::dup_assign_error;
use crate::item::Item;
#[cfg(feature = "ck3")]
use crate::report::fatal;
use crate::report::{report, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
#[cfg(any(feature = "ck3", feature = "imperator"))]
use crate::script_value::validate_script_value_no_breakdown;
use crate::token::Token;
use crate::trigger::{validate_target, validate_target_ok_this};

pub use self::value_validator::ValueValidator;

mod value_validator;

/// A validator for one `Block`.
/// The intended usage is that you wrap the `Block` in a validator, then call validation functions on it
/// until you've validated all the possible legitimate contents of the `Block`, and then the `Validator`
/// will warn the user about anything left over when it goes out of scope. This way you don't have to worry
/// about checking for unknown fields yourself.
///
/// The validator is mostly for checking "fields" (`key = value` and `key = { block }` items in the block),
/// but it can validate loose blocks and loose values and comparisons as well.
pub struct Validator<'a> {
    /// The block being validated
    block: &'a Block,
    /// A link to all the loaded and processed CK3 and mod files
    data: &'a Everything,
    /// Fields that have been requested so far
    known_fields: Vec<&'a str>,
    /// Whether loose tokens are expected
    accepted_tokens: bool,
    /// Whether subblocks are expected
    accepted_blocks: bool,
    /// Whether unknown block fields are expected
    accepted_block_fields: bool,
    /// Whether unknown value fields are expected
    accepted_value_fields: bool,
    /// Whether key comparisons should be done case-sensitively
    case_sensitive: bool,
    /// Whether this block can have ?= operators
    allow_questionmark_equals: bool,
    /// Maximum severity of problems reported by this `Validator`. Defaults to `Error`.
    /// This is intended to be set lower by validators for less-important items.
    /// As an exception, `Fatal` severity reports will still always be logged as `Fatal`.
    /// TODO: pass this down to all the helper functions
    max_severity: Severity,
}

impl<'a> Debug for Validator<'a> {
    /// Roll our own `Debug` implementation in order to leave out the `data` field.
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("Validator")
            .field("block", &self.block)
            .field("known_fields", &self.known_fields)
            .field("accepted_tokens", &self.accepted_tokens)
            .field("accepted_blocks", &self.accepted_blocks)
            .field("accepted_block_fields", &self.accepted_block_fields)
            .field("accepted_value_fields", &self.accepted_value_fields)
            .field("case_sensitive", &self.case_sensitive)
            .field("allow_questionmark_equals", &self.allow_questionmark_equals)
            .field("max_severity", &self.max_severity)
            .finish()
    }
}

impl<'a> Validator<'a> {
    /// Construct a new `Validator` for a [`Block`]. The `data` reference is there to help out some of the convenience
    /// functions, and also to pass along to closures so that you can easily pass independent functions as the closures.
    pub fn new(block: &'a Block, data: &'a Everything) -> Self {
        Validator {
            block,
            data,
            known_fields: Vec::new(),
            accepted_tokens: false,
            accepted_blocks: false,
            accepted_block_fields: false,
            accepted_value_fields: false,
            case_sensitive: true,
            allow_questionmark_equals: false,
            max_severity: Severity::Fatal,
        }
    }

    /// Control whether the fields in this `Block` will be matched case-sensitively or not.
    /// Whether this should be on or off depends on what the game engine allows, which is not always known.
    pub fn set_case_sensitive(&mut self, cs: bool) {
        self.case_sensitive = cs;
    }

    /// Whether this block can contain `?=` as well as `=` for assignments and definitions.
    /// Blocks that allow `?=` are mostly specialized ones such as triggers and effects.
    pub fn set_allow_questionmark_equals(&mut self, allow_questionmark_equals: bool) {
        self.allow_questionmark_equals = allow_questionmark_equals;
    }

    pub fn set_max_severity(&mut self, max_severity: Severity) {
        self.max_severity = max_severity;
    }

    /// Require field `name` to be present in the block, and warn if it isn't there.
    /// Returns true iff the field is present.
    pub fn req_field(&mut self, name: &str) -> bool {
        let found = self.check_key(name);
        if !found {
            let msg = format!("required field `{name}` missing");
            let sev = Severity::Error.at_most(self.max_severity);
            report(ErrorKey::FieldMissing, sev).msg(msg).loc(self.block).push();
        }
        found
    }

    /// Require exactly one of the fields in `names` to be present in the block,
    /// and warn if they are missing or there is more than one.
    /// Returns true iff it found exactly one.
    pub fn req_field_one_of(&mut self, names: &[&str]) -> bool {
        let mut count = 0;
        for name in names {
            if self.check_key(name) {
                count += 1;
            }
        }
        if count != 1 {
            let msg = format!("expected exactly 1 of {}", names.join(", "));
            let key = if count == 0 { ErrorKey::FieldMissing } else { ErrorKey::Validation };
            let sev = Severity::Error.at_most(self.max_severity);
            report(key, sev).msg(msg).loc(self.block).push();
        }
        count == 1
    }

    /// Require field `name` to be present in the block, and warn if it isn't there.
    /// Returns true iff the field is present. Warns at a lower severity than `req_field`.
    pub fn req_field_warn(&mut self, name: &str) -> bool {
        let found = self.check_key(name);
        if !found {
            let msg = format!("required field `{name}` missing");
            let sev = Severity::Warning.at_most(self.max_severity);
            report(ErrorKey::FieldMissing, sev).msg(msg).loc(self.block).push();
        }
        found
    }

    /// Require field `name` to be present in the block, and warn if it isn't there.
    /// Returns true iff the field is present. Warns at [`Severity::Fatal`] level.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn req_field_fatal(&mut self, name: &str) -> bool {
        let found = self.check_key(name);
        if !found {
            let msg = format!("required field `{name}` missing");
            fatal(ErrorKey::FieldMissing).msg(msg).loc(self.block).push();
        }
        found
    }

    /// Require field `name` to not be in the block, and warn if it is found.
    /// The warning will include the output from the `only_for` closure,
    /// which describes where the field *is* expected.
    /// TODO: make lower-severity versions of this function.
    pub fn ban_field<F, S>(&mut self, name: &str, only_for: F)
    where
        F: Fn() -> S,
        S: Borrow<str> + Display,
    {
        let sev = Severity::Error.at_most(self.max_severity);
        self.multi_field_check(name, |key, _| {
            let msg = format!("`{name} = ` is only for {}", only_for());
            report(ErrorKey::Validation, sev).msg(msg).loc(key).push();
        });
    }

    /// Require field `name` to not be in the block. If it is found, warn that it has been replaced by `replaced_by`.
    /// This is used to adapt to and warn about changes in the game engine.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn replaced_field(&mut self, name: &str, replaced_by: &str) {
        let sev = Severity::Error.at_most(self.max_severity);
        self.multi_field_check(name, |key, _| {
            let msg = format!("`{name}` has been replaced by {replaced_by}");
            report(ErrorKey::Validation, sev).msg(msg).loc(key).push();
        });
    }

    fn check_key(&mut self, name: &str) -> bool {
        for Field(key, _, _) in self.block.iter_fields() {
            if (self.case_sensitive && key.is(name))
                || (!self.case_sensitive && key.lowercase_is(name))
            {
                self.known_fields.push(key.as_str());
                return true;
            }
        }
        false
    }

    fn field_check<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BV),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if (self.case_sensitive && key.is(name))
                || (!self.case_sensitive && key.lowercase_is(name))
            {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                f(key, bv);
                found = Some(key);
            }
        }
        found.is_some()
    }

    fn multi_field_check<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BV),
    {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if (self.case_sensitive && key.is(name))
                || (!self.case_sensitive && key.lowercase_is(name))
            {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                f(key, bv);
                found = true;
            }
        }
        found
    }

    /// Expect field `name`, if present, to be either an assignment (`= value`) or a definition (`= { block }`).
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field(&mut self, name: &str) -> bool {
        self.field_check(name, |_, _| ())
    }

    /// Just like [`Validator::field`], but expects any number of `name` fields in the block.
    pub fn multi_field(&mut self, name: &str) -> bool {
        self.multi_field_check(name, |_, _| ())
    }

    /// Expect field `name`, if present, to be... present.
    /// Expect no more than one `name` field in the block.
    /// Returns the field's `BV` (block or value) if the field is present.
    /// TODO: replace this with a `field_validated` variant.
    pub fn field_any_cmp(&mut self, name: &str) -> Option<&BV> {
        let mut found = None;
        for Field(key, _, bv) in self.block.iter_fields() {
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
        if let Some((_, bv)) = found {
            Some(bv)
        } else {
            None
        }
    }

    /// Expect field `name`, if present, to be an assignment (`name = value`).
    /// Expect no more than one `name` field in the block.
    /// Returns the field's value if the field is present.
    pub fn field_value(&mut self, name: &str) -> Option<&Token> {
        let mut found = None;
        let mut result = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if (self.case_sensitive && key.is(name))
                || (!self.case_sensitive && key.lowercase_is(name))
            {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                if let Some(token) = bv.expect_value() {
                    result = Some(token);
                }
                found = Some(key);
            }
        }
        result
    }

    /// Expect field `name`, if present, to be an assignment (`name = value`).
    /// Expect no more than one `name` field in the block.
    /// Runs the validation closure `f(key, vd)` for every matching field.
    /// Returns true iff the field is present.
    pub fn field_validated_value<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, ValueValidator),
    {
        let max_sev = self.max_severity;
        self.field_check(name, |k, bv| {
            if let Some(value) = bv.expect_value() {
                let mut vd = ValueValidator::new(value, self.data);
                vd.set_max_severity(max_sev);
                f(k, vd);
            }
        })
    }

    /// Just like [`Validator::field_validated_value`], but expect any number of `name` fields in the block.
    pub fn multi_field_validated_value<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, ValueValidator),
    {
        let max_sev = self.max_severity;
        self.multi_field_check(name, |k, bv| {
            if let Some(value) = bv.expect_value() {
                let mut vd = ValueValidator::new(value, self.data);
                vd.set_max_severity(max_sev);
                f(k, vd);
            }
        })
    }

    /// Expect field `name`, if present, to be set to the key of an `itype` item the game database.
    /// The item is looked up and must exist.
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_item(&mut self, name: &str, itype: Item) -> bool {
        let sev = self.max_severity;
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                self.data.verify_exists_max_sev(itype, token, sev);
            }
        })
    }

    /// Expect field `name`, if present, to be an assignment where the value evaluates to a scope type in `outscopes`.
    ///
    /// The value is evaluated in the scope context `sc`, so for example if the value does `scope:actor` but there is
    /// no named scope "actor" in the scope context, then a warning is emitted.
    /// Also emits a warning if the value is simply "`this`", because that is almost never correct.
    ///
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_target(&mut self, name: &str, sc: &mut ScopeContext, outscopes: Scopes) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                // TODO: pass max_severity here
                validate_target(token, self.data, sc, outscopes);
            }
        })
    }

    /// Just like [`Validator::field_target`], but allows the value to be simply "`this`".
    /// It is expected to be used judiciously in cases where "`this`" can be correct.
    pub fn field_target_ok_this(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        outscopes: Scopes,
    ) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                // TODO: pass max_severity here
                validate_target_ok_this(token, self.data, sc, outscopes);
            }
        })
    }

    /// This is a combination of [`Validator::field_item`] and [`Validator::field_target`]. If the field is present
    /// and is not a known `itype` item, then it is evaluated as a target.
    /// Returns true iff the field is present.
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
                    // TODO: pass max_severity here
                    validate_target(token, self.data, sc, outscopes);
                }
            }
        })
    }

    /// This is a combination of [`Validator::field_integer`] and [`Validator::field_target`].
    /// Returns true if the field is present.
    #[cfg(feature = "imperator")] // other games don't use, silence dead code warning.
    pub fn field_target_or_integer(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        outscopes: Scopes,
    ) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if token.expect_integer().is_some() {
                    // TODO: pass max_severity here
                    validate_target(token, self.data, sc, outscopes);
                }
            }
        })
    }

    /// Expect field `name`, if present, to be a definition `name = { block }`.
    /// Expect no more than one `name` field.
    /// No other validation is done.
    /// Returns true iff the field is present.
    #[cfg(any(feature = "ck3", feature = "imperator"))] // vic3 happens not to use; silence dead code warning
    pub fn field_block(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| _ = bv.expect_block())
    }

    /// Expect field `name`, if present, to be `name = yes` or `name = no`.
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_bool(&mut self, name: &str) -> bool {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !token.is("yes") && !token.is("no") && !token.is("YES") && !token.is("NO") {
                    report(ErrorKey::Validation, sev).msg("expected yes or no").loc(token).push();
                }
            }
        })
    }

    /// Expect field `name`, if present, to be set to an integer.
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_integer(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                // TODO: pass max_severity here
                token.expect_integer();
            }
        })
    }

    /// Expect field `name`, if present, to be set to an integer between `low` and `high` (inclusive).
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_integer_range(&mut self, name: &str, low: i64, high: i64) {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                // TODO: pass max_severity here
                if let Some(i) = token.expect_integer() {
                    if !(low..=high).contains(&i) {
                        let msg = format!("should be between {low} and {high} (inclusive)");
                        report(ErrorKey::Range, sev).msg(msg).loc(token).push();
                    }
                }
            }
        });
    }

    /// Expect field `name`, if present, to be set to an integer at least `min`
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    #[cfg(feature = "vic3")]
    pub fn field_integer_min(&mut self, name: &str, min: i64) {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                // TODO: pass max_severity here
                if let Some(i) = token.expect_integer() {
                    if i < min {
                        let msg = format!("should be at least {min}");
                        report(ErrorKey::Range, sev).msg(msg).loc(token).push();
                    }
                }
            }
        });
    }

    /// Expect field `name`, if present, to be set to a number with up to 5 decimals.
    /// (5 decimals is the limit accepted by the game engine in most contexts).
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_numeric(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                token.expect_number();
            }
        })
    }

    /// Expect field `name`, if present, to be set to a number with any number of decimals.
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_precise_numeric(&mut self, name: &str) -> bool {
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                token.expect_precise_number();
            }
        })
    }

    /// Expect field `name`, if present, to be set to a number between `low` and `high` (inclusive).
    /// Accept at most 5 decimals. (5 decimals is the limit accepted by the game engine in most contexts).
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_numeric_range(&mut self, name: &str, low: f64, high: f64) {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if let Some(f) = token.expect_number() {
                    if !(low..=high).contains(&f) {
                        let msg = format!("should be between {low} and {high} (inclusive)");
                        report(ErrorKey::Range, sev).msg(msg).loc(token).push();
                    }
                }
            }
        });
    }

    /// Expect field `name`, if present, to be set to a date.
    /// The format of dates is very flexible, from a single number (the year), to a year.month or year.month.day.
    /// No checking is done on the validity of the date as a date (so January 42nd is okay).
    /// Expect no more than one `name` field.
    /// Returns true iff the field is present.
    pub fn field_date(&mut self, name: &str) -> bool {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if Date::from_str(token.as_str()).is_err() {
                    let msg = "expected date value";
                    report(ErrorKey::Validation, sev).msg(msg).loc(token).push();
                }
            }
        })
    }

    /// Expect field `name`, if present, to be set to a script value, either a named one (simply `name = scriptvaluename`)
    /// or an inline one (can be a simple number, or a range `{ min max }`, or a full script value definition with limits
    /// and math).
    ///
    /// The script value is evaluated in the scope context `sc`, so for example if the script value does `scope:actor` but
    /// there is no named scope "actor" in the scope context, then a warning is emitted.
    ///
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_script_value(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.field_check(name, |_, bv| {
            // TODO: pass max_severity value down
            validate_script_value(bv, self.data, sc);
        })
    }

    /// Just like [`Validator::field_script_value`], but does not warn if it is an inline script value and the `desc` fields
    /// in it do not contain valid localizations. This is generally used for script values that will never be shown to
    /// the user except in debugging contexts, such as `ai_will_do`.
    #[cfg(any(feature = "ck3", feature = "imperator"))] // vic3 happens not to use; silence dead code warning
    pub fn field_script_value_no_breakdown(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.field_check(name, |_, bv| {
            // TODO: pass max_severity value down
            validate_script_value_no_breakdown(bv, self.data, sc);
        })
    }

    /// Just like [`Validator::field_script_value`], but instead of a full [`ScopeContext`] it just gets the scope type
    /// to be used for the `root` of a `ScopeContext` that is made on the spot. This is a convenient way to associate the
    /// `root` type with the key of this field, for clearer warnings. A passed-in `ScopeContext` would have to be associated
    /// with a key that is further away.
    pub fn field_script_value_rooted(&mut self, name: &str, scopes: Scopes) -> bool {
        self.field_check(name, |key, bv| {
            let mut sc = ScopeContext::new(scopes, key);
            // TODO: pass max_severity value down
            validate_script_value(bv, self.data, &mut sc);
        })
    }

    /// Just like [`Validator::field_script_value`], but it takes a closure that uses the field key token
    /// as the input to build and output a [`ScopeContext`]. This is a convenient way to associate the `root` type with the key
    /// of this field, for clearer warnings. A passed-in `ScopeContext` would have to be associated with a key that is further away.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn field_script_value_key<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token) -> ScopeContext,
    {
        self.field_check(name, |key, bv| {
            let mut sc = f(key);
            // TODO: pass max_severity value down
            validate_script_value(bv, self.data, &mut sc);
        })
    }

    /// Just like [`Validator::field_script_value`], but it can accept a literal `flag:something` value as well as a script value.
    pub fn field_script_value_or_flag(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.field_check(name, |_, bv| {
            // TODO: pass max_severity value down
            if let Some(token) = bv.get_value() {
                validate_target(token, self.data, sc, Scopes::Value | Scopes::Bool | Scopes::Flag);
            } else {
                validate_script_value(bv, self.data, sc);
            }
        })
    }

    /// Just like [`Validator::field_script_value`], but it it expects any number of `name` fields.
    pub fn fields_script_value(&mut self, name: &str, sc: &mut ScopeContext) -> bool {
        self.multi_field_check(name, |_, bv| {
            // TODO: pass max_severity value down
            validate_script_value(bv, self.data, sc);
        })
    }

    /// Expect field `name`, if present, to be set to one of the listed strings in `choices`.
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_choice(&mut self, name: &str, choices: &[&str]) -> bool {
        let sev = Severity::Error.at_most(self.max_severity);
        self.field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !choices.contains(&token.as_str()) {
                    let msg = format!("expected one of {}", choices.join(", "));
                    report(ErrorKey::Choice, sev).msg(msg).loc(token).push();
                }
            }
        })
    }

    /// Just like [`Validator::field_choice`], but expect any number of `name` fields in the block.
    #[allow(dead_code)] // not currently used
    pub fn multi_field_choice(&mut self, name: &str, choices: &[&str]) -> bool {
        let sev = Severity::Error.at_most(self.max_severity);
        self.multi_field_check(name, |_, bv| {
            if let Some(token) = bv.expect_value() {
                if !choices.contains(&token.as_str()) {
                    let msg = format!("expected one of {}", choices.join(", "));
                    report(ErrorKey::Choice, sev).msg(msg).loc(token).push();
                }
            }
        })
    }

    /// Expect field `name`, if present, to be of the form `name = { value value value ... }` with any number of values.
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_list(&mut self, name: &str) -> bool {
        self.field_validated_list(name, |_, _| ())
    }

    /// Expect field `name`, if present, to be of the form `name = { value value value ... }` with any number of values.
    /// Expect no more than one `name` field in the block.
    /// Calls the closure `f(value, data)` for every value in the list.
    /// Returns true iff the field is present.
    pub fn field_validated_list<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Everything),
    {
        self.field_check(name, |_, bv| {
            if let Some(block) = bv.expect_block() {
                for token in block.iter_values_warn() {
                    f(token, self.data);
                }
            }
        })
    }

    /// Expect field `name`, if present, to be of the form `name = { value value value ... }` with any number of values.
    /// Expect every value to be an `itype` item in the game database.
    /// Expect no more than one `name` field in the block.
    /// Returns true iff the field is present.
    pub fn field_list_items(&mut self, name: &str, item: Item) -> bool {
        let sev = self.max_severity;
        self.field_validated_list(name, |token, data| {
            data.verify_exists_max_sev(item, token, sev);
        })
    }

    /// Just like [`Validator::field_validated_list`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_validated_list<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Everything),
    {
        self.multi_field_check(name, |_, bv| {
            if let Some(block) = bv.expect_block() {
                for token in block.iter_values_warn() {
                    f(token, self.data);
                }
            }
        })
    }

    /// Just like [`Validator::field_list_items`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_list_items(&mut self, name: &str, item: Item) -> bool {
        let sev = self.max_severity;
        self.multi_field_validated_list(name, |token, data| {
            data.verify_exists_max_sev(item, token, sev);
        })
    }

    /// Just like [`Validator::field_value`], but expect any number of `name` fields in the block.
    pub fn multi_field_value(&mut self, name: &str) -> Vec<&Token> {
        let mut vec = Vec::new();
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(token) = bv.expect_value() {
                    vec.push(token);
                }
            }
        }
        vec
    }

    /// Just like [`Validator::field_item`], but expect any number of `name` fields in the block.
    pub fn multi_field_item(&mut self, name: &str, itype: Item) {
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(token) = bv.expect_value() {
                    self.data.verify_exists_max_sev(itype, token, self.max_severity);
                }
            }
        }
    }

    /// Just like [`Validator::field_any_cmp`], but expect any number of `name` fields in the block.
    pub fn multi_field_any_cmp(&mut self, name: &str) -> bool {
        let mut found = false;
        for Field(key, _, _) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                found = true;
            }
        }
        found
    }

    /// Expect field `name`, if present, to be either an assignment (`= value`) or a definition (`= { block }`).
    /// Expect no more than one `name` field in the block.
    /// Calls the closure `f(bv, data)` for every matching field.
    /// Returns true iff the field is present.
    pub fn field_validated<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BV, &Everything),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                f(bv, self.data);
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_validated`], but the closure is `f(key, bv, data)`.
    pub fn field_validated_key<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BV, &Everything),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                f(key, bv, self.data);
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_validated`], but the closure is `f(bv, data, sc)` where `sc` is
    /// the passed-in [`ScopeContext`].
    ///
    /// This method is useful for delegating to [`validate_desc`](crate::desc::validate_desc) which takes a bv and a sc.
    pub fn field_validated_sc<F>(&mut self, name: &str, sc: &mut ScopeContext, mut f: F) -> bool
    where
        F: FnMut(&BV, &Everything, &mut ScopeContext),
    {
        self.field_validated(name, |bv, data| f(bv, data, sc))
    }

    /// Just like [`Validator::field_validated_sc`], but instead of a full [`ScopeContext`] it just gets the scope type
    /// to be used for the `root` of a [`ScopeContext`] that is made on the spot. This is a convenient way to associate the
    /// `root` type with the key of this field, for clearer warnings. A passed-in [`ScopeContext`] would have to be associated
    /// with a key that is further away.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn field_validated_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F) -> bool
    where
        F: FnMut(&BV, &Everything, &mut ScopeContext),
    {
        self.field_validated_key(name, |key, bv, data| {
            let mut sc = ScopeContext::new(scopes, key);
            f(bv, data, &mut sc);
        })
    }

    /// Just like [`Validator::field_validated`], but expect any number of `name` fields in the block.
    pub fn multi_field_validated<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&BV, &Everything),
    {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                f(bv, self.data);
                found = true;
            }
        }
        found
    }

    /// Just like [`Validator::field_validated_key`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_validated_key<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &BV, &Everything),
    {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                f(key, bv, self.data);
                found = true;
            }
        }
        found
    }

    /// Just like [`Validator::field_validated_sc`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_validated_sc<F>(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&BV, &Everything, &mut ScopeContext),
    {
        self.multi_field_validated(name, |b, data| f(b, data, sc))
    }

    /// Just like [`Validator::field_validated_block`], but expect any number of `name` fields in the block.
    pub fn multi_field_validated_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    f(block, self.data);
                }
                found = true;
            }
        }
        found
    }

    /// Just like [`Validator::field_validated_block_sc`], but expect any number of `name` fields in the block.
    pub fn multi_field_validated_block_sc<F>(
        &mut self,
        name: &str,
        sc: &mut ScopeContext,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        self.multi_field_validated_block(name, |b, data| f(b, data, sc))
    }

    /// Expect field `name`, if present, to be a definition `name = { block }`.
    /// Expect no more than one `name` field in the block.
    /// Calls the closure `f(block, data)` for every matching field.
    /// Returns true iff the field is present.
    pub fn field_validated_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    f(block, self.data);
                }
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_validated_block`], but the closure is `f(key, block, data)`.
    pub fn field_validated_key_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Block, &Everything),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    f(key, block, self.data);
                }
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_validated_key_block`], but expect any number of `name` fields in the block.
    pub fn multi_field_validated_key_block<F>(&mut self, name: &str, mut f: F) -> bool
    where
        F: FnMut(&Token, &Block, &Everything),
    {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    f(key, block, self.data);
                }
                found = true;
            }
        }
        found
    }

    /// Just like [`Validator::field_validated_block`], but the closure is `f(block, data, sc)` where sc is the passed-in `ScopeContext`.
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

    /// Just like [`Validator::field_validated_block_sc`], but instead of a full [`ScopeContext`] it just gets the scope type
    /// to be used for the `root` of a [`ScopeContext`] that is made on the spot. This is a convenient way to associate the
    /// `root` type with the key of this field, for clearer warnings. A passed-in [`ScopeContext`] would have to be associated
    /// with a key that is further away.
    pub fn field_validated_block_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F) -> bool
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        let mut found = None;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    let mut sc = ScopeContext::new(scopes, key);
                    f(block, self.data, &mut sc);
                }
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_validated_block_rooted`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_validated_block_rooted<F>(&mut self, name: &str, scopes: Scopes, mut f: F)
    where
        F: FnMut(&Block, &Everything, &mut ScopeContext),
    {
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    let mut sc = ScopeContext::new(scopes, key);
                    f(block, self.data, &mut sc);
                }
            }
        }
    }

    /// Just like [`Validator::field_validated_block_rooted`], but it takes the passed-in `ScopeContext` and associates its
    /// root with this field's key instead of whatever it was associated with before. This is purely to get better warnings.
    ///
    /// TODO: get rid of this in favor of making proper `ScopeContext` to begin with.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
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
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                if let Some(other) = found {
                    dup_assign_error(key, other);
                }
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    let mut sc = sc.clone();
                    sc.change_root(scopes, key);
                    f(block, self.data, &mut sc);
                }
                found = Some(key);
            }
        }
        found.is_some()
    }

    /// Just like [`Validator::field_block`], but expect any number of `name` fields in the block.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn multi_field_block(&mut self, name: &str) -> bool {
        let mut found = false;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is(name) {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                bv.expect_block();
                found = true;
            }
        }
        found
    }

    /// Expect this [`Block`] to be a block with exactly `expect` loose values that are integers.
    /// So it's of the form `{ 1 2 3 }`.
    pub fn req_tokens_integers_exactly(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for token in self.block.iter_values() {
            if token.expect_integer().is_some() {
                found += 1;
            }
        }
        if found != expect {
            let msg = format!("expected {expect} integers");
            let sev = Severity::Error.at_most(self.max_severity);
            report(ErrorKey::Validation, sev).msg(msg).loc(self.block).push();
        }
    }

    /// Expect this [`Block`] to be a block at least `expect` loose values that are integers.
    /// So it's of the form `{ 1 2 3 }`.
    #[cfg(feature = "imperator")]
    pub fn req_tokens_integers_at_least(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for token in self.block.iter_values() {
            if token.expect_integer().is_some() {
                found += 1;
            }
        }
        if found < expect {
            let msg = format!("expected {expect} integers");
            let sev = Severity::Error.at_most(self.max_severity);
            report(ErrorKey::Validation, sev).msg(msg).loc(self.block).push();
        }
    }

    /// Expect this [`Block`] to be a block with exactly `expect` loose values that are numeric with up to 5 decimals.
    /// So it's of the form `{ 0.0 0.5 1.0 }`
    pub fn req_tokens_numbers_exactly(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for token in self.block.iter_values() {
            if token.expect_number().is_some() {
                found += 1;
            }
        }
        if found != expect {
            let msg = format!("expected {expect} numbers");
            let sev = Severity::Error.at_most(self.max_severity);
            report(ErrorKey::Validation, sev).msg(msg).loc(self.block).push();
        }
    }

    /// Expect field `name`, if present, to be of the form `name = { value value value ... }` with exactly `expect` values.
    /// Expect every value to be a number with up to 5 decimals.
    /// Expect no more than one `name` field in the block.
    pub fn field_list_numeric_exactly(&mut self, name: &str, expect: usize) {
        self.field_validated_block(name, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_numbers_exactly(expect);
        });
    }

    /// Like [`Validator::req_tokens_numbers_exactly`] but the numbers can have any number of decimals.
    pub fn req_tokens_precise_numbers_exactly(&mut self, expect: usize) {
        self.accepted_tokens = true;
        let mut found = 0;
        for token in self.block.iter_values() {
            if token.expect_precise_number().is_some() {
                found += 1;
            }
        }
        if found != expect {
            let msg = format!("expected {expect} numbers");
            let sev = Severity::Error.at_most(self.max_severity);
            report(ErrorKey::Validation, sev).msg(msg).loc(self.block).push();
        }
    }

    /// Like [`Validator::field_list_numeric_exactly`] but the numbers can have any number of decimals.
    pub fn field_list_precise_numeric_exactly(&mut self, name: &str, expect: usize) {
        self.field_validated_block(name, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_precise_numbers_exactly(expect);
        });
    }

    /// Like [`Validator::field_list_numeric_exactly`] but the numbers have to be integers.
    pub fn field_list_integers_exactly(&mut self, name: &str, expect: usize) {
        self.field_validated_block(name, |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_tokens_integers_exactly(expect);
        });
    }

    /// If `name` is present in the block, emit a low-severity warning together with the helpful message `msg`.
    /// This is for harmless but unneeded fields.
    pub fn advice_field(&mut self, name: &str, msg: &str) {
        if let Some(key) = self.block.get_key(name) {
            self.known_fields.push(key.as_str());
            let sev = Severity::Untidy.at_most(self.max_severity);
            report(ErrorKey::Unneeded, sev).msg(msg).loc(key).push();
        }
    }

    /// Expect the block to contain any number of loose values (possibly in addition to other things).
    /// Return a vector of those values.
    /// TODO: make this take a closure or make it an iterator.
    pub fn values(&mut self) -> Vec<&Token> {
        self.accepted_tokens = true;
        self.block.iter_values().collect()
    }

    /// Expect the block to contain any number of loose sub-blocks (possibly in addition to other things).
    /// Return a vector of those blocks.
    /// TODO: make callers use `validated_blocks` instead.
    pub fn blocks(&mut self) -> Vec<&Block> {
        self.accepted_blocks = true;
        self.block.iter_blocks().collect()
    }

    /// Expect the block to contain any number of loose sub-blocks (possibly in addition to other things).
    /// Run the closure `f(block, data)` for every sub-block.
    #[cfg(any(feature = "vic3", feature = "imperator"))] // ck3 happens not to use; silence dead code warning
    pub fn validated_blocks<F>(&mut self, mut f: F)
    where
        F: FnMut(&Block, &Everything),
    {
        self.accepted_blocks = true;
        for block in self.block.iter_blocks() {
            f(block, self.data);
        }
    }

    /// Expect the block to contain any number of `key = { block }` fields where the key is an integer.
    /// Return them as a vector of (key, block) pairs.
    /// TODO: make this take a closure.
    pub fn integer_blocks(&mut self) -> Vec<(&Token, &Block)> {
        let mut vec = Vec::new();
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is_integer() {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    vec.push((key, block));
                }
            }
        }
        vec
    }

    /// Expect the block to contain any number of `key = value` fields where the key is an integer.
    /// Return them as a vector of (key, value) pairs.
    /// TODO: make this take a closure.
    pub fn integer_values(&mut self) -> Vec<(&Token, &Token)> {
        let mut vec = Vec::new();
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is_integer() {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(token) = bv.expect_value() {
                    vec.push((key, token));
                }
            }
        }
        vec
    }

    /// Expect the block to contain any number of `key = value` or `key = { block }` fields where the key is an integer.
    /// Return them as a vector of (key, bv) pairs.
    /// TODO: make this take a closure.
    pub fn integer_keys<F: FnMut(&Token, &BV)>(&mut self, mut f: F) {
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is_integer() {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                f(key, bv);
            }
        }
    }

    /// Expect the block to contain any number of `key = value` or `key = { block }` fields where the key is a number with up to 5 decimals.
    /// Return them as a vector of (key, bv) pairs.
    /// TODO: make this take a closure.
    #[cfg(feature = "vic3")] // ck3 happens not to use; silence dead code warning
    pub fn numeric_keys<F: FnMut(&Token, &BV)>(&mut self, mut f: F) {
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if key.is_number() {
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                f(key, bv);
            }
        }
    }

    /// Expect the block to contain any number of `key = { block }` fields where the key is a date.
    /// Run the closure `f(date, block, data)` for every matching field.
    #[cfg(feature = "ck3")] // vic3 happens not to use; silence dead code warning
    pub fn validate_history_blocks<F>(&mut self, mut f: F)
    where
        F: FnMut(Date, &Token, &Block, &Everything),
    {
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if let Ok(date) = Date::try_from(key) {
                key.expect_date(); // warn about unusual date formats
                self.known_fields.push(key.as_str());
                self.expect_eq_qeq(key, *cmp);
                if let Some(block) = bv.expect_block() {
                    f(date, key, block, self.data);
                }
            }
        }
    }

    /// Expect the block to contain any number of unknown fields (so don't warn about unknown fields anymore).
    /// Loose values and loose sub-blocks will still be warned about.
    /// Run the closure `f(key, bv)` on every matching *unknown* field. Previously-validated fields will be skipped.
    pub fn unknown_fields<F: FnMut(&Token, &BV)>(&mut self, mut f: F) {
        self.accepted_block_fields = true;
        self.accepted_value_fields = true;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            self.expect_eq_qeq(key, *cmp);
            if !self.known_fields.contains(&key.as_str()) {
                f(key, bv);
            }
        }
    }

    /// Expect the block to contain any number of unknown `key = { block }` fields.
    /// Run the closure `f(key, block)` on every matching *unknown* field. Previously-validated fields will be skipped.
    pub fn unknown_block_fields<F: FnMut(&Token, &Block)>(&mut self, mut f: F) {
        self.accepted_block_fields = true;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if let Some(block) = bv.get_block() {
                self.expect_eq_qeq(key, *cmp);
                if !self.known_fields.contains(&key.as_str()) {
                    f(key, block);
                }
            }
        }
    }

    /// Expect the block to contain any number of unknown `key = value` fields.
    /// Run the closure `f(key, value)` on every matching *unknown* field. Previously-validated fields will be skipped.
    pub fn unknown_value_fields<F: FnMut(&Token, &Token)>(&mut self, mut f: F) {
        self.accepted_value_fields = true;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if let Some(value) = bv.get_value() {
                self.expect_eq_qeq(key, *cmp);
                if !self.known_fields.contains(&key.as_str()) {
                    f(key, value);
                }
            }
        }
    }

    /// Like [`Validator::unknown_fields`] but passes the comparator, so that `f` can determine whether it is `=` or `?=`
    /// It still expects the comparator to be one of those two.
    pub fn unknown_fields_cmp<F: FnMut(&Token, Comparator, &BV)>(&mut self, mut f: F) {
        self.accepted_block_fields = true;
        self.accepted_value_fields = true;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if !self.known_fields.contains(&key.as_str()) {
                self.expect_eq_qeq(key, *cmp);
                f(key, *cmp, bv);
            }
        }
    }

    /// Like [`Validator::unknown_fields_cmp`] but accepts and passes any comparator.
    pub fn unknown_fields_any_cmp<F: FnMut(&Token, Comparator, &BV)>(&mut self, mut f: F) {
        self.accepted_block_fields = true;
        self.accepted_value_fields = true;
        for Field(key, cmp, bv) in self.block.iter_fields() {
            if !self.known_fields.contains(&key.as_str()) {
                f(key, *cmp, bv);
            }
        }
    }

    /// Tells the Validator to not warn about any unknown block contents when it goes out of scope.
    /// (The default is to warn.)
    pub fn no_warn_remaining(&mut self) {
        self.accepted_block_fields = true;
        self.accepted_value_fields = true;
        self.accepted_tokens = true;
        self.accepted_blocks = true;
    }

    /// Tells the Validator to warn about any unknown block contents right now, before it goes out of scope.
    /// It will not warn again when it does go out of scope.
    /// Returns true iff any warnings were emitted.
    pub fn warn_remaining(&mut self) -> bool {
        let mut warned = false;
        for item in self.block.iter_items() {
            match item {
                BlockItem::Field(Field(key, _, bv)) => match bv {
                    BV::Value(_) => {
                        if !self.accepted_value_fields && !self.known_fields.contains(&key.as_str())
                        {
                            let msg = format!("unknown field `{key}`");
                            let sev = Severity::Error.at_most(self.max_severity);
                            report(ErrorKey::UnknownField, sev).weak().msg(msg).loc(key).push();
                            warned = true;
                        }
                    }
                    BV::Block(_) => {
                        if !self.accepted_block_fields && !self.known_fields.contains(&key.as_str())
                        {
                            let msg = format!("unknown field `{key}`");
                            let sev = Severity::Error.at_most(self.max_severity);
                            report(ErrorKey::UnknownField, sev).weak().msg(msg).loc(key).push();
                            warned = true;
                        }
                    }
                },
                BlockItem::Value(t) => {
                    if !self.accepted_tokens {
                        let msg = format!("found loose value {t}, expected only `key =`");
                        let sev = Severity::Error.at_most(self.max_severity);
                        report(ErrorKey::Structure, sev).msg(msg).loc(t).push();
                        warned = true;
                    }
                }
                BlockItem::Block(b) => {
                    if !self.accepted_blocks {
                        let msg = "found sub-block, expected only `key =`";
                        let sev = Severity::Error.at_most(self.max_severity);
                        report(ErrorKey::Structure, sev).msg(msg).loc(b).push();
                        warned = true;
                    }
                }
            }
        }
        self.no_warn_remaining();
        warned
    }

    fn expect_eq_qeq(&self, key: &Token, cmp: Comparator) {
        #[allow(clippy::collapsible_else_if)]
        if self.allow_questionmark_equals {
            if !matches!(cmp, Comparator::Equals(Single | Question)) {
                let msg = format!("expected `{key} =` or `?=`, found `{cmp}`");
                let sev = Severity::Error.at_most(self.max_severity);
                report(ErrorKey::Validation, sev).msg(msg).loc(key).push();
            }
        } else {
            if !matches!(cmp, Comparator::Equals(Single)) {
                let msg = format!("expected `{key} =`, found `{cmp}`");
                let sev = Severity::Error.at_most(self.max_severity);
                report(ErrorKey::Validation, sev).msg(msg).loc(key).push();
            }
        }
    }
}

impl<'a> Drop for Validator<'a> {
    fn drop(&mut self) {
        self.warn_remaining();
    }
}
