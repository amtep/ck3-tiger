//! Collect error reports and then write them out.

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
use crate::report::writer_json::log_report_json;
use crate::report::{
    err, tips, warn, ErrorKey, FilterRule, LogReport, OutputStyle, PointedMessage,
};
use crate::token::Loc;

static ERRORS: Lazy<Mutex<Errors>> = Lazy::new(|| Mutex::new(Errors::default()));

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

    /// Files that have been read in to get the lines where errors occurred.
    /// Cached here to avoid duplicate I/O and UTF-8 parsing.
    filecache: FnvHashMap<PathBuf, String>,

    /// Determines whether a report should be printed.
    pub(crate) filter: ReportFilter,
    /// Output color and style configuration.
    pub(crate) styles: OutputStyle,
    /// Currently unused
    pub(crate) max_line_length: Option<usize>,

    /// All reports that passed the checks, stored here to be sorted before being emitted all at once.
    /// The "abbreviated" reports don't participate in this. They are still emitted immediately.
    /// It's a `HashSet` because duplicate reports are fairly common due to macro expansion and other revalidations.
    storage: FnvHashSet<LogReport>,
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
            filecache: FnvHashMap::default(),
            filter: ReportFilter::default(),
            styles: OutputStyle::default(),
            max_line_length: None,
            storage: FnvHashSet::default(),
        }
    }
}

impl Errors {
    /// Get the full filesystem path for a path that is anchored at one of the "virtual roots" of
    /// the game's VFS.
    pub(crate) fn get_fullpath(&mut self, kind: FileKind, path: &Path) -> PathBuf {
        match kind {
            FileKind::Internal => path.to_path_buf(),
            FileKind::Clausewitz => self.clausewitz_root.join(path),
            FileKind::Jomini => self.jomini_root.join(path),
            FileKind::Vanilla => self.vanilla_root.join(path),
            FileKind::LoadedMod(idx) => self.loaded_mods[idx as usize].join(path),
            FileKind::Mod => self.mod_root.join(path),
        }
    }

    /// Fetch the contents of a single line from a script file.
    pub(crate) fn get_line(&mut self, loc: &Loc) -> Option<String> {
        if loc.line == 0 {
            return None;
        }
        let pathname = self.get_fullpath(loc.kind, loc.pathname());
        if let Some(contents) = self.filecache.get(&pathname) {
            return contents.lines().nth(loc.line as usize - 1).map(str::to_string);
        }
        let bytes = read(&pathname).ok()?;
        let contents = match UTF_8.decode(&bytes, DecoderTrap::Strict) {
            Ok(contents) => contents,
            Err(_) => WINDOWS_1252.decode(&bytes, DecoderTrap::Strict).ok()?,
        };
        // Strip the BOM, if any
        #[allow(clippy::map_unwrap_or)] // borrow checker won't allow map_or here
        let contents = contents.strip_prefix('\u{feff}').map(str::to_string).unwrap_or(contents);
        let line = contents.lines().nth(loc.line as usize - 1).map(str::to_string);
        self.filecache.insert(pathname, contents);
        line
    }

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will add it to the storage.
    fn push_report(&mut self, report: LogReport) {
        if !self.filter.should_print_report(&report) {
            return;
        }
        self.storage.insert(report);
    }

    /// Immediately log a single-line report about this error.
    ///
    /// This is intended for voluminous almost-identical errors, such as from the "unused
    /// localization" check.
    // TODO: integrate this function into the error reporting framework.
    pub fn push_abbreviated<E: ErrorLoc>(&mut self, eloc: E, key: ErrorKey) {
        let loc = eloc.into_loc();
        if loc.line == 0 {
            _ = writeln!(self.output.get_mut(), "({key}) {}", loc.pathname().to_string_lossy());
        } else if let Some(line) = self.get_line(&loc) {
            _ = writeln!(self.output.get_mut(), "({key}) {line}");
        }
    }

    /// Immediately print an error message. It is intended to introduce a following block of
    /// messages printed with [`Errors::push_abbreviated`].
    // TODO: integrate this function into the error reporting framework.
    pub fn push_header(&mut self, _key: ErrorKey, msg: &str) {
        _ = writeln!(self.output.get_mut(), "{msg}");
    }

