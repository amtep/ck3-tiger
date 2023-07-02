use std::fmt::{Debug, Display, Formatter};
use std::fs::read;
use std::io::{stdout, Stderr, Stdout, Write};
use std::path::PathBuf;

use ansi_term::{ANSIString, ANSIStrings};
use encoding::all::{UTF_8, WINDOWS_1252};
use encoding::{DecoderTrap, Encoding};
use fnv::{FnvHashMap, FnvHashSet};
use strum_macros::EnumIter;
use unicode_width::UnicodeWidthChar;

use crate::block::{Block, BV};
use crate::error::{Confidence, LogLevel, LogReport, PointedMessage, Severity};
use crate::errorkey::ErrorKey;
use crate::fileset::{FileEntry, FileKind};
use crate::output_style::{OutputStyle, Styled};
use crate::token::{Loc, Token};

static mut ERRORS: Option<Errors> = None;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, PartialEq, Hash, EnumIter)]
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

    /// Extra CK3 directory loaded before `vanilla_root`
    clausewitz_root: PathBuf,

    /// Extra CK3 directory loaded before `vanilla_root`
    jomini_root: PathBuf,

    /// Extra loaded mods' directories
    loaded_mods: Vec<PathBuf>,

    /// Extra loaded mods' error tags
    loaded_mods_labels: Vec<String>,

    /// The mod directory
    mod_root: PathBuf,

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
    min_level: LogLevel,

    /// Errors that have already been logged (to avoid duplication, which is common
    /// when validating macro expanded triggers and effects)
    seen: FnvHashSet<ErrorRecord>,

    filecache: FnvHashMap<PathBuf, String>,

    /// Output color and style configuration.
    styles: OutputStyle,
}

