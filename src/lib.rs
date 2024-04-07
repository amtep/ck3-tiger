//! This library forms the bulk of the -tiger family of validators: `ck3-tiger`, `vic3-tiger`, and
//! `imperator-tiger`. Each executable is a small wrapper around the functions in this library that
//! start and perform validation.

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
// This was causing a lot of warnings.
#![allow(clippy::too_many_lines)]
// The construction being warned about here is the best way to express
// validation of a field while handling the case of the field not existing.
#![allow(clippy::blocks_in_conditions)]
// Turn on selected warnings from clippy's restricted set
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::decimal_literal_representation)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::fn_to_numeric_cast_any)]
#![warn(clippy::format_push_string)]
#![warn(clippy::get_unwrap)]
#![warn(clippy::if_then_some_else_none)]
#![warn(clippy::impl_trait_in_params)]
#![warn(clippy::integer_division)]
#![warn(clippy::lossy_float_literal)]
#![warn(clippy::mixed_read_write_in_expression)]
#![warn(clippy::mutex_atomic)]
#![warn(clippy::rc_buffer)]
#![warn(clippy::rc_mutex)]
#![warn(clippy::rest_pat_in_fully_bound_structs)]
#![warn(clippy::string_add)]
#![warn(clippy::string_to_string)]

#[cfg(all(feature = "ck3", feature = "vic3", feature = "imperator", not(doc)))]
compile_error!("features \"ck3\", \"vic3\", and \"imperator\" cannot be enabled at the same time");

#[cfg(all(not(feature = "ck3"), not(feature = "vic3"), not(feature = "imperator")))]
compile_error!("exactly one of the features \"ck3\", \"vic3\", \"imperator\" must be enabled");

pub use crate::config_load::validate_config_file;
pub use crate::everything::Everything;
pub use crate::fileset::FileKind;
pub use crate::game::Game;
pub use crate::gamedir::{find_game_directory_steam, find_paradox_directory};
pub use crate::item::Item;
#[cfg(feature = "vic3")]
pub use crate::mod_metadata::ModMetadata;
#[cfg(any(feature = "ck3", feature = "imperator"))]
pub use crate::modfile::ModFile;
pub use crate::report::{
    add_loaded_mod_root, disable_ansi_colors, emit_reports, log, set_output_file, set_output_style,
    set_show_loaded_mods, set_show_vanilla, suppress_from_file, take_reports, Confidence,
    LogReport, PointedMessage, Severity,
};
pub use crate::token::{Loc, Token};
pub use crate::update::{update, UpdateError};

#[cfg(feature = "ck3")]
mod ck3;
#[cfg(feature = "imperator")]
mod imperator;
#[cfg(feature = "vic3")]
mod vic3;

mod block;
mod config_load;
mod context;
mod data;
mod datatype;
mod date;
mod db;
mod dds;
mod desc;
mod effect;
mod effect_validation;
mod everything;
mod fileset;
mod game;
mod gamedir;
mod gui;
mod helpers;
mod item;
mod lowercase;
mod macros;
#[cfg(feature = "vic3")]
mod mod_metadata;
#[cfg(any(feature = "ck3", feature = "imperator"))]
mod modfile;
mod modif;
mod on_action;
mod parse;
mod pathtable;
mod pdxfile;
mod report;
mod rivers;
mod scopes;
mod script_value;
mod token;
mod tooltipped;
mod trigger;
mod update;
mod util;
mod validate;
mod validator;
