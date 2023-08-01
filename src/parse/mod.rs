//! Parsers for the various kinds of game script.

#[cfg(feature = "ck3")]
pub mod csv;
#[cfg(feature = "vic3")]
pub mod json;
pub mod localization;
pub mod pdxfile;
