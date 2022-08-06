use std::fmt::{Display, Formatter};
use std::slice::Iter;

use crate::scope::Token;

#[derive(Clone, Debug, Default)]
pub struct Errors {
    v: Vec<Error>,
}

#[derive(Clone, Debug)]
pub struct Error {
    token: Token,
    level: ErrorLevel,
    key: ErrorKey,
    msg: String,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKey {
    ParseError,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorLevel {
    Error,
    Warning,
    Advice,
}

impl Errors {
    pub fn new() -> Self {
        Errors { v: Vec::new() }
    }

    pub fn push(&mut self, token: Token, level: ErrorLevel, key: ErrorKey, msg: String) {
        self.v.push(Error {
            token,
            key,
            level,
            msg,
        });
    }

    pub fn error(&mut self, token: Token, key: ErrorKey, msg: String) {
        self.push(token, ErrorLevel::Error, key, msg);
    }

    pub fn warn(&mut self, token: Token, key: ErrorKey, msg: String) {
        self.push(token, ErrorLevel::Warning, key, msg);
    }

    pub fn advice(&mut self, token: Token, key: ErrorKey, msg: String) {
        self.push(token, ErrorLevel::Advice, key, msg);
    }

    pub fn iter(&self) -> Iter<'_, Error> {
        self.v.iter()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}{}: {}", self.token.loc.marker(), self.level, self.msg)
    }
}

impl Display for ErrorLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ErrorLevel::Error => write!(f, "ERROR"),
            ErrorLevel::Warning => write!(f, "WARNING"),
            ErrorLevel::Advice => write!(f, "ADVICE"),
        }
    }
}
