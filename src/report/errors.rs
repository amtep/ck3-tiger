use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs::{read, File};
use std::io::{stdout, Write};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use anyhow::Result;
use encoding::all::{UTF_8, WINDOWS_1252};
use encoding::{DecoderTrap, Encoding};
use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;

use crate::fileset::FileKind;
use crate::report::error_loc::ErrorLoc;
use crate::report::filter::ReportFilter;
use crate::report::writer::log_report;
use crate::report::{
    err, tips, warn, ErrorKey, FilterRule, LogReport, OutputStyle, PointedMessage,
};
use crate::token::Loc;

static ERRORS: Lazy<Mutex<Errors>> = Lazy::new(|| Mutex::new(Errors::default()));

type ErrorRecord = (Loc, ErrorKey, String, Option<Loc>, Option<Loc>);

#[allow(missing_debug_implementations)]
pub struct Errors {
    pub(crate) output: RefCell<Box<dyn Write + Send>>,

    /// The base game directory
    vanilla_root: PathBuf,
    /// Extra base game directory loaded before `vanilla_root`
    clausewitz_root: PathBuf,
    /// Extra base game directory loaded before `vanilla_root`
    jomini_root: PathBuf,
    /// The mod directory
    mod_root: PathBuf,
    /// Extra loaded mods' directories
    loaded_mods: Vec<PathBuf>,
    /// Extra loaded mods' error tags
    pub(crate) loaded_mods_labels: Vec<String>,

    /// Errors that have already been logged (to avoid duplication, which is common
    /// when validating macro expanded triggers and effects)
    seen: FnvHashSet<ErrorRecord>,

    filecache: FnvHashMap<PathBuf, String>,

    /// Determines whether a report should be printed.
    pub(crate) filter: ReportFilter,
    /// Output color and style configuration.
    pub(crate) styles: OutputStyle,
    pub(crate) max_line_length: Option<usize>,

    /// All reports that passed the checks, stored here to be sorted before being emitted all at once.
    /// The "abbreviated" reports don't participate in this. They are still emitted immediately.
    storage: Vec<LogReport>,
}

impl Default for Errors {
    fn default() -> Self {
        Errors {
            output: RefCell::new(Box::new(stdout())),
            vanilla_root: PathBuf::default(),
            clausewitz_root: PathBuf::default(),
            jomini_root: PathBuf::default(),
            mod_root: PathBuf::default(),
            loaded_mods: Vec::default(),
            loaded_mods_labels: Vec::default(),
            seen: FnvHashSet::default(),
            filecache: FnvHashMap::default(),
            filter: ReportFilter::default(),
            styles: OutputStyle::default(),
            max_line_length: None,
            storage: Vec::default(),
        }
    }
}

impl Errors {
    pub(crate) fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = match loc.kind {
            FileKind::Internal => loc.pathname().to_path_buf(),
            FileKind::Clausewitz => self.clausewitz_root.join(loc.pathname()),
            FileKind::Jomini => self.jomini_root.join(loc.pathname()),
            FileKind::Vanilla => self.vanilla_root.join(loc.pathname()),
            FileKind::LoadedMod(idx) => self.loaded_mods[idx as usize].join(loc.pathname()),
            FileKind::Mod => self.mod_root.join(loc.pathname()),
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

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will do so.
    fn push_report(&mut self, report: LogReport) {
        if !self.filter.should_print_report(&report) {
            return;
        }
        let loc = report.primary().loc.clone();
        let loc2 = report.pointers.get(1).map(|p| p.loc.clone());
        let loc3 = report.pointers.get(2).map(|p| p.loc.clone());
        let index = (loc, report.key, report.msg.to_string(), loc2, loc3);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        self.storage.push(report);
    }

    pub fn log_abbreviated(&mut self, loc: &Loc, key: ErrorKey) {
        if loc.line == 0 {
            _ = writeln!(self.output.get_mut(), "({key}) {}", loc.pathname().to_string_lossy());
        } else if let Some(line) = self.get_line(loc) {
            _ = writeln!(self.output.get_mut(), "({key}) {line}");
        }
    }

    pub fn push_abbreviated<E: ErrorLoc>(&mut self, eloc: E, key: ErrorKey) {
        let loc = eloc.into_loc();
        let index = (loc.clone(), key, String::new(), None, None);
        if self.seen.contains(&index) {
            return;
        }
        self.seen.insert(index);
        self.log_abbreviated(&loc, key);
    }

    pub fn push_header(&mut self, _key: ErrorKey, msg: &str) {
        _ = writeln!(self.output.get_mut(), "{msg}");
    }

    pub fn emit_reports(&mut self) {
        let mut reports = take(&mut self.storage);
        reports.sort_unstable_by(|a, b| {
            // Severity in descending order
            let mut cmp = b.severity.cmp(&a.severity);
            if cmp != Ordering::Equal {
                return cmp;
            }
            // Confidence in descending order too
            cmp = b.confidence.cmp(&a.confidence);
            if cmp != Ordering::Equal {
                return cmp;
            }
            // If severity and confidence are the same, order by loc. Check all locs in order.
            for (a, b) in a.pointers.iter().zip(b.pointers.iter()) {
                cmp = a.loc.cmp(&b.loc);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            // Shorter chain goes first, if it comes to that.
            cmp = b.pointers.len().cmp(&a.pointers.len());
            if cmp != Ordering::Equal {
                return cmp;
            }
            // Fallback: order by message text.
            if cmp == Ordering::Equal {
                cmp = a.msg.cmp(&b.msg)
            }
            cmp
        });
        for report in &reports {
            log_report(self, report);
        }
    }

    pub fn get_mut() -> MutexGuard<'static, Errors> {
        ERRORS.lock().unwrap()
    }

    pub fn get() -> MutexGuard<'static, Errors> {
        ERRORS.lock().unwrap()
    }
}