    /// Extract the stored reports, sort them, and return them as a vector of [`LogReport`].
    /// The stored reports will be left empty.
    pub fn take_reports(&mut self) -> Vec<LogReport> {
        let mut reports: Vec<LogReport> = take(&mut self.storage).into_iter().collect();
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
                cmp = a.msg.cmp(&b.msg);
            }
            cmp
        });
        reports
    }

    /// Print all the stored reports to the error output.
    /// Set `json` if they should be printed as a JSON array. Otherwise they are printed in the
    /// default output format.
    ///
    /// Note that the default output format is not stable across versions. It is meant for human
    /// readability and occasionally gets changed to improve that.
    pub fn emit_reports(&mut self, json: bool) {
        let reports = self.take_reports();
        if json {
            _ = writeln!(self.output.get_mut(), "[");
            let mut first = true;
            for report in &reports {
                if !first {
                    _ = writeln!(self.output.get_mut(), ",");
                }
                first = false;
                log_report_json(self, report);
            }
            _ = writeln!(self.output.get_mut(), "\n]");
        } else {
            for report in &reports {
                log_report(self, report);
            }
        }
    }

    /// Get a mutable lock on the global ERRORS struct.
    ///
    /// # Panics
    /// May panic when the mutex has been poisoned by another thread.
    pub fn get_mut() -> MutexGuard<'static, Errors> {
        ERRORS.lock().unwrap()
    }

    /// Like [`Errors::get_mut`] but intended for read-only access.
    ///
    /// Currently there is no difference, but if the locking mechanism changes there may be a
    /// difference.
    ///
    /// # Panics
    /// May panic when the mutex has been poisoned by another thread.
    pub fn get() -> MutexGuard<'static, Errors> {
        ERRORS.lock().unwrap()
    }
}

/// Record `dir` as the path to the base game files.
/// It should be a path to the directory containing the `game` directory.
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

/// Record `dir` as the path to the mod being validated.
pub fn set_mod_root(dir: PathBuf) {
    Errors::get_mut().mod_root = dir;
}

/// Record `dir` as the path to a secondary mod to be loaded before the one being validated.
/// `label` is what this mod should be called in the error reports; ideally only a few characters long.
pub fn add_loaded_mod_root(label: String, dir: PathBuf) {
    let mut errors = Errors::get_mut();
    errors.loaded_mods_labels.push(label);
    errors.loaded_mods.push(dir);
}

/// Configure the error reports to be written to this file instead of to stdout.
pub fn set_output_file(file: &Path) -> Result<()> {
    let file = File::create(file)?;
    Errors::get_mut().output = RefCell::new(Box::new(file));
    Ok(())
}

/// Store an error report to be emitted when [`emit_reports`] is called.
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
            length: 0,
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

/// Print all the stored reports to the error output.
/// Set `json` if they should be printed as a JSON array. Otherwise they are printed in the
/// default output format.
///
/// Note that the default output format is not stable across versions. It is meant for human
/// readability and occasionally gets changed to improve that.
pub fn emit_reports(json: bool) {
    Errors::get_mut().emit_reports(json);
}

/// Extract the stored reports, sort them, and return them as a vector of [`LogReport`].
/// The stored reports will be left empty.
pub fn take_reports() -> Vec<LogReport> {
    Errors::get_mut().take_reports()
}

// =================================================================================================
// =============== Deprecated legacy calls to submit reports:
// =================================================================================================

/// Deprecated. Use [`err`] instead.
pub(crate) fn error<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    err(key).msg(msg).loc(eloc).push();
}

/// Deprecated. Use [`err`] instead.
pub(crate) fn error_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    err(key).msg(msg).info(info).loc(eloc).push();
}

/// Deprecated. Use [`warn`] instead.
pub(crate) fn old_warn<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str) {
    warn(key).msg(msg).loc(eloc).push();
}

/// Deprecated. Use [`warn`] instead.
pub(crate) fn warn2<E: ErrorLoc, F: ErrorLoc>(
    eloc: E,
    key: ErrorKey,
    msg: &str,
    eloc2: F,
    msg2: &str,
) {
    warn(key).msg(msg).loc(eloc).loc(eloc2, msg2).push();
}

/// Deprecated. Use [`warn`] instead.
pub(crate) fn warn3<E: ErrorLoc, E2: ErrorLoc, E3: ErrorLoc>(
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

/// Deprecated. Use [`warn`] instead.
pub(crate) fn warn_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    warn(key).msg(msg).info(info).loc(eloc).push();
}

/// Deprecated. Use [`tips`] instead.
pub(crate) fn advice_info<E: ErrorLoc>(eloc: E, key: ErrorKey, msg: &str, info: &str) {
    tips(key).msg(msg).info(info).loc(eloc).push();
}

/// Immediately print an error message. It is intended to introduce a following block of
/// messages printed with [`warn_abbreviated`].
pub(crate) fn warn_header(key: ErrorKey, msg: &str) {
    Errors::get_mut().push_header(key, msg);
}

/// Immediately log a single-line report about this error.
///
/// This is intended for voluminous almost-identical errors, such as from the "unused
/// localization" check.
pub(crate) fn warn_abbreviated<E: ErrorLoc>(eloc: E, key: ErrorKey) {
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

/// Configure the error reporter to show errors that are in the base game code.
/// Normally those are filtered out, to only show errors that involve the mod's code.
pub fn set_show_vanilla(v: bool) {
    Errors::get_mut().filter.show_vanilla = v;
}

/// Configure the error reporter to show errors that are in extra loaded mods.
/// Normally those are filtered out, to only show errors that involve the mod's code.
pub fn set_show_loaded_mods(v: bool) {
    Errors::get_mut().filter.show_loaded_mods = v;
}

/// Configure the error reporter to only show errors that match this [`FilterRule`].
pub(crate) fn set_predicate(predicate: FilterRule) {
    Errors::get_mut().filter.predicate = predicate;
}
