//! Error report collection and printing facilities.

pub(crate) use builder::{err, fatal, report, tips, untidy, warn};
pub(crate) use error_key::ErrorKey;
pub(crate) use error_loc::ErrorLoc;
pub use errors::*;
pub(crate) use filter::FilterRule;
pub(crate) use output_style::OutputStyle;
pub use report_struct::{Confidence, LogReport, PointedMessage, Severity};

mod builder;
mod error_key;
mod error_loc;
mod errors;
mod filter;
mod output_style;
mod report_struct;
mod writer;
mod writer_json;
