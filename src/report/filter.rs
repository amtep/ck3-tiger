use std::path::PathBuf;

use crate::block::Comparator;

use crate::fileset::FileKind;
use crate::report::{Confidence, ErrorKey, LogReport, Severity};
use crate::token::Loc;

/// Determines whether a given Report should be printed.
/// If a report is matched by both the blacklist and the whitelist, it will not be printed.
#[derive(Default, Debug)]
pub struct ReportFilter {
    /// Whether to log errors in vanilla CK3 files
    pub show_vanilla: bool,
    /// Whether to log errors in other loaded mods
    pub show_loaded_mods: bool,
    /// A complex trigger that evaluates a report to assess whether it should be printed.
    pub predicate: FilterRule,
}

impl ReportFilter {
    /// Returns true iff the report should be printed.
    /// A print will be rejected if the report matches at least one of the following conditions:
    /// - Its Severity or Confidence level is too low.
    /// - It's from vanilla or a loaded mod and the program is configured to ignore those locations.
    /// - The filter has a whitelist, and the report doesn't match it.
    /// - The filter has a blacklist, and the report does match it.
    pub fn should_print_report(&self, report: &LogReport) -> bool {
        if report.key == ErrorKey::Config {
            // Any errors concerning the Config should be easy to fix and will fundamentally
            // undermine the operation of the application. They must always be printed.
            return true;
        }
        // If every single Loc in the chain is out of scope, the report is out of scope.
        let out_of_scope = report.pointers.iter().map(|p| &p.location).all(|loc| {
            (loc.kind <= FileKind::Vanilla && !self.show_vanilla)
                || (matches!(loc.kind, FileKind::LoadedMod(_)) && !self.show_loaded_mods)
        });
        if out_of_scope {
            return false;
        }
        self.predicate.apply(report)
    }
    /// TODO: Check the filter rules to be more sure.
    pub fn should_maybe_print(&self, key: ErrorKey, location: &Loc) -> bool {
        if key == ErrorKey::Config {
            // Any errors concerning the Config should be easy to fix and will fundamentally
            // undermine the operation of the application. They must always be printed.
            return true;
        }
        if (location.kind <= FileKind::Vanilla && !self.show_vanilla)
            || (matches!(location.kind, FileKind::LoadedMod(_)) && !self.show_loaded_mods)
        {
            return false;
        }
        true
    }
}

#[derive(Default, Debug)]
pub enum FilterRule {
    /// Always true.
    #[default]
    Tautology,
    /// Always false.
    Contradiction,
    /// Configured by the AND-key. The top-level rule is always a conjunction.
    /// Reports must match all enclosed rules to match the conjunction.
    Conjunction(Vec<FilterRule>),
    /// Configured by the OR-key.
    /// Reports must match at least one of the enclosed rules to match the disjunction.
    Disjunction(Vec<FilterRule>),
    /// Configured by the NOT-key.
    /// Reports must not match the enclosed rule to match the negation.
    Negation(Box<FilterRule>),
    /// Reports must be within the given Severity range to match the rule.
    /// The condition is built as such: `report.lvl.severity OPERATOR SEVERITY`
    Severity(Comparator, Severity),
    /// Reports must be within the given Confidence range to match the rule.
    /// The condition is built as such: `report.lvl.confidence OPERATOR CONFIDENCE`
    Confidence(Comparator, Confidence),
    /// The report's `ErrorKey` must be the listed key for the report to match the rule.
    Key(ErrorKey),
    /// The report's pointers must contain the given file for the report to match the rule.
    File(PathBuf),
}

impl FilterRule {
    fn apply(&self, report: &LogReport) -> bool {
        match self {
            FilterRule::Tautology => true,
            FilterRule::Contradiction => false,
            FilterRule::Conjunction(children) => children.iter().all(|child| child.apply(report)),
            FilterRule::Disjunction(children) => children.iter().any(|child| child.apply(report)),
            FilterRule::Negation(child) => !child.apply(report),
            FilterRule::Severity(comparator, level) => match comparator {
                Comparator::Equals(_) => report.severity == *level,
                Comparator::NotEquals => report.severity != *level,
                Comparator::GreaterThan => report.severity > *level,
                Comparator::AtLeast => report.severity >= *level,
                Comparator::LessThan => report.severity < *level,
                Comparator::AtMost => report.severity <= *level,
                Comparator::None => panic!("Encountered unexpected operator: {comparator}"),
            },
            FilterRule::Confidence(comparator, level) => match comparator {
                Comparator::Equals(_) => report.confidence == *level,
                Comparator::NotEquals => report.confidence != *level,
                Comparator::GreaterThan => report.confidence > *level,
                Comparator::AtLeast => report.confidence >= *level,
                Comparator::LessThan => report.confidence < *level,
                Comparator::AtMost => report.confidence <= *level,
                Comparator::None => panic!("Encountered unexpected operator: {comparator}"),
            },
            FilterRule::Key(key) => report.key == *key,
            FilterRule::File(path) => {
                report.pointers.iter().any(|pointer| pointer.location.pathname.starts_with(path))
            }
        }
    }
}
