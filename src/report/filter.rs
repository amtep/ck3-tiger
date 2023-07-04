use std::path::PathBuf;

use fnv::FnvHashMap;

use crate::fileset::FileKind;
use crate::report::{Confidence, ErrorKey, LogLevel, LogReport, Severity};
use crate::token::Loc;

/// Determines whether a given Report should be printed.
/// If a report is matched by both the blacklist and the whitelist, it will not be printed.
#[derive(Default, Debug)]
pub struct ReportFilter {
    /// Whether to log errors in vanilla CK3 files
    pub show_vanilla: bool,
    /// Whether to log errors in other loaded mods
    pub show_loaded_mods: bool,
    pub rules: FilterRule,

    /// Minimum error level to log
    pub min_level: LogLevel,
    /// If a whitelist exists, reports must match ALL FilterMatches to be eligible to be printed.
    pub whitelist: Option<RulesList>,
    /// If a blacklist exists, reports must match NONE of the FilterMatches to be eligible to be printed.
    pub blacklist: Option<RulesList>,
}

#[derive(Default, Debug)]
pub enum FilterRule {
    /// Mostly serves as a sensible default value. All reports always match this rule.
    #[default]
    Tautology,
    /// Configured by the AND-key. The top-level rule is always a conjunction.
    /// Reports must match all enclosed rules to match the conjunction.
    Conjunction(Vec<FilterRule>),
    /// Configured by the OR-key.
    /// Reports must match at least one of the enclosed rules to match the disjunction.
    Disjunction(Vec<FilterRule>),
    /// Configured by the NOT-key.
    /// Reports must not match the enclosed rule to match the negation.
    Negation(Box<FilterRule>),
    /// Reports must be at least of the given Severity level to match the rule.
    Severity(Severity),
    /// Reports must be at least of the given Confidence level to match the rule.
    Confidence(Confidence),
    /// The report's `ErrorKey` must be one of the listed keys for the report to match the rule.
    Key(Vec<ErrorKey>),
    /// The report's pointers must contain a file that
    File(Vec<PathBuf>),
}

impl ReportFilter {
    /// Returns true iff the report should be printed.
    /// A print will be rejected if the report matches at least one of the following conditions:
    /// - Its Severity or Confidence level is too low.
    /// - It's from vanilla or a loaded mod and the program is configured to ignore those locations.
    /// - The filter has a whitelist, and the report doesn't match it.
    /// - The filter has a blacklist, and the report does match it.
    pub fn should_print_report(&self, report: &LogReport) -> bool {
        self.should_print(report.lvl, &report.primary().location, report.key)
    }
    pub fn should_print(&self, lvl: LogLevel, location: &Loc, key: ErrorKey) -> bool {
        if key == ErrorKey::Config {
            // Any errors concerning the Config should be easy to fix and will fundamentally
            // undermine the operation of the application. They must always be printed.
            return true;
        }
        if lvl.severity < self.min_level.severity || lvl.confidence < self.min_level.confidence {
            return false;
        }
        let linked_location = Self::get_end_of_link_chain(location);
        if (linked_location.kind <= FileKind::Vanilla && !self.show_vanilla)
            || (matches!(linked_location.kind, FileKind::LoadedMod(_)) && !self.show_loaded_mods)
        {
            return false;
        }
        if let Some(whitelist) = &self.whitelist {
            if !whitelist.matches(linked_location, key) {
                return false;
            }
        }
        if let Some(blacklist) = &self.blacklist {
            if blacklist.matches(linked_location, key) {
                return false;
            }
        }
        true
    }
    /// Check all elements of the loc link chain.
    /// This is necessary because of cases like a mod passing `CHARACTER = this` to a vanilla
    /// script effect that does not expect that. The error would be located in the vanilla script
    /// but would be caused by the mod.
    fn get_end_of_link_chain(loc: &Loc) -> &Loc {
        if let Some(loc) = &loc.link {
            Self::get_end_of_link_chain(loc)
        } else {
            loc
        }
    }
    pub fn get_or_create_rules_list(&mut self, rules_type: RulesListType) -> &mut RulesList {
        if rules_type == RulesListType::Whitelist {
            if self.whitelist.is_none() {
                self.whitelist = Some(RulesList::default());
            }
            self.whitelist.as_mut().expect("Must be present")
        } else {
            if self.blacklist.is_none() {
                self.blacklist = Some(RulesList::default());
            }
            self.blacklist.as_mut().expect("Must be present")
        }
    }
}

/// This struct forms a disjunction: only one of the rules needs to match for a report
/// to match the `RulesList` as a whole.
#[derive(Default, Debug)]
pub struct RulesList {
    /// Match reports of which the end of its location's link-chain and its `ErrorKey` matches
    /// one of the entries in this map.
    keys_for_path: FnvHashMap<PathBuf, Vec<ErrorKey>>,
    /// Match reports whose `ErrorKey` is contained within this Vec.
    keys: Vec<ErrorKey>,
    /// Match reports of which the end of its location's link-chain is contained within this vec.
    paths: Vec<PathBuf>,
}

impl RulesList {
    pub fn matches(&self, loc: &Loc, key: ErrorKey) -> bool {
        if self.keys.contains(&key) {
            return true;
        }
        for (path, keys) in &self.keys_for_path {
            if loc.pathname.starts_with(path) && keys.contains(&key) {
                return true;
            }
        }
        for path in &self.paths {
            if loc.pathname.starts_with(path) {
                return true;
            }
        }
        false
    }

    pub fn add_rule_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }
    pub fn add_rule_key(&mut self, key: ErrorKey) {
        self.keys.push(key);
    }
    pub fn add_rule_key_in_path(&mut self, path: PathBuf, key: ErrorKey) {
        self.keys_for_path.entry(path).or_default().push(key);
    }
}

/// Deprecated
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RulesListType {
    Blacklist,
    Whitelist,
}