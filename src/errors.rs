use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::PathBuf;

use crate::everything::FileKind;
use crate::scope::Token;

static mut ERRORS: Option<Errors> = None;

#[derive(Clone, Debug, Default)]
pub struct Errors {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKey {
    ParseError,
    Packaging,
    Validation,
    TooManyErrors,
    Filename,
    Encoding,
    Localization,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorLevel {
    Error,
    Warning,
    Advice,
}

impl Errors {
    #[allow(clippy::unused_self)] // At some point we will cache files in self
    fn get_line(&mut self, token: &Token) -> Option<String> {
        if token.loc.line == 0 {
            return None;
        }
        let pathname = match token.loc.kind {
            FileKind::VanillaFile => self.vanilla_root.join(&*token.loc.pathname),
            FileKind::ModFile => self.mod_root.join(&*token.loc.pathname),
        };
        read_to_string(&pathname)
            .ok()
            .and_then(|contents| contents.lines().nth(token.loc.line - 1).map(str::to_string))
    }

    pub fn push(
        &mut self,
        token: &Token,
        level: ErrorLevel,
        _key: ErrorKey,
        msg: &str,
        info: Option<&str>,
    ) {
        if let Some(line) = self.get_line(token) {
            let line_marker = token.loc.line_marker();
            eprintln!("{}{}", line_marker, line);
            eprintln!("{}{:>count$}", line_marker, "^", count = token.loc.column);
        }
        // TODO: get terminal column width and do line wrapping of msg and info
        eprintln!("{}{}: {}", token.loc.marker(), level, msg);
        if let Some(info) = info {
            eprintln!("  {}", info);
        }
    }

    pub fn get_mut() -> &'static mut Self {
        // Safe because we're single-threaded, and won't start reporting
        // validation errors until we're well past initialization.
        unsafe {
            if ERRORS.is_none() {
                ERRORS = Some(Errors::default());
            }
            match ERRORS {
                Some(ref mut errors) => errors,
                None => unreachable!(),
            }
        }
    }
}

pub fn set_vanilla_root(root: PathBuf) {
    Errors::get_mut().vanilla_root = root;
}

pub fn set_mod_root(root: PathBuf) {
    Errors::get_mut().mod_root = root;
}

pub fn error(token: &Token, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(token, ErrorLevel::Error, key, msg, None);
}

pub fn error_info(token: &Token, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(token, ErrorLevel::Error, key, msg, Some(info));
}

pub fn warn(token: &Token, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(token, ErrorLevel::Warning, key, msg, None);
}

pub fn warn_info(token: &Token, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(token, ErrorLevel::Warning, key, msg, Some(info));
}

pub fn advice(token: &Token, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(token, ErrorLevel::Advice, key, msg, None);
}

pub fn advice_info(token: &Token, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(token, ErrorLevel::Advice, key, msg, Some(info));
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
