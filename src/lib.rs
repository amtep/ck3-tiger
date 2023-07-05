#![warn(missing_debug_implementations)]
// Turn on clippy pedantic, but not all of them yet.
#![warn(clippy::pedantic)]
#![allow(clippy::struct_excessive_bools)] // we like our bools
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
// When we do wildcards in `use`, it's deliberate
#![allow(clippy::enum_glob_use)]
#![allow(clippy::wildcard_imports)]
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
// This was causing a lot of warnings.
#![allow(clippy::too_many_lines)]

pub mod everything;
pub mod modfile;
pub mod report;

mod block;
mod context;
mod data;
mod datatype;
mod db;
mod dds;
mod desc;
mod effect;
mod effect_validation;
mod fileset;
mod helpers;
mod item;
mod macrocache;
mod modif;
mod parse;
mod pdxfile;
mod rivers;
mod scopes;
mod scriptvalue;
mod tables;
mod token;
mod tooltipped;
mod trigger;
mod util;
mod validate;
