use crate::errors::{error, warn, ErrorKey};
use crate::scope::{Comparator, Scope, ScopeValue, Token};
use crate::validate::ValidationError;

#[derive(Debug)]
pub struct Validator<'a> {
    // The scope being validated
    scope: &'a Scope,
    // Identifier used for error messages
    id: &'a str,
    // Fields that have been requested so far
    known_fields: Vec<&'a str>,
    // Max number of errors to report
    error_limit: usize,
    // Errors reported so far,
    errors: usize,
    // Message to print when error limit is reached.
    error_limit_msg: &'a str,
    // Fatal error, if any
    pub err: Option<ValidationError>,
}

impl<'a> Validator<'a> {
    pub fn new(scope: &'a Scope, id: &'a str) -> Self {
        Validator {
            scope,
            id,
            known_fields: Vec::new(),
            error_limit: 9,
            errors: 0,
            error_limit_msg: "too many errors",
            err: None,
        }
    }

    pub fn warn(&mut self, t: &Token, msg: &str) {
        if self.errors < self.error_limit {
            warn(t, ErrorKey::Validation, msg);
            self.errors += 1;
            if self.errors == self.error_limit {
                warn(t, ErrorKey::TooManyErrors, self.error_limit_msg);
            }
        }
    }

    pub fn error(&mut self, t: &Token, msg: &str) {
        if self.errors < self.error_limit {
            error(t, ErrorKey::Validation, msg);
            self.errors += 1;
            if self.errors == self.error_limit {
                warn(t, ErrorKey::TooManyErrors, self.error_limit_msg);
            }
        }
    }

    pub fn err(&mut self, e: ValidationError) {
        if self.err.is_none() {
            self.err = Some(e);
        }
    }

    pub fn error_limit(&mut self, limit: usize, msg: &'a str) {
        self.error_limit = limit;
        self.error_limit_msg = msg;
    }

    pub fn require_unique_field_value(&mut self, name: &'a str) {
        self.known_fields.push(name);

        let mut found = false;
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if found {
                        self.warn(
                            key,
                            &format!("multiple definitions of `{}`, expected only one.", key),
                        );
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        self.error(key, &format!("expected `{} =`, found `{}`", key, cmp));
                    }
                    match v {
                        ScopeValue::Token(_) => (),
                        ScopeValue::Scope(s) => {
                            self.err(ValidationError::RequiredFieldInvalid(name.to_string()));
                            self.error(&s.token(), "expected value, found scope");
                        }
                    }
                    found = true;
                }
            }
        }
        if !found {
            self.err(ValidationError::RequiredFieldMissing(name.to_string()));
            self.error(
                &self.scope.token(),
                &format!("required field `{}` missing", name),
            );
        }
    }

    pub fn allow_unique_field_value(&mut self, name: &'a str) {
        self.known_fields.push(name);

        let mut found = false;
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if found {
                        self.warn(
                            key,
                            &format!("multiple definitions of `{}`, expected only one.", key),
                        );
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        self.error(key, &format!("expected `{} =`, found `{}`", key, cmp));
                    }
                    match v {
                        ScopeValue::Token(_) => (),
                        ScopeValue::Scope(s) => {
                            self.error(&s.token(), "expected value, found scope");
                        }
                    }
                    found = true;
                }
            }
        }
    }

    pub fn allow_unique_field_list(&mut self, name: &'a str) {
        self.known_fields.push(name);

        let mut found = false;
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if found {
                        self.warn(
                            key,
                            &format!("multiple definitions of `{}`, expected only one.", key),
                        );
                    }
                    if !matches!(cmp, Comparator::Eq) {
                        self.error(key, &format!("expected `{} =`, found `{}`", key, cmp));
                    }
                    match v {
                        ScopeValue::Token(t) => self.error(t, "expected scope, found value"),
                        ScopeValue::Scope(s) => {
                            for (k, _, v) in &s.v {
                                if let Some(key) = k {
                                    self.warn(
                                        key,
                                        &format!("found key `{}`, expected only values", key),
                                    );
                                }
                                match v {
                                    ScopeValue::Token(_) => (),
                                    ScopeValue::Scope(s) => {
                                        self.error(&s.token(), "expected value, found scope");
                                    }
                                }
                            }
                        }
                    }
                    found = true;
                }
            }
        }
    }

    pub fn allow_multiple_field_values(&mut self, name: &'a str) {
        self.known_fields.push(name);

        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if !matches!(cmp, Comparator::Eq) {
                        self.error(key, &format!("expected `{} =`, found `{}`", key, cmp));
                    }
                    match v {
                        ScopeValue::Token(_) => (),
                        ScopeValue::Scope(s) => {
                            self.error(&s.token(), "expected value, found scope");
                        }
                    }
                }
            }
        }
    }

    pub fn warn_unused_entries(&mut self) {
        for (k, _, v) in &self.scope.v {
            match k {
                Some(key) => {
                    if !self.known_fields.contains(&key.as_str()) {
                        self.warn(key, &format!("unknown field `{}` for {}", key, self.id));
                    }
                }
                None => match v {
                    ScopeValue::Token(t) => {
                        self.warn(t, "found loose value, expected only `key =`")
                    }
                    ScopeValue::Scope(s) => {
                        self.warn(&s.token(), "found subscope, expected only `key =`");
                    }
                },
            }
        }
    }
}
