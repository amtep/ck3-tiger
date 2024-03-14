use std::borrow::Cow;
use std::fmt::{Debug, Display, Error, Formatter};
#[cfg(feature = "ck3")]
use std::ops::{Bound, RangeBounds};
use std::str::FromStr;

use crate::context::ScopeContext;
use crate::date::Date;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{report, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_target;
#[cfg(feature = "imperator")]
use crate::trigger::validate_target_ok_this;

/// A validator for one `Token`.
/// The intended usage is that the block-level [`Validator`](crate::validator::Validator) wraps the `Token`
/// in a `ValueValidator`, then you call one or more of the validation functions on it.
/// If the `ValueValidator` goes out of scope without having been validated, it will report an error.
///
/// Calling multiple validation functions is only supported when starting with the `maybe_`
/// variants. Calling one of the definite validation methods will either accept the value or warn
/// about it, and it's considered validated after that.
pub struct ValueValidator<'a> {
    /// The value being validated
    value: Cow<'a, Token>,
    /// A link to all the loaded and processed CK3 and mod files
    data: &'a Everything,
    /// Whether the value has been validated. If true, it means either the value was accepted as
    /// correct or it was warned about.
    validated: bool,
    /// Maximum severity of problems reported by this `ValueValidator`. Defaults to `Error`.
    /// This is intended to be set lower by validators for less-important items.
    /// As an exception, `Fatal` severity reports will still always be logged as `Fatal`.
    max_severity: Severity,
}

impl<'a> Debug for ValueValidator<'a> {
    /// Roll our own `Debug` implementation in order to leave out the `data` field.
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("ValueValidator")
            .field("value", &self.value)
            .field("validated", &self.validated)
            .field("max_severity", &self.max_severity)
            .finish()
    }
}

impl<'a> ValueValidator<'a> {
    /// Construct a new `ValueValidator` for a `&Token`.
    pub fn new(value: &'a Token, data: &'a Everything) -> Self {
        Self { value: Cow::Borrowed(value), data, validated: false, max_severity: Severity::Error }
    }

    /// Construct a new `ValueValidator` for an owned `Token`.
    pub fn new_owned(value: Token, data: &'a Everything) -> Self {
        Self { value: Cow::Owned(value), data, validated: false, max_severity: Severity::Error }
    }

    /// Maximum severity of problems reported by this `ValueValidator`. Defaults to `Error`.
    /// This is intended to be set lower by validators for less-important items.
    /// As an exception, `Fatal` severity reports will still always be logged as `Fatal`.
    pub fn set_max_severity(&mut self, max_severity: Severity) {
        self.max_severity = max_severity;
    }

    /// Make a descendant validator from this one, usually for a sub-token.
    fn value_validator(&self, token: Token) -> Self {
        let mut vd = ValueValidator::new_owned(token, self.data);
        vd.set_max_severity(self.max_severity);
        vd
    }

    /// Access the value that this `ValueValidator` validates.
    pub fn value(&self) -> &Token {
        &self.value
    }

    /// Mark this value as valid, without placing any restrictions on it.
    pub fn accept(&mut self) {
        self.validated = true;
    }

    /// Expect the value to be the key of an `itype` item the game database.
    /// The item is looked up and must exist.
    #[cfg(not(feature = "imperator"))] // silence dead code warning
    pub fn item(&mut self, itype: Item) {
        if self.validated {
            return;
        }
        self.validated = true;
        self.data.verify_exists_max_sev(itype, &self.value, self.max_severity);
    }

    /// Add the given suffix to the value and mark that as a used item, without doing any validation.
    /// This is used for very weakly required localization, for example, where no warning is warranted.
    #[cfg(feature = "ck3")] // silence dead code warning
    pub fn item_used_with_suffix(&mut self, itype: Item, sfx: &str) {
        let implied = format!("{}{sfx}", self.value);
        self.data.mark_used(itype, &implied);
    }

    /// Validate a localization whose key is derived from the value, in the given [`ScopeContext`].
    #[allow(dead_code)]
    pub fn implied_localization_sc(&mut self, pfx: &str, sfx: &str, sc: &mut ScopeContext) {
        let implied = format!("{pfx}{}{sfx}", self.value);
        self.data.validate_localization_sc(&implied, sc);
    }

