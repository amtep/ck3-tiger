//! Collect error reports and then write them out.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs::{File, read};
use std::io::{Write, stdout};
use std::mem::take;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use anyhow::Result;
use encoding_rs::{UTF_8, WINDOWS_1252};
use once_cell::sync::Lazy;

use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::macros::MACRO_MAP;
use crate::report::error_loc::ErrorLoc;
use crate::report::filter::ReportFilter;
use crate::report::suppress::{Suppression, SuppressionKey};
use crate::report::writer::log_report;
use crate::report::writer_json::log_report_json;
use crate::report::{ErrorKey, FilterRule, LogReport, OutputStyle, PointedMessage};
use crate::token::{Loc, leak};

static ERRORS: Lazy<Mutex<Errors>> = Lazy::new(|| Mutex::new(Errors::default()));

#[allow(missing_debug_implementations)]
pub struct Errors {
    pub(crate) output: RefCell<Box<dyn Write + Send>>,

    /// Extra loaded mods' error tags.
    pub(crate) loaded_mods_labels: Vec<String>,

    /// Loaded DLCs' error tags.
    pub(crate) loaded_dlcs_labels: Vec<String>,

    pub(crate) cache: Cache,

    /// Determines whether a report should be printed.
    pub(crate) filter: ReportFilter,
    /// Output color and style configuration.
    pub(crate) styles: OutputStyle,

    pub(crate) suppress: TigerHashMap<SuppressionKey, Vec<Suppression>>,

    /// All reports that passed the checks, stored here to be sorted before being emitted all at once.
    /// The "abbreviated" reports don't participate in this. They are still emitted immediately.
    /// It's a `HashSet` because duplicate reports are fairly common due to macro expansion and other revalidations.
    storage: TigerHashSet<LogReport>,
}

impl Default for Errors {
    fn default() -> Self {
        Errors {
            output: RefCell::new(Box::new(stdout())),
            loaded_mods_labels: Vec::default(),
            loaded_dlcs_labels: Vec::default(),
            cache: Cache::default(),
            filter: ReportFilter::default(),
            styles: OutputStyle::default(),
            storage: TigerHashSet::default(),
            suppress: TigerHashMap::default(),
        }
    }
}

impl Errors {
    fn should_suppress(&mut self, report: &LogReport) -> bool {
        // TODO: see if this can be done without cloning
        let key = SuppressionKey { key: report.key, message: report.msg.clone() };
        if let Some(v) = self.suppress.get(&key) {
            for suppression in v {
                if suppression.len() != report.pointers.len() {
                    continue;
                }
                for (s, p) in suppression.iter().zip(report.pointers.iter()) {
                    if s.path == p.loc.pathname().to_string_lossy()
                        && s.tag == p.msg
                        && s.line.as_deref() == self.cache.get_line(p.loc)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will add it to the storage.
    fn push_report(&mut self, report: LogReport) {
        if !self.filter.should_print_report(&report) || self.should_suppress(&report) {
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
        if self.filter.should_maybe_print(key, loc) {
            if loc.line == 0 {
                _ = writeln!(self.output.get_mut(), "({key}) {}", loc.pathname().to_string_lossy());
            } else if let Some(line) = self.cache.get_line(loc) {
                _ = writeln!(self.output.get_mut(), "({key}) {line}");
            }
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

    pub fn store_source_file(&mut self, fullpath: PathBuf, source: &'static str) {
        self.cache.filecache.insert(fullpath, source);
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

#[derive(Debug, Default)]
pub(crate) struct Cache {
    /// Files that have been read in to get the lines where errors occurred.
    /// Cached here to avoid duplicate I/O and UTF-8 parsing.
    filecache: TigerHashMap<PathBuf, &'static str>,

    /// Files that have been linesplit, cached to avoid doing that work again
    linecache: TigerHashMap<PathBuf, Vec<&'static str>>,
}

impl Cache {
    /// Fetch the contents of a single line from a script file.
    pub(crate) fn get_line(&mut self, loc: Loc) -> Option<&'static str> {
        if loc.line == 0 {
            return None;
        }
        let fullpath = loc.fullpath();
        if let Some(lines) = self.linecache.get(fullpath) {
            return lines.get(loc.line as usize - 1).copied();
        }
        if let Some(contents) = self.filecache.get(fullpath) {
            let lines: Vec<_> = contents.lines().collect();
            let line = lines.get(loc.line as usize - 1).copied();
            self.linecache.insert(fullpath.to_path_buf(), lines);
            return line;
        }
        let bytes = read(fullpath).ok()?;
        // Try decoding it as UTF-8. If that succeeds without errors, use it, otherwise fall back
        // to WINDOWS_1252. The decode method will do BOM stripping.
        let contents = match UTF_8.decode(&bytes) {
            (contents, _, false) => contents,
            (_, _, true) => WINDOWS_1252.decode(&bytes).0,
        };
        let contents = leak(contents.into_owned());
        self.filecache.insert(fullpath.to_path_buf(), contents);

        let lines: Vec<_> = contents.lines().collect();
        let line = lines.get(loc.line as usize - 1).copied();
        self.linecache.insert(fullpath.to_path_buf(), lines);
        line
    }
}

/// Record a secondary mod to be loaded before the one being validated.
/// `label` is what it should be called in the error reports; ideally only a few characters long.
pub fn add_loaded_mod_root(label: String) {
    let mut errors = Errors::get_mut();
    errors.loaded_mods_labels.push(label);
}

/// Record a DLC directory from the vanilla installation.
/// `label` is what it should be called in the error reports.
pub fn add_loaded_dlc_root(label: String) {
    let mut errors = Errors::get_mut();
    errors.loaded_dlcs_labels.push(label);
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
    if let Some(link) = pointer.loc.link_idx {
        let from_here = PointedMessage {
            loc: MACRO_MAP.get_loc(link).unwrap(),
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
    Errors::get().filter.should_maybe_print(key, eloc.into_loc())
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

pub fn store_source_file(fullpath: PathBuf, source: &'static str) {
    Errors::get_mut().store_source_file(fullpath, source);
}

// =================================================================================================
// =============== Deprecated legacy calls to submit reports:
// =================================================================================================

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
