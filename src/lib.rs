//! This library forms the bulk of the -tiger family of validators: `ck3-tiger`, `vic3-tiger`, and
//! `imperator-tiger`. Each executable is a small wrapper around the functions in this library that
//! start and perform validation.

#[cfg(all(feature = "ck3", feature = "vic3", feature = "imperator", not(doc)))]
compile_error!("features \"ck3\", \"vic3\", and \"imperator\" cannot be enabled at the same time");

#[cfg(all(not(feature = "ck3"), not(feature = "vic3"), not(feature = "imperator")))]
compile_error!("exactly one of the features \"ck3\", \"vic3\", \"imperator\" must be enabled");

pub use crate::config_load::validate_config_file;
pub use crate::everything::Everything;
pub use crate::fileset::FileKind;
pub use crate::game::Game;
pub use crate::item::Item;
#[cfg(feature = "vic3")]
pub use crate::mod_metadata::ModMetadata;
#[cfg(any(feature = "ck3", feature = "imperator"))]
pub use crate::modfile::ModFile;
pub use crate::report::{
    Confidence, LogReport, PointedMessage, Severity, add_loaded_mod_root, disable_ansi_colors,
    emit_reports, log, set_output_file, set_output_style, set_show_loaded_mods, set_show_vanilla,
    suppress_from_json, take_reports,
};
pub use crate::token::{Loc, Token};

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
mod util;
mod validate;
mod validator;