    /// Check if the value is the key of an `itype` item the game database.
    /// The item is looked up, and if it exists then this validator is considered validated.
    /// Return whether the item exists.
    #[cfg(feature = "ck3")] // silence dead code warning
    pub fn maybe_item(&mut self, itype: Item) -> bool {
        if self.data.item_exists(itype, self.value.as_str()) {
            self.validated = true;
            true
        } else {
            false
        }
    }

    /// Check if the value is be the key of an `itype` item the game database, after removing the prefix `pfx`.
    /// The item is looked up, and if it exists then this validator is considered validated.
    /// Return whether the item exists.
    #[cfg(feature = "vic3")] // silence dead code warning
    pub fn maybe_prefix_item(&mut self, pfx: &str, itype: Item) -> bool {
        if let Some(value) = self.value.as_str().strip_prefix(pfx) {
            if self.data.item_exists(itype, value) {
                self.validated = true;
                return true;
            }
        }
        false
    }

    /// Expect the value to be the name of a file under the directory given here.
    #[cfg(feature = "ck3")] // silence dead code warning
    pub fn dir_file(&mut self, path: &str) {
        if self.validated {
            return;
        }
        self.validated = true;
        let pathname = format!("{path}/{}", self.value);
        // TODO: pass max_severity here
        self.data.verify_exists_implied(Item::File, &pathname, &self.value);
    }

    /// Expect the value to be the name of a file under the directory specified by the given `define`.
    #[cfg(feature = "ck3")] // silence dead code warnings
    pub fn defined_dir_file(&mut self, define: &str) {
        if self.validated {
            return;
        }
        self.validated = true;
        if let Some(path) = self.data.get_defined_string_warn(&self.value, define) {
            let pathname = format!("{path}/{}", self.value);
            // TODO: pass max_severity here
            self.data.verify_exists_implied(Item::File, &pathname, &self.value);
        }
    }

    #[must_use]
    pub fn split(&mut self, c: char) -> Vec<ValueValidator> {
        self.validated = true;
        self.value.split(c).into_iter().map(|value| self.value_validator(value)).collect()
    }

    /// Expect the value to be a (possibly single-element) scope chain which evaluates to a scope type in `outscopes`.
    ///
    /// The value is evaluated in the scope context `sc`, so for example if the value does `scope:actor` but there is
    /// no named scope "actor" in the scope context, then a warning is emitted.
    ///
    /// Also emits a warning if the value is simply "`this`", because that is almost never correct.
    #[allow(dead_code)] // not used yet
    pub fn target(&mut self, sc: &mut ScopeContext, outscopes: Scopes) {
        if self.validated {
            return;
        }
        self.validated = true;
        // TODO: pass max_severity here
        validate_target(&self.value, self.data, sc, outscopes);
    }

    /// Just like [`ValueValidator::target`], but allows the value to be simply "`this`".
    /// It is expected to be used judiciously in cases where "`this`" can be correct.
    #[cfg(feature = "imperator")]
    pub fn target_ok_this(&mut self, sc: &mut ScopeContext, outscopes: Scopes) {
        if self.validated {
            return;
        }
        self.validated = true;
        // TODO: pass max_severity here
        validate_target_ok_this(&self.value, self.data, sc, outscopes);
    }

    /// This is a combination of [`ValueValidator::item`] and [`ValueValidator::target`]. If the field is present
    /// and is not a known `itype` item, then it is evaluated as a target.
    #[allow(dead_code)] // not used yet
    pub fn item_or_target(&mut self, sc: &mut ScopeContext, itype: Item, outscopes: Scopes) {
        if self.validated {
            return;
        }
        self.validated = true;
        if !self.data.item_exists(itype, self.value.as_str()) {
            // TODO: pass max_severity here
            validate_target(&self.value, self.data, sc, outscopes);
        }
    }

    /// Expect the value to be just `yes` or `no`.
    #[cfg(feature = "ck3")] // silence dead code warning
    pub fn bool(&mut self) {
        if self.validated {
            return;
        }
        let sev = Severity::Error.at_most(self.max_severity);
        self.validated = true;
        if !self.value.lowercase_is("yes") && !self.value.lowercase_is("no") {
            report(ErrorKey::Validation, sev).msg("expected yes or no").loc(self).push();
        }
    }

