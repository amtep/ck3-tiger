use encoding::all::{UTF_8, WINDOWS_1252};
use encoding::{DecoderTrap, Encoding};
use fnv::FnvHashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs::read;
use std::io::{stderr, Stderr, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::block::{Block, BlockOrValue};
use crate::errorkey::ErrorKey;
use crate::fileset::{FileEntry, FileKind};
use crate::token::{Loc, Token};

static mut ERRORS: Option<Errors> = None;

#[derive(Clone, Copy, Debug)]
pub enum ErrorLevel {
    Error,
    Warning,
    Info,
    Advice,
}

impl Display for ErrorLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ErrorLevel::Error => write!(f, "ERROR"),
            ErrorLevel::Warning => write!(f, "WARNING"),
            ErrorLevel::Info => write!(f, "INFO"),
            ErrorLevel::Advice => write!(f, "ADVICE"),
        }
    }
}

// This trait lets the error functions accept a variety of things as the error locator.
pub trait ErrorLoc {
    fn into_loc(self) -> Loc;
}

impl ErrorLoc for (PathBuf, FileKind) {
    fn into_loc(self) -> Loc {
        Loc::for_file(Rc::new(self.0), self.1)
    }
}

impl ErrorLoc for (&Path, FileKind) {
    fn into_loc(self) -> Loc {
        Loc::for_file(Rc::new(self.0.to_path_buf()), self.1)
    }
}

