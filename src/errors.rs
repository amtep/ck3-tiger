use encoding::all::{UTF_8, WINDOWS_1252};
use encoding::{DecoderTrap, Encoding};
use fnv::{FnvHashMap, FnvHashSet};
use std::fmt::{Debug, Display, Formatter};
use std::fs::read;
use std::io::{stdout, Stderr, Stdout, Write};
use std::path::PathBuf;
use unicode_width::UnicodeWidthChar;

use crate::block::{Block, BV};
use crate::errorkey::ErrorKey;
use crate::fileset::{FileEntry, FileKind};
use crate::token::{Loc, Token};

static mut ERRORS: Option<Errors> = None;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum ErrorLevel {
    #[default]
    Advice,
    Info,
    Warning,
    Error,
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

impl ErrorLoc for BV {
    fn into_loc(self) -> Loc {
        match self {
            BV::Value(t) => t.into_loc(),
            BV::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for &BV {
    fn into_loc(self) -> Loc {
        match self {
            BV::Value(t) => t.into_loc(),
            BV::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_entry(&self)
    }
}

impl ErrorLoc for &FileEntry {
    fn into_loc(self) -> Loc {
        Loc::for_entry(self)
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

type ErrorRecord = (Loc, ErrorKey, String, Option<Loc>, Option<Loc>);

#[derive(Default)]
struct Errors {
    /// The CK3 game directory
    vanilla_root: PathBuf,

    /// Extra loaded mods' directories
    loaded_mods: Vec<PathBuf>,

    /// Extra loaded mods' error tags
    loaded_mods_labels: Vec<String>,

    /// The mod directory
    mod_root: PathBuf,

    /// Don't log if this is > 0,
    logging_paused: isize,

    /// Whether to log errors in vanilla CK3 files
    show_vanilla: bool,

    /// Whether to log errors in other loaded mods
    show_loaded_mods: bool,

    /// Skip logging errors with these keys for these files and directories
    ignore_keys_for: FnvHashMap<PathBuf, Vec<ErrorKey>>,

    /// Skip logging errors with these keys
    ignore_keys: Vec<ErrorKey>,

    /// Skip logging errors for these files and directories (regardless of key)
    ignore_paths: Vec<PathBuf>,

    /// Error logs are written here (initially stderr)
    outfile: Option<Box<dyn ErrorLogger>>,

    /// Minimum error level to log
    minimum_level: ErrorLevel,

    /// Errors that have already been logged (to avoid duplication, which is common
    /// when validating macro expanded triggers and effects)
    seen: FnvHashSet<ErrorRecord>,

    filecache: FnvHashMap<PathBuf, String>,
}

impl Errors {
    fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = match loc.kind {
            FileKind::Vanilla => self.vanilla_root.join(&*loc.pathname),
            FileKind::LoadedMod(idx) => self.loaded_mods[idx as usize].join(&*loc.pathname),
            FileKind::Mod => self.mod_root.join(&*loc.pathname),
        };
        if let Some(contents) = self.filecache.get(&pathname) {
            return contents.lines().nth(loc.line - 1).map(str::to_string);
        }
        let bytes = read(&pathname).ok()?;
        let contents = match UTF_8.decode(&bytes, DecoderTrap::Strict) {
            Ok(contents) => contents,
            Err(_) => WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()?,
        };
        let line = contents.lines().nth(loc.line - 1).map(str::to_string);
        self.filecache.insert(pathname, contents);
        line
    }

    pub fn will_log(&self, loc: &Loc, key: ErrorKey) -> bool {
        // Check all elements of the loc link chain.
        // This is necessary because of cases like a mod passing `CHARACTER = this` to a vanilla script effect
        // that does not expect that. The error would be located in the vanilla script but would be caused by the mod.
        if let Some(loc) = &loc.link {
            if self.will_log(loc, key) {
                return true;
            }
        }
        if self.logging_paused > 0
            || self.ignore_keys.contains(&key)
            || (loc.kind == FileKind::Vanilla && !self.show_vanilla)
            || (matches!(loc.kind, FileKind::LoadedMod(_)) && !self.show_loaded_mods)
        {
            return false;
        }
        for (path, keys) in &self.ignore_keys_for {
            if loc.pathname.starts_with(path) && keys.contains(&key) {
                return false;
            }
        }
        for path in &self.ignore_paths {
            if loc.pathname.starts_with(path) {
                return false;
            }
        }
        true
    }

    pub fn log(
        &mut self,
        loc: &Loc,
        level: ErrorLevel,
        key: ErrorKey,
        msg: &str,
        info: Option<&str>,
    ) {
        if self.outfile.is_none() {
            self.outfile = Some(Box::new(stdout()));
        }
        let marker = self.loc_file_marker(loc);
        writeln!(self.outfile.as_mut().expect("outfile"), "{marker}").expect("writeln");
        if let Some(line) = self.get_line(loc) {
            let line_marker = loc.line_marker();
            if loc.line > 0 {
                writeln!(
                    self.outfile.as_mut().expect("outfile"),
                    "{line_marker} {line}"
                )
                .expect("writeln");
                let mut spacing = String::new();
                for c in line.chars().take(loc.column.saturating_sub(1)) {
                    if c == '\t' {
                        spacing.push('\t');
                    } else {
                        for _ in 0..c.width().unwrap_or(0) {
                            spacing.push(' ');
                        }
                    }
                }
                writeln!(
                    self.outfile.as_mut().expect("outfile"),
                    "{line_marker} {spacing}^",
                )
                .expect("writeln");
            }
        }
        writeln!(
            self.outfile.as_mut().expect("outfile"),
            "{level} ({key}): {msg}"
        )
        .expect("writeln");
        if let Some(info) = info {
            writeln!(self.outfile.as_mut().expect("outfile"), "  {info}").expect("writeln");
        }
        if let Some(link) = &loc.link {
            self.log(link, level, key, "from here", None);
        }
    }

    pub fn log_abbreviated(&mut self, loc: &Loc, key: ErrorKey) {
        if self.outfile.is_none() {
            self.outfile = Some(Box::new(stdout()));
        }
        if loc.line == 0 {
            writeln!(
                self.outfile.as_mut().expect("outfile"),
                "({key}) {}",
                loc.pathname.to_string_lossy()
            )
            .expect("writeln");
        } else if let Some(line) = self.get_line(loc) {
            writeln!(self.outfile.as_mut().expect("outfile"), "({key}) {line}").expect("writeln");
        }
    }

    #[allow(clippy::similar_names)] // eloc and loc are perfectly clear
    pub fn push<E: ErrorLoc>(
        &mut self,
        eloc: E,
        level: ErrorLevel,
        key: ErrorKey,
        msg: &str,
        info: Option<&str>,
    ) {
        if level < self.minimum_level {
            return;
        }
        let loc = eloc.into_loc();
        let index = (loc.clone(), key, msg.to_string(), None, None);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        if !self.will_log(&loc, key) {
            return;
        }
        self.log(&loc, level, key, msg, info);
        writeln!(self.outfile.as_mut().expect("outfile")).expect("writeln");
    }

    #[allow(clippy::similar_names)] // eloc and loc are perfectly clear
    pub fn push2<E: ErrorLoc, E2: ErrorLoc>(
        &mut self,
        eloc: E,
        level: ErrorLevel,
        key: ErrorKey,
        msg: &str,
        eloc2: E2,
        msg2: &str,
    ) {
        if level < self.minimum_level {
            return;
        }
        let loc = eloc.into_loc();
        let loc2 = eloc2.into_loc();
        let index = (loc.clone(), key, msg.to_string(), Some(loc2.clone()), None);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        if !self.will_log(&loc, key) {
            return;
        }
        self.log(&loc, level, key, msg, None);
        self.log(&loc2, ErrorLevel::Info, key, msg2, None);
        writeln!(self.outfile.as_mut().expect("outfile")).expect("writeln");
    }

    #[allow(clippy::similar_names)] // eloc and loc are perfectly clear
    #[allow(clippy::too_many_arguments)]
    pub fn push3<E: ErrorLoc, E2: ErrorLoc, E3: ErrorLoc>(
        &mut self,
        eloc: E,
        level: ErrorLevel,
        key: ErrorKey,
        msg: &str,
        eloc2: E2,
        msg2: &str,
        eloc3: E3,
        msg3: &str,
    ) {
        if level < self.minimum_level {
            return;
        }
        let loc = eloc.into_loc();
        let loc2 = eloc2.into_loc();
        let loc3 = eloc3.into_loc();
        let index = (
            loc.clone(),
            key,
            msg.to_string(),
            Some(loc2.clone()),
            Some(loc3.clone()),
        );
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        if !self.will_log(&loc, key) {
            return;
        }
        self.log(&loc, level, key, msg, None);
        self.log(&loc2, ErrorLevel::Info, key, msg2, None);
        self.log(&loc3, ErrorLevel::Info, key, msg3, None);
        writeln!(self.outfile.as_mut().expect("outfile")).expect("writeln");
    }

    pub fn push_abbreviated<E: ErrorLoc>(&mut self, eloc: E, level: ErrorLevel, key: ErrorKey) {
        if level < self.minimum_level {
            return;
        }
        let loc = eloc.into_loc();
        let index = (loc.clone(), key, String::new(), None, None);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        if !self.will_log(&loc, key) {
            return;
        }
        self.log_abbreviated(&loc, key);
    }

    pub fn push_header(&mut self, level: ErrorLevel, key: ErrorKey, msg: &str) {
        if level < self.minimum_level || self.logging_paused > 0 || self.ignore_keys.contains(&key)
        {
            return;
        }
        writeln!(self.outfile.as_mut().expect("outfile"), "{msg}").expect("writeln");
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

    fn loc_file_marker(&self, loc: &Loc) -> String {
        format!(
            "[{}] file {}",
            self.kind_tag(loc.kind),
            loc.pathname.display()
        )
    }

    fn kind_tag(&self, kind: FileKind) -> &str {
        match kind {
            FileKind::Vanilla => "CK3",
            FileKind::LoadedMod(idx) => &self.loaded_mods_labels[idx as usize],
            FileKind::Mod => "MOD",
        }
    }
}

pub fn log_to(outfile: Box<dyn ErrorLogger>) {
    Errors::get_mut().outfile = Some(outfile);
}

/// # Panics
/// Can panic if it is called without a previous `log_to()` call.
pub fn take_log_to() -> Box<dyn ErrorLogger> {
    Errors::get_mut().outfile.take().expect("outfile")
}

pub fn pause_logging() {
    Errors::get_mut().logging_paused += 1;
}

pub fn resume_logging() {
    Errors::get_mut().logging_paused -= 1;
}

pub fn show_vanilla(v: bool) {
    Errors::get_mut().show_vanilla = v;
}

pub fn show_loaded_mods(v: bool) {
    Errors::get_mut().show_loaded_mods = v;
}

pub fn minimum_level(lvl: ErrorLevel) {
    Errors::get_mut().minimum_level = lvl;
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

pub fn add_loaded_mod_root(label: String, root: PathBuf) {
    Errors::get_mut().loaded_mods_labels.push(label);
    Errors::get_mut().loaded_mods.push(root);
}

pub fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Error, key, msg, None);
}

pub fn error2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    Errors::get_mut().push2(eloc, ErrorLevel::Error, key, msg, eloc2, msg2);
}

pub fn error3<E: ErrorLoc, E2: ErrorLoc, E3: ErrorLoc>(
    eloc: E,
    key: ErrorKey,
    msg: &str,
    eloc2: E2,
    msg2: &str,
    eloc3: E3,
    msg3: &str,
) {
    Errors::get_mut().push3(eloc, ErrorLevel::Error, key, msg, eloc2, msg2, eloc3, msg3);
}

pub fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    Errors::get_mut().push(eloc, ErrorLevel::Error, key, msg, info);
}

pub fn warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Warning, key, msg, None);
}

pub fn warn2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    Errors::get_mut().push2(eloc, ErrorLevel::Warning, key, msg, eloc2, msg2);
}

pub fn warn3<E: ErrorLoc, E2: ErrorLoc, E3: ErrorLoc>(
    eloc: E,
    key: ErrorKey,
    msg: &str,
    eloc2: E2,
    msg2: &str,
    eloc3: E3,
    msg3: &str,
) {
    Errors::get_mut().push3(
        eloc,
        ErrorLevel::Warning,
        key,
        msg,
        eloc2,
        msg2,
        eloc3,
        msg3,
    );
}

pub fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    Errors::get_mut().push(eloc, ErrorLevel::Warning, key, msg, info);
}