pub fn set_vanilla_dir(dir: PathBuf) {
    let mut game = dir.clone();
    game.push("game");
    Errors::get_mut().vanilla_root = game;

    let mut clausewitz = dir.clone();
    clausewitz.push("clausewitz");
    Errors::get_mut().clausewitz_root = clausewitz;

    let mut jomini = dir;
    jomini.push("jomini");
    Errors::get_mut().jomini_root = jomini;
}

pub fn set_mod_root(root: PathBuf) {
    Errors::get_mut().mod_root = root;
}

pub fn add_loaded_mod_root(label: String, root: PathBuf) {
    let mut errors = Errors::get_mut();
    errors.loaded_mods_labels.push(label);
    errors.loaded_mods.push(root);
}

pub fn set_output_file(file: &Path) -> Result<()> {
    let file = File::create(file)?;
    Errors::get_mut().output = RefCell::new(Box::new(file));
    Ok(())
}

pub fn log(mut report: LogReport) {
    let mut vec = Vec::new();
    report.pointers.drain(..).for_each(|pointer| {
        let index = vec.len();
        recursive_pointed_msg_expansion(&mut vec, &pointer);
        vec.insert(index, pointer);
    });
    report.pointers.extend(vec);
    Errors::get_mut().push_report(report);
}

/// Expand `PointedMessage` recursively.
/// That is; for the given `PointedMessage`, follow its location's link until such link is no
/// longer available, adding a newly created `PointedMessage` to the given `Vec` for each linked
/// location.
fn recursive_pointed_msg_expansion(vec: &mut Vec<PointedMessage>, pointer: &PointedMessage) {
    if let Some(link) = &pointer.loc.link {
        let from_here = PointedMessage {
            loc: link.as_ref().into_loc(),
            length: 1,
            msg: Some("from here".to_owned()),
        };
        let index = vec.len();
        recursive_pointed_msg_expansion(vec, &from_here);
        vec.insert(index, from_here);
    }
}

/// Tests whether the report might be printed. If false, the report will definitely not be printed.
pub fn will_maybe_log<E: ErrorLoc>(eloc: E, key: ErrorKey) -> bool {
    Errors::get().filter.should_maybe_print(key, &eloc.into_loc())
}

pub fn emit_reports() {
    Errors::get_mut().emit_reports();
}

// =================================================================================================
// =============== Deprecated legacy calls to submit reports:
// =================================================================================================

pub fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    err(key).msg(msg).loc(eloc).push();
}

pub fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    err(key).msg(msg).info(info).loc(eloc).push();
}

pub fn old_warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    warn(key).msg(msg).loc(eloc).push();
}

pub fn warn2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    warn(key).msg(msg).loc(eloc).loc(eloc2, msg2).push();
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
    warn(key).msg(msg).loc(eloc).loc(eloc2, msg2).loc(eloc3, msg3).push();
}

pub fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    warn(key).msg(msg).info(info).loc(eloc).push();
}

pub fn advice<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    tips(key).msg(msg).loc(eloc).push();
}

pub fn advice2<E: ErrorLoc, F: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, eloc2: F, msg2: &str) {
    tips(key).msg(msg).loc(eloc).loc(eloc2, msg2).push();
}

pub fn advice_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    tips(key).msg(msg).info(info).loc(eloc).push();
}

pub fn warn_header(key: ErrorKey, msg: &str) {
    Errors::get_mut().push_header(key, msg);
}

pub fn warn_abbreviated<E: ErrorLoc>(eloc: E, key: ErrorKey) {
    Errors::get_mut().push_abbreviated(eloc, key);
}

// =================================================================================================
// =============== Configuration (Output style):
// =================================================================================================

/// Override the default `OutputStyle`. (Controls ansi colors)
pub fn set_output_style(style: OutputStyle) {
    Errors::get_mut().styles = style;
}

/// Disable color in the output.
pub fn disable_ansi_colors() {
    Errors::get_mut().styles = OutputStyle::no_color();
}

/// TODO:
pub fn set_max_line_length(max_line_length: usize) {
    Errors::get_mut().max_line_length =
        if max_line_length == 0 { None } else { Some(max_line_length) };
}

// =================================================================================================
// =============== Configuration (Filter):
// =================================================================================================

pub fn set_show_vanilla(v: bool) {
    Errors::get_mut().filter.show_vanilla = v;
}

pub fn set_show_loaded_mods(v: bool) {
    Errors::get_mut().filter.show_loaded_mods = v;
}

pub fn set_predicate(predicate: FilterRule) {
    Errors::get_mut().filter.predicate = predicate;
}