impl Errors {
    fn outfile(&mut self) -> &mut dyn Write {
        self.outfile.as_mut().expect("outfile")
    }
    fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = match loc.kind {
            FileKind::Internal => (*loc.pathname).clone(),
            FileKind::Clausewitz => self.clausewitz_root.join(&*loc.pathname),
            FileKind::Jomini => self.jomini_root.join(&*loc.pathname),
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
        if self.ignore_keys.contains(&key)
            || (loc.kind <= FileKind::Vanilla && !self.show_vanilla)
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

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will do so.
    fn push_report(&mut self, report: LogReport) {
        if self.outfile.is_none() {
            // TODO: should this be evaluated every time? Can it not happen once at the start?
            self.outfile = Some(Box::new(stdout()));
        }
        if report.lvl.severity < self.min_level.severity
            || report.lvl.confidence < self.min_level.confidence
        {
            return;
        }
        // TODO: Re-implement 'seen'
        // let loc = eloc.into_loc();
        // let loc2 = eloc2.into_loc();
        // let index = (loc.clone(), key, msg.to_string(), Some(loc2.clone()), None);
        // if self.seen.contains(&index) {
        //     return;
        // }
        // self.seen.insert(index);
        if !self.will_log(&report.primary().location, report.key) {
            return;
        }
        self.log_report(report);
    }

    /// Log the report.
    fn log_report(&mut self, report: LogReport) {
        // Log error lvl and message:
        self.log_line_title(&report);
        // Log the primary pointer:
        self.log_pointer_primary(report.primary(), report.indentation(), report.lvl.severity);
        // Log the other pointers:
        report.pointers.windows(2).for_each(|pointers| {
            self.log_pointer(
                pointers.get(0).expect("Must exist."),
                pointers.get(1).expect("Must exist."),
                report.indentation(),
                report.lvl.severity,
            );
        });
        if let Some(info) = report.info {
            let line_info: &[ANSIString<'static>] = &[
                self.styles.style(&Styled::Default).paint(format!(
                    "{:width$}",
                    "",
                    width = report.indentation()
                )),
                self.styles.style(&Styled::Default).paint(" "),
                self.styles.style(&Styled::Location).paint("="),
                self.styles.style(&Styled::Default).paint(" "),
                self.styles.style(&Styled::InfoTag).paint("Info:"),
                self.styles.style(&Styled::Default).paint(" "),
                self.styles.style(&Styled::Info).paint(format!("{info}")),
            ];
            writeln!(self.outfile(), "{}", ANSIStrings(line_info)).expect("writeln");
        }

        // if let Some(link) = &loc.link {
        //     self.log(link, level, key, "from here", None);
        // }

        // Write a blank line to visually separate reports:
        writeln!(self.outfile.as_mut().expect("outfile"), "").expect("writeln");
    }
    fn log_pointer_primary(
        &mut self,
        pointer: &PointedMessage,
        indentation: usize,
        severity: Severity,
    ) {
        self.log_line_file_location(pointer, indentation);
        if pointer.location.line == 0 {
            // Zero-length line means the location is an entire file,
            // not any particular location within the file.
            return;
        }
        if let Some(line) = self.get_line(&pointer.location) {
            self.log_line_from_source(pointer, indentation, &line);
            self.log_line_carets(pointer, &line, indentation, severity);
        }
    }
    fn log_pointer(
        &mut self,
        previous: &PointedMessage,
        pointer: &PointedMessage,
        indentation: usize,
        severity: Severity,
    ) {
        if previous.location.pathname != pointer.location.pathname
            || previous.location.kind != pointer.location.kind
        {
            // This pointer is not the same as the previous pointer. Print file location as well:
            self.log_line_file_location(pointer, indentation);
        } else {
            self.log_line_blank(indentation);
        }
        if pointer.location.line == 0 {
            // Zero-length line means the location is an entire file,
            // not any particular location within the file.
            return;
        }
        if let Some(line) = self.get_line(&pointer.location) {
            self.log_line_from_source(pointer, indentation, &line);
            self.log_line_carets(pointer, &line, indentation, severity);
        }
    }

    /// Log the first line of a report, containing the severity level and the error message.
    fn log_line_title(&mut self, report: &LogReport) {
        let line: &[ANSIString<'static>] = &[
            self.styles
                .style(&Styled::Tag(report.lvl.severity, true))
                .paint(format!("{}", report.lvl.severity)),
            self.styles
                .style(&Styled::Tag(report.lvl.severity, false))
                .paint("("),
            self.styles
                .style(&Styled::Tag(report.lvl.severity, false))
                .paint(format!("{}", report.key)),
            self.styles
                .style(&Styled::Tag(report.lvl.severity, false))
                .paint(")"),
            self.styles.style(&Styled::Default).paint(": "),
            self.styles
                .style(&Styled::ErrorMessage)
                .paint(format!("{}", report.msg)),
        ];
        writeln!(self.outfile(), "{}", ANSIStrings(line)).expect("writeln");
    }

    /// Log the line containing the location's mod name and filename.
    fn log_line_file_location(&mut self, pointer: &PointedMessage, indentation: usize) {
        let line_filename: &[ANSIString<'static>] = &[
            self.styles.style(&Styled::Default).paint(format!(
                "{:width$}",
                "",
                width = indentation
            )),
            self.styles.style(&Styled::Location).paint("-->"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles.style(&Styled::Location).paint("["),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", self.kind_tag(pointer.location.kind))),
            self.styles.style(&Styled::Location).paint("]"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", pointer.location.pathname.display())),
        ];
        writeln!(self.outfile(), "{}", ANSIStrings(line_filename)).expect("writeln");
    }

    /// Print a line from the source file.
    fn log_line_from_source(&mut self, pointer: &PointedMessage, indentation: usize, line: &str) {
        let line_from_source: &[ANSIString<'static>] = &[
            self.styles.style(&Styled::Location).paint(format!(
                "{:width$}",
                pointer.location.line,
                width = indentation
            )),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles.style(&Styled::Location).paint("|"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles
                .style(&Styled::SourceText)
                .paint(format!("{line}")),
        ];
        writeln!(self.outfile(), "{}", ANSIStrings(line_from_source)).expect("writeln");
    }

    fn log_line_carets(
        &mut self,
        pointer: &PointedMessage,
        line: &str,
        indentation: usize,
        severity: Severity,
    ) {
        let mut spacing = String::new();
        for c in line.chars().take(pointer.location.column.saturating_sub(1)) {
            if c == '\t' {
                // spacing.push_str("  ");
                spacing.push('\t');
            } else {
                for _ in 0..c.width().unwrap_or(0) {
                    spacing.push(' ');
                }
            }
        }
        // A line containing the carets that point upwards at the source line.
        let line_carets: &[ANSIString<'static>] = &[
            self.styles.style(&Styled::Default).paint(format!(
                "{:width$}",
                "",
                width = indentation
            )),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles.style(&Styled::Location).paint("|"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles
                .style(&Styled::Default)
                .paint(format!("{spacing}")),
            self.styles
                .style(&Styled::Tag(severity, true))
                .paint(format!("{:^^width$}", "", width = pointer.length)),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles
                .style(&Styled::Tag(severity, true))
                .paint(format!(
                    "{}",
                    pointer.msg.as_ref().map(|_| "<-- ").unwrap_or(&"")
                )),
            self.styles
                .style(&Styled::Tag(severity, true))
                .paint(format!("{}", pointer.msg.as_ref().unwrap_or(&""))),
        ];
        writeln!(self.outfile(), "{}", ANSIStrings(line_carets)).expect("writeln");
    }

    /// Print a blank line to represent space between two lines in the same file.
    fn log_line_blank(&mut self, indentation: usize) {
        let line_blank: &[ANSIString<'static>] = &[
            self.styles
                .style(&Styled::Location)
                .paint("-".repeat(indentation)),
            self.styles.style(&Styled::Default).paint("   "),
        ];
        writeln!(self.outfile(), "{}", ANSIStrings(line_blank)).expect("writeln");
    }

    /// Deprecated in favour of log_report().
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
        let first_line: &[ANSIString<'static>] = &[
            self.styles
                .style(&Styled::TagOld(level, true))
                .paint(format!("{level}")),
            self.styles.style(&Styled::TagOld(level, false)).paint("("),
            self.styles
                .style(&Styled::TagOld(level, false))
                .paint(format!("{key}")),
            self.styles.style(&Styled::TagOld(level, false)).paint(")"),
            self.styles.style(&Styled::Default).paint(": "),
            self.styles
                .style(&Styled::ErrorMessage)
                .paint(format!("{msg}")),
        ];
        writeln!(
            self.outfile.as_mut().expect("outfile"),
            "{}",
            ANSIStrings(first_line)
        )
        .expect("writeln");

        let second_line: &[ANSIString<'static>] = &[
            self.styles.style(&Styled::Default).paint(format!(
                "{:width$}",
                "",
                width = loc.line.to_string().len()
            )),
            self.styles.style(&Styled::Location).paint("-->"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles.style(&Styled::Location).paint("["),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", self.kind_tag(loc.kind))),
            self.styles.style(&Styled::Location).paint("]"),
            self.styles.style(&Styled::Default).paint(" "),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", loc.pathname.display())),
            self.styles.style(&Styled::Location).paint(":"),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", loc.line)),
            self.styles.style(&Styled::Location).paint(":"),
            self.styles
                .style(&Styled::Location)
                .paint(format!("{}", loc.column)),
        ];
        writeln!(
            self.outfile.as_mut().expect("outfile"),
            "{}",
            ANSIStrings(second_line)
        )
        .expect("writeln");

        if let Some(line) = self.get_line(loc) {
            if loc.line > 0 {
                let third_line: &[ANSIString<'static>] = &[
                    self.styles
                        .style(&Styled::Location)
                        .paint(format!("{}", loc.line)),
                    self.styles.style(&Styled::Default).paint(" "),
                    self.styles.style(&Styled::Location).paint("|"),
                    self.styles.style(&Styled::Default).paint(" "),
                    self.styles
                        .style(&Styled::SourceText)
                        .paint(format!("{line}")),
                ];
                writeln!(
                    self.outfile.as_mut().expect("outfile"),
                    "{}",
                    ANSIStrings(third_line)
                )
                .expect("writeln");

                let mut spacing = String::new();
                for c in line.chars().take(loc.column.saturating_sub(1)) {
                    if c == '\t' {
                        // spacing.push_str("  ");
                        spacing.push('\t');
                    } else {
                        for _ in 0..c.width().unwrap_or(0) {
                            spacing.push(' ');
                        }
                    }
                }
                let third_line: &[ANSIString<'static>] = &[
                    self.styles.style(&Styled::Default).paint(format!(
                        "{:width$}",
                        "",
                        width = loc.line.to_string().len()
                    )),
                    self.styles.style(&Styled::Default).paint(" "),
                    self.styles.style(&Styled::Location).paint("|"),
                    self.styles.style(&Styled::Default).paint(" "),
                    self.styles
                        .style(&Styled::Default)
                        .paint(format!("{spacing}")),
                    self.styles.style(&Styled::TagOld(level, true)).paint("^"),
                ];
                writeln!(
                    self.outfile.as_mut().expect("outfile"),
                    "{}",
                    ANSIStrings(third_line)
                )
                .expect("writeln");
            }
        }
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
        if level < self.minimum_level || self.ignore_keys.contains(&key) {
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
            FileKind::Internal => "Internal",
            FileKind::Clausewitz => "Clausewitz",
            FileKind::Jomini => "Jomini",
            FileKind::Vanilla => "CK3",
            FileKind::LoadedMod(idx) => &self.loaded_mods_labels[idx as usize],
            FileKind::Mod => "MOD",
        }
    }
}

/// Exclusively used in tests. Deprecated?
pub fn log_to(outfile: Box<dyn ErrorLogger>) {
    Errors::get_mut().outfile = Some(outfile);
}

/// # Panics
/// Can panic if it is called without a previous `log_to()` call.
pub fn take_log_to() -> Box<dyn ErrorLogger> {
    Errors::get_mut().outfile.take().expect("outfile")
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

pub fn set_vanilla_dir(dir: PathBuf) {
    let mut game = dir.clone();
    game.push("game");
    Errors::get_mut().vanilla_root = game;

    let mut clausewitz = dir.clone();
    clausewitz.push("clausewitz");
    Errors::get_mut().clausewitz_root = clausewitz;

    let mut jomini = dir.clone();
    jomini.push("jomini");
    Errors::get_mut().jomini_root = jomini;
}

pub fn set_mod_root(root: PathBuf) {
    Errors::get_mut().mod_root = root;
}

pub fn add_loaded_mod_root(label: String, root: PathBuf) {
    Errors::get_mut().loaded_mods_labels.push(label);
    Errors::get_mut().loaded_mods.push(root);
}

pub fn log(report: LogReport) {
    Errors::get_mut().push_report(report);
}

pub fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Error, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    log(LogReport {
        lvl: LogLevel::new(Severity::Error, Confidence::Reasonable),
        key,
        msg,
        info,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn warn2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
        ],
    });
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
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
            PointedMessage {
                location: eloc3.into_loc(),
                length: 1,
                msg: Some(msg3),
            },
        ],
    });
}

pub fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    let info = if info.is_empty() { None } else { Some(info) };
    log(LogReport {
        lvl: LogLevel::new(Severity::Warning, Confidence::Reasonable),
        key,
        msg,
        info,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn advice<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Info, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![PointedMessage {
            location: eloc.into_loc(),
            length: 1,
            msg: None,
        }],
    });
}

pub fn advice2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    log(LogReport {
        lvl: LogLevel::new(Severity::Info, Confidence::Reasonable),
        key,
        msg,
        info: None,
        pointers: vec![
            PointedMessage {
                location: eloc.into_loc(),
                length: 1,
                msg: None,
            },
            PointedMessage {
                location: eloc2.into_loc(),
                length: 1,
                msg: Some(msg2),
            },
        ],
    });
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

/// Override the default OutputStyle. (Controls ansi colors)
pub fn set_output_style(style: OutputStyle) {
    Errors::get_mut().styles = style;
}

/// Disable color in the output.
pub fn disable_ansi_colors() {
    Errors::get_mut().styles = OutputStyle::no_color();
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