pub fn info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Info, key, msg, None);
}

pub fn info_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    Errors::get_mut().push(eloc, ErrorLevel::Info, key, msg, info);
}

pub fn advice<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    Errors::get_mut().push(eloc, ErrorLevel::Advice, key, msg, None);
}

pub fn advice2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    Errors::get_mut().push2(eloc, ErrorLevel::Advice, key, msg, eloc2, msg2);
}

pub fn advice_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    Errors::get_mut().push(eloc, ErrorLevel::Advice, key, msg, info);
}

pub fn warn_header(key: ErrorKey, msg: &str) {
    Errors::get_mut().push_header(ErrorLevel::Warning, key, msg);
}

pub fn warn_abbreviated<E: ErrorLoc>(eloc: E, key: ErrorKey) {
    Errors::get_mut().push_abbreviated(eloc, ErrorLevel::Warning, key);
}

pub fn ignore_key_for(path: PathBuf, key: ErrorKey) {
    Errors::get_mut()
        .ignore_keys_for
        .entry(path)
        .or_default()
        .push(key);
}

/// Ignore this key for all files
pub fn ignore_key(key: ErrorKey) {
    Errors::get_mut().ignore_keys.push(key);
}

/// Ignore this path for all keys
pub fn ignore_path(path: PathBuf) {
    Errors::get_mut().ignore_paths.push(path);
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

impl ErrorLogger for Stdout {
    fn get_logs(&self) -> Option<String> {
        None
    }
}

impl ErrorLogger for Vec<u8> {
    fn get_logs(&self) -> Option<String> {
        Some(String::from_utf8_lossy(self).to_string())
    }
}
