use strum_macros::EnumString;
use strum_macros::{Display, EnumIter};

use crate::report::ErrorKey;
use crate::token::Loc;

/// Describes a report about a potentially problematic situation that can be logged.
#[derive(Debug)]
pub struct LogReport {
    /// Used for choosing output colors and for filtering reports.
    pub severity: Severity,
    /// Mostly used for filtering reports.
    pub confidence: Confidence,
    /// Defines the problem category. Used for filtering reports.
    pub key: ErrorKey,
    /// The primary error message. A short description of the problem.
    pub msg: String,
    /// Optional info message to be printed at the end.
    pub info: Option<String>,
    /// Should contain one or more elements.
    pub pointers: Vec<PointedMessage>,
}

impl LogReport {
    /// Returns the primary pointer.
    pub fn primary(&self) -> &PointedMessage {
        self.pointers.get(0).expect("A LogReport must always have at least one PointedMessage.")
    }
    /// Returns the length of the longest line number.
    pub fn indentation(&self) -> usize {
        self.pointers
            .iter()
            .map(|pointer| pointer.location.line.to_string().len())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub struct PointedMessage {
    /// Which file and where in the file the error occurs.
    /// Might point to a whole file, rather than a specific location in the file.
    pub location: Loc,
    /// The length of the offending phrase in characters.
    /// Set this to 1 if the length cannot be determined.
    /// This will determine the number of carets that are printed at the given location.
    /// e.g.:     ^^^^^^^^^
    /// TODO: If we end up adding length to Loc, this field can be deleted.
    pub length: usize,
    /// A short message that will be printed at the caret location.
    pub msg: Option<String>,
}

impl PointedMessage {
    pub fn new(location: Loc) -> Self {
        Self { location, msg: None, length: 1 }
    }
}

/// Determines the output colour.
/// User can also filter by minimum severity level: e.g. don't show me Info-level messages.
///
/// The order of these enum values determines the level of severity they denote.
/// Do not change the order unless you mean to change the logic of the program!
#[derive(
    Default, Debug, Display, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumString, EnumIter,
)]
pub enum Severity {
    /// These are things that aren't necessarily wrong, but there may be a better, more
    /// idiomatic way to do it. This may also include performance issues.
    Tips,
    /// This code smells.
    /// The player is unlikely to be impacted directly, but developers working on this codebase
    /// will likely experience maintenance headaches.
    Untidy,
    /// This will result in glitches that will noticeably impact the player's gaming experience.
    /// Missing translations are an example.
    #[default]
    Warning,
    /// This code probably doesn't work as intended. The player may experience bugs.
    Error,
    /// This is likely to cause crashes.
    Fatal,
}

/// Mostly invisible in the output.
/// User can filter by minimum confidence level.
/// This would be a dial for how many false positives they're willing to put up with.
///
/// The order of these enum values determines the level of confidence they denote.
/// Do not change the order unless you mean to change the logic of the program!
#[derive(
    Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumString, EnumIter,
)]
pub enum Confidence {
    /// Quite likely to be a false positive.
    Weak,
    /// Reasonably confident that the problem is real.
    #[default]
    Reasonable,
    /// Very confident that this problem is real.
    Strong,
}
