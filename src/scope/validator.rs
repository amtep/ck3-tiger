use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
use crate::scope::{Comparator, Scope, ScopeOrValue, Token};
use crate::validate::{Validate, ValidationError};

#[derive(Debug)]
pub struct Validator<'a> {
    // The scope being validated
    scope: &'a Scope,
    // Identifier used for error messages
    id: &'a str,
    // Fields that have been requested so far
    known_fields: Vec<&'a str>,
    // Fatal error, if any
    pub err: Option<ValidationError>,
}

impl<'a> Validator<'a> {
    pub fn new(scope: &'a Scope, id: &'a str) -> Self {
        Validator {
            scope,
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
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
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
                        ScopeOrValue::Token(t) => value = Some(t.clone()),
                        ScopeOrValue::Scope(s) => {
                            self.err(ValidationError::RequiredFieldInvalid(name.to_string()));
                            error(s, ErrorKey::Validation, "expected value, found scope");
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
                &self.scope.loc,
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
        F: FnMut(ScopeOrValue) -> Option<T>,
    {
        self.known_fields.push(name);

        let mut found = false;
        let mut value = None;
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
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

    pub fn allow_unique_field(&mut self, name: &'a str) -> Option<ScopeOrValue> {
        self.allow_unique_field_check(name, Some)
    }

    pub fn allow_unique_field_value(&mut self, name: &'a str) -> Option<Token> {
        self.allow_unique_field_check(name, |v| match v {
            ScopeOrValue::Token(t) => Some(t),
            ScopeOrValue::Scope(s) => {
                error(s, ErrorKey::Validation, "expected value, found scope");
                None
            }
        })
    }

    pub fn allow_unique_field_scope(&mut self, name: &'a str) -> Option<Scope> {
        self.allow_unique_field_check(name, |v| match v {
            ScopeOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected scope, found value");
                None
            }
            ScopeOrValue::Scope(s) => Some(s),
        })
    }

    pub fn allow_unique_field_boolean(&mut self, name: &'a str) -> Option<bool> {
        self.allow_unique_field_check(name, |v| match v {
            ScopeOrValue::Token(t) if t.as_str() == "yes" => Some(true),
            ScopeOrValue::Token(t) if t.as_str() == "no" => Some(false),
            ScopeOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected yes or no");
                None
            }
            ScopeOrValue::Scope(s) => {
                error(s, ErrorKey::Validation, "expected value, found scope");
                None
            }
        })
    }

    pub fn allow_unique_field_integer(&mut self, name: &'a str) -> Option<i64> {
        self.allow_unique_field_check(name, |v| match v {
            ScopeOrValue::Token(t) => {
                if let Ok(i) = t.as_str().parse() {
                    Some(i)
                } else {
                    error(t, ErrorKey::Validation, "expected integer");
                    None
                }
            }
            ScopeOrValue::Scope(s) => {
                error(s, ErrorKey::Validation, "expected value, found scope");
                None
            }
        })
    }

    pub fn allow_unique_field_list(&mut self, name: &'a str) -> Option<Vec<Token>> {
        self.allow_unique_field_check(name, |v| match v {
            ScopeOrValue::Token(t) => {
                error(t, ErrorKey::Validation, "expected scope, found value");
                None
            }
            ScopeOrValue::Scope(s) => {
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
                        ScopeOrValue::Token(t) => vec.push(t.clone()),
                        ScopeOrValue::Scope(s) => {
                            error(s, ErrorKey::Validation, "expected value, found scope");
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
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        ScopeOrValue::Token(t) => vec.push(t.clone()),
                        ScopeOrValue::Scope(s) => {
                            error(s, ErrorKey::Validation, "expected value, found scope");
                        }
                    }
                }
            }
        }
        vec
    }

    pub fn allow_field_validated_scopes<V: Validate>(
        &mut self,
        name: &'a str,
    ) -> Result<Vec<V>, ValidationError> {
        self.known_fields.push(name);
        let id = format!("{}.{}", self.id, name);

        let mut vec = Vec::new();
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        ScopeOrValue::Token(t) => {
                            error(t, ErrorKey::Validation, "expected scope, found value");
                        }
                        ScopeOrValue::Scope(s) => vec.push(V::from_scope(s.clone(), &id)?),
                    }
                }
            }
        }
        Ok(vec)
    }

    pub fn allow_field_scopes(&mut self, name: &'a str) -> Vec<Scope> {
        self.known_fields.push(name);

        let mut vec = Vec::new();
        for (k, cmp, v) in &self.scope.v {
            if let Some(key) = k {
                if key.as_str() == name {
                    if !matches!(cmp, Comparator::Eq) {
                        error(
                            key,
                            ErrorKey::Validation,
                            &format!("expected `{} =`, found `{}`", key, cmp),
                        );
                    }
                    match v {
                        ScopeOrValue::Token(t) => {
                            error(t, ErrorKey::Validation, "expected scope, found value");
                        }
                        ScopeOrValue::Scope(s) => vec.push(s.clone()),
                    }
                }
            }
        }
        vec
    }

    pub fn warn_unused_entries(&mut self) {
        for (k, _, v) in &self.scope.v {
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
                    ScopeOrValue::Token(t) => {
                        warn(
                            t,
                            ErrorKey::Validation,
                            "found loose value, expected only `key =`",
                        );
                    }
                    ScopeOrValue::Scope(s) => {
                        warn(
                            s,
                            ErrorKey::Validation,
                            "found subscope, expected only `key =`",
                        );
                    }
                },
            }
        }
    }
}
