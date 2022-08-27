use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKey {
    Config,
    ReadError,
    ParseError,
    BracePlacement,
    Packaging,
    Validation,
    TooManyErrors,
    Filename,
    Encoding,
    Localization,
    Duplicate,
    EventNamespace,
    MissingLocalization,
    MissingFile,
    Conflict,
    ImageFormat,
    Unneeded,
    PrincesOfDarkness,
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
            "packaging" => ErrorKey::Packaging,
            "validation" => ErrorKey::Validation,
            "too-many-errors" => ErrorKey::TooManyErrors,
            "filename" => ErrorKey::Filename,
            "encoding" => ErrorKey::Encoding,
            "localization" => ErrorKey::Localization,
            "duplicate" => ErrorKey::Duplicate,
            "event-namespace" => ErrorKey::EventNamespace,
            "missing-localization" => ErrorKey::MissingLocalization,
            "missing-file" => ErrorKey::MissingFile,
            "conflict" => ErrorKey::Conflict,
            "image-format" => ErrorKey::ImageFormat,
            "unneeded" => ErrorKey::Unneeded,
            "princes-of-darkness" => ErrorKey::PrincesOfDarkness,
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
            ErrorKey::Packaging => write!(fmt, "packaging"),
            ErrorKey::Validation => write!(fmt, "validation"),
            ErrorKey::TooManyErrors => write!(fmt, "too-many-errors"),
            ErrorKey::Filename => write!(fmt, "filename"),
            ErrorKey::Encoding => write!(fmt, "encoding"),
            ErrorKey::Localization => write!(fmt, "localization"),
            ErrorKey::Duplicate => write!(fmt, "duplicate"),
            ErrorKey::EventNamespace => write!(fmt, "event-namespace"),
            ErrorKey::MissingLocalization => write!(fmt, "missing-localization"),
            ErrorKey::MissingFile => write!(fmt, "missing-file"),
            ErrorKey::Conflict => write!(fmt, "conflict"),
            ErrorKey::ImageFormat => write!(fmt, "image-format"),
            ErrorKey::Unneeded => write!(fmt, "unneeded"),
            ErrorKey::PrincesOfDarkness => write!(fmt, "princes-of-darkness"),
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
