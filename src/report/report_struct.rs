use strum_macros::{Display, EnumIter};

use crate::report::ErrorKey;
use crate::token::Loc;

/// Describes a report about a potentially problematic situation that can be logged.
#[derive(Debug)]
pub struct LogReport<'a> {
    /// Used for choosing output colors and for filtering reports.
    pub lvl: LogLevel,
    /// Defines the problem category. Used for filtering reports.
    pub key: ErrorKey,
    /// The primary error message. A short description of the problem.
    pub msg: &'a str,
    /// Optional info message to be printed at the end.
    pub info: Option<&'a str>,
    /// Should contain one or more elements.
    pub pointers: Vec<PointedMessage<'a>>,
}

impl LogReport<'_> {
    /// Returns the primary pointer.
    pub fn primary(&self) -> &PointedMessage {
        self.pointers
            .get(0)
            .expect("A LogReport must always have at least one PointedMessage.")
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
pub struct PointedMessage<'a> {
    /// Which file and where in the file the error occurs.
    /// Might point to a whole file, rather than a specific location in the file.
    pub location: Loc,
    /// The length of the offending phrase in characters.
    /// Set this to 1 if the length cannot be determined.
    /// This will determine the number of carets that are printed at the given location.
    /// e.g.:     ^^^^^^^^^
    pub length: usize,
    /// A short message that will be printed at the caret location.
    pub msg: Option<&'a str>,
}

/// Replaces the `ErrorLevel` that previously existed.
#[derive(Default, Debug, Clone, Copy)]
pub struct LogLevel {
    /// The seriousness of the error.
    pub severity: Severity,
    pub confidence: Confidence,
}

impl LogLevel {
    pub fn new(severity: Severity, confidence: Confidence) -> Self {
        LogLevel {
            severity,
            confidence,
        }
    }
}

/// Determines the output colour.
/// User can also filter by minimum severity level: e.g. don't show me Info-level messages.
#[derive(Default, Debug, Display, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash, EnumIter)]
pub enum Severity {
    /// This problem likely will not affect the end-user, but is just sloppy code.
    /// It constitutes technical debt that will increase maintenance costs.
    Untidy,
    /// The problem may lead to minor glitches, but won't seriously affect gameplay.
    Info,
    /// The problem may lead to features not working, unexpected behaviour and other
    /// bugs that will noticeably impact the end-user's experience playing the game.
    #[default]
    Warning,
    /// The problem can potentially cause crashes.
    Error,
}

/// Mostly invisible in the output.
/// User can filter by minimum confidence level.
/// This would be a dial for how many false positives they're willing to put up with.
#[derive(Default, Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Confidence {
    /// Quite likely to be a false positive.
    Weak,
    /// Reasonably confident that the problem is real.
    #[default]
    Reasonable,
    /// Very confident that this problem is real.
    Strong,
}