impl ErrorLoc for BlockOrValue {
    fn into_loc(self) -> Loc {
        match self {
            BlockOrValue::Token(t) => t.into_loc(),
            BlockOrValue::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for &BlockOrValue {
    fn into_loc(self) -> Loc {
        match self {
            BlockOrValue::Token(t) => t.into_loc(),
            BlockOrValue::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_file(Rc::new(self.path().to_path_buf()), self.kind())
    }
}

impl ErrorLoc for &FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_file(Rc::new(self.path().to_path_buf()), self.kind())
    }
}

impl ErrorLoc for Loc {
    fn into_loc(self) -> Loc {
        self
    }
}

impl ErrorLoc for &Loc {
    fn into_loc(self) -> Loc {
        self.clone()
    }
}

impl ErrorLoc for Token {
    fn into_loc(self) -> Loc {
        self.loc
    }
}

impl ErrorLoc for &Token {
    fn into_loc(self) -> Loc {
        self.loc.clone()
    }
}

impl ErrorLoc for Block {
    fn into_loc(self) -> Loc {
        self.loc
    }
}

impl ErrorLoc for &Block {
    fn into_loc(self) -> Loc {
        self.loc.clone()
    }
}

#[derive(Default)]
struct Errors {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// The mod directory
    mod_root: PathBuf,

    /// Don't log if this is > 0,
    logging_paused: isize,
    /// Unless never_pause is set
    never_pause: bool,

    /// Skip logging errors with these keys for these files
    ignore_keys_for: FnvHashMap<PathBuf, Vec<ErrorKey>>,

    /// Skip logging errors with these keys
    ignore_keys: Vec<ErrorKey>,

    /// Error logs are written here (initially stderr)
    outfile: Option<Box<dyn ErrorLogger>>,
}

// TODO: allow a message to have multiple tokens, and print the relevant lines as a stack
// before the message. This might be implemented by letting Token have something like an
// `Option<Token>` field to chain them.

impl Errors {
    #[allow(clippy::unused_self)] // At some point we will cache files in self
    fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = match loc.kind {
            FileKind::VanillaFile => self.vanilla_root.join(&*loc.pathname),
            FileKind::ModFile => self.mod_root.join(&*loc.pathname),
        };
        let bytes = read(&pathname).ok()?;
        let contents = match UTF_8.decode(&bytes, DecoderTrap::Strict) {
            Ok(contents) => contents,
            Err(_) => WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()?,
        };
        contents.lines().nth(loc.line - 1).map(str::to_string)
    }

    pub fn will_log(&self, loc: &Loc, key: ErrorKey) -> bool {
        if (self.logging_paused > 0 && !self.never_pause) || self.ignore_keys.contains(&key) {
            return false;
        }
        if let Some(true) = self
            .ignore_keys_for
            .get(&*loc.pathname)
            .map(|v| v.contains(&key))
        {
            return false;
        }
        true
    }

    #[allow(unused_must_use)] // If logging errors fails, there's not much we can do
    #[allow(clippy::similar_names)] // eloc and loc are perfectly clear
    pub fn push<E: ErrorLoc>(
        &mut self,
        eloc: E,
        level: ErrorLevel,
        key: ErrorKey,
        msg: &str,
        info: Option<&str>,
    ) {
        let loc = eloc.into_loc();
        if !self.will_log(&loc, key) {
            return;
        }
        if self.outfile.is_none() {
            self.outfile = Some(Box::new(stderr()));
        }
        if let Some(line) = self.get_line(&loc) {
            let line_marker = loc.line_marker();
            writeln!(self.outfile.as_mut().unwrap(), "{}{}", line_marker, line);
            // TODO: adjust the column count for tabs in the line
            writeln!(
                self.outfile.as_mut().unwrap(),
                "{}{:>count$}",
                line_marker,
                "^",
                count = loc.column
            );
        }
        // TODO: get terminal column width and do line wrapping of msg and info
        writeln!(
            self.outfile.as_mut().unwrap(),
            "{}{}: {}",
            loc.marker(),
            level,
            msg
        );
        if let Some(info) = info {
            writeln!(self.outfile.as_mut().unwrap(), "  {}", info);
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

    pub fn get() -> &'static Self {
        unsafe {
            if ERRORS.is_none() {
                ERRORS = Some(Errors::default());
            }
            match ERRORS {
                Some(ref errors) => errors,
                None => unreachable!(),
            }
        }
    }
}

pub fn log_to(outfile: Box<dyn ErrorLogger>) {
    Errors::get_mut().outfile = Some(outfile);
}

/// # Panics
/// Can panic if it is called without a previous `log_to()` call.
pub fn take_log_to() -> Box<dyn ErrorLogger> {
    Errors::get_mut().outfile.take().unwrap()
}

pub fn pause_logging() {
    Errors::get_mut().logging_paused += 1;
}

pub fn resume_logging() {
    Errors::get_mut().logging_paused -= 1;
}

pub fn never_pause() {
    Errors::get_mut().never_pause = true;
}

/// This is an object that can pause logging as long as it's in scope.
/// Whether it does to depends on its constructor's `pause` argument.
#[derive(Debug)]
pub struct LogPauseRaii {
    paused: bool,
}

impl LogPauseRaii {
    pub fn new(pause: bool) -> Self {
        if pause {
            pause_logging();
        }
        Self { paused: pause }
    }
}

impl Drop for LogPauseRaii {
    fn drop(&mut self) {
        if self.paused {
            resume_logging();
        }
    }
}

pub fn set_vanilla_root(root: PathBuf) {
    Errors::get_mut().vanilla_root = root;
}

pub fn set_mod_root(root: PathBuf) {
    Errors::get_mut().mod_root = root;
}

pub fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Error, key, msg, None);
}

pub fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Error, key, msg, Some(info));
}

pub fn warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Warning, key, msg, None);
}

pub fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Warning, key, msg, Some(info));
}

pub fn info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Info, key, msg, None);
}

pub fn info_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Info, key, msg, Some(info));
}

pub fn advice<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Advice, key, msg, None);
}

pub fn advice_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Advice, key, msg, Some(info));
}

pub fn ignore_key_for(path: PathBuf, key: ErrorKey) {
    Errors::get_mut()
        .ignore_keys_for
        .entry(path)
        .or_default()
        .push(key);
}

pub fn ignore_key(key: ErrorKey) {
    Errors::get_mut().ignore_keys.push(key);
}

pub fn will_log<E: ErrorLoc>(eloc: E, key: ErrorKey) -> bool {
    Errors::get().will_log(&eloc.into_loc(), key)
}

pub trait ErrorLogger: Write {
    fn get_logs(&self) -> Option<String>;
}

impl ErrorLogger for Stderr {
    fn get_logs(&self) -> Option<String> {
        None
    }
}

impl ErrorLogger for Vec<u8> {
    fn get_logs(&self) -> Option<String> {
        Some(String::from_utf8_lossy(self).to_string())
    }
}
