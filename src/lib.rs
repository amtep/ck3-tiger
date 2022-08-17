#![warn(missing_debug_implementations)]
// Turn on clippy pedantic, but not all of them yet.
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
// Turn on some rustc lints
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(noop_method_call)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]

pub mod errors;
pub mod everything;
pub mod modfile;

mod localization;
mod pdxfile;
mod scope;
mod validate;
