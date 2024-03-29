//! Parsers for the various kinds of game script.

pub mod cob;
#[cfg(any(feature = "ck3", feature = "imperator"))]
pub mod csv;
#[cfg(feature = "vic3")]
pub mod json;
pub mod localization;
pub mod pdxfile;
