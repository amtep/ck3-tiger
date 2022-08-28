#![warn(missing_debug_implementations)]
// Turn on clippy pedantic, but not all of them yet.
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
// Turn on some rustc lints
#![warn(future_incompatible)]
#![warn(missing_copy_implementations)]
#![warn(noop_method_call)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
// As long as the validator is so unfinished, this warning is just noise
// Unfortunately we can't set it to warn only on unused functions. It keeps
// warning about struct fields.
#![allow(dead_code)]

pub mod errorkey;
pub mod errors;
pub mod everything;
pub mod modfile;

mod block;
mod decisions;
mod desc;
mod dynasties;
mod events;
mod fileset;
mod gameconcepts;
mod houses;
mod interactions;
mod localization;
mod pdxfile;
mod prov_history;
mod provinces;
mod religions;
mod titles;
mod validate;
