use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKey {
    Config,
    ReadError,
    ParseError,
    BracePlacement,
    Bom,
    Packaging,
    Validation,
    TooManyErrors,
    Filename,
    Encoding,
    Localization,
    LocalizationDup,
    EventNamespace,
}

// This has to be kept up to date with ErrorKey and with its Display impl
impl FromStr for ErrorKey {
    type Err = ParseKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let key = match s {
            "config" => ErrorKey::Config,
            "read-error" => ErrorKey::ReadError,
            "parse-error" => ErrorKey::ParseError,
            "brace-placement" => ErrorKey::BracePlacement,
            "bom" => ErrorKey::Bom,
            "packaging" => ErrorKey::Packaging,
            "validation" => ErrorKey::Validation,
            "too-many-errors" => ErrorKey::TooManyErrors,
            "filename" => ErrorKey::Filename,
            "encoding" => ErrorKey::Encoding,
            "localization" => ErrorKey::Localization,
            "localization-dup" => ErrorKey::LocalizationDup,
            "event-namespace" => ErrorKey::EventNamespace,
            _ => {
                return Err(ParseKeyError::new("unknown error key"));
            }
        };
        Ok(key)
    }
}

impl Display for ErrorKey {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            ErrorKey::Config => write!(fmt, "config"),
            ErrorKey::ReadError => write!(fmt, "read-error"),
            ErrorKey::ParseError => write!(fmt, "parse-error"),
            ErrorKey::BracePlacement => write!(fmt, "brace-placement"),
            ErrorKey::Bom => write!(fmt, "bom"),
            ErrorKey::Packaging => write!(fmt, "packaging"),
            ErrorKey::Validation => write!(fmt, "validation"),
            ErrorKey::TooManyErrors => write!(fmt, "too-many-errors"),
            ErrorKey::Filename => write!(fmt, "filename"),
            ErrorKey::Encoding => write!(fmt, "encoding"),
            ErrorKey::Localization => write!(fmt, "localization"),
            ErrorKey::LocalizationDup => write!(fmt, "localization-dup"),
            ErrorKey::EventNamespace => write!(fmt, "event-namespace"),
        }
    }
}

#[derive(Debug, Error)]
pub struct ParseKeyError {
    msg: String,
}

impl ParseKeyError {
    fn new(msg: &str) -> Self {
        ParseKeyError {
            msg: msg.to_string(),
        }
    }
}

impl Display for ParseKeyError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.msg)
    }
}