    /// Expect the value to be an integer.
    #[cfg(not(feature = "imperator"))]
    pub fn integer(&mut self) {
        if self.validated {
            return;
        }
        self.validated = true;
        // TODO: pass max_severity here
        self.value.expect_integer();
    }

    /// Expect the value to be an integer between `low` and `high` (inclusive).
    #[cfg(feature = "ck3")]
    pub fn integer_range<R: RangeBounds<i64>>(&mut self, range: R) {
        if self.validated {
            return;
        }
        let sev = Severity::Error.at_most(self.max_severity);
        self.validated = true;
        // TODO: pass max_severity here
        if let Some(i) = self.value.expect_integer() {
            if !range.contains(&i) {
                let low = match range.start_bound() {
                    Bound::Unbounded => None,
                    Bound::Included(&n) => Some(n),
                    Bound::Excluded(&n) => Some(n + 1),
                };
                let high = match range.end_bound() {
                    Bound::Unbounded => None,
                    Bound::Included(&n) => Some(n),
                    Bound::Excluded(&n) => Some(n - 1),
                };
                let msg;
                if low.is_some() && high.is_some() {
                    msg = format!(
                        "should be between {} and {} (inclusive)",
                        low.unwrap(),
                        high.unwrap()
                    );
                } else if low.is_some() {
                    msg = format!("should be at least {}", low.unwrap());
                } else if high.is_some() {
                    msg = format!("should be at most {}", high.unwrap());
                } else {
                    unreachable!(); // could not have failed the contains check
                }
                report(ErrorKey::Range, sev).msg(msg).loc(self).push();
            }
        }
    }

    /// Expect the value to be a number with up to 5 decimals.
    /// (5 decimals is the limit accepted by the game engine in most contexts).
    #[allow(dead_code)] // not used yet
    pub fn numeric(&mut self) {
        if self.validated {
            return;
        }
        self.validated = true;
        // TODO: pass max_severity here
        self.value.expect_number();
    }

    /// Expect the value to be a number with any number of decimals.
    #[allow(dead_code)] // not used yet
    pub fn precise_numeric(&mut self) {
        if self.validated {
            return;
        }
        self.validated = true;
        // TODO: pass max_severity here
        self.value.expect_precise_number();
    }

    /// Expect the value to be a date.
    /// The format of dates is very flexible, from a single number (the year), to a year.month or year.month.day.
    /// No checking is done on the validity of the date as a date (so January 42nd is okay).
    #[allow(dead_code)] // not used yet
    pub fn date(&mut self) {
        if self.validated {
            return;
        }
        let sev = Severity::Error.at_most(self.max_severity);
        self.validated = true;
        if Date::from_str(self.value.as_str()).is_err() {
            let msg = "expected date value";
            report(ErrorKey::Validation, sev).msg(msg).loc(self).push();
        }
    }

    /// Expect the value to be one of the listed strings in `choices`.
    pub fn choice(&mut self, choices: &[&str]) {
        if self.validated {
            return;
        }
        self.validated = true;
        let sev = Severity::Error.at_most(self.max_severity);
        if !choices.contains(&self.value.as_str()) {
            let msg = format!("expected one of {}", choices.join(", "));
            report(ErrorKey::Choice, sev).msg(msg).loc(self).push();
        }
    }

    /// Check if the value is equal to the given string.
    /// If it is, mark this value as validated.
    /// Return whether the value matched the string.
    pub fn maybe_is(&mut self, s: &str) -> bool {
        if self.value.is(s) {
            self.validated = true;
            true
        } else {
            false
        }
    }

    /// Tells the `ValueValidator` to report a warning if the value is still unvalidated.
    pub fn warn_unvalidated(&mut self) {
        if !self.validated {
            let sev = Severity::Error.at_most(self.max_severity);
            let msg = format!("unknown value `{}`", self.value);
            report(ErrorKey::Validation, sev).msg(msg).loc(self).push();
        }
    }
}

impl<'a> Drop for ValueValidator<'a> {
    fn drop(&mut self) {
        self.warn_unvalidated();
    }
}

impl<'a> Display for ValueValidator<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        Display::fmt(&self.value, f)
    }
}
