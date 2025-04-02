//! Validators for game item types which are generic across all supported games.
//! Each sub-mod handles a specific item type or group of related item types.

pub mod accessory;
pub mod achievements;
pub mod assets;
#[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator"))]
pub mod coa;
pub mod coadesigner;
pub mod colors;
pub mod customloca;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod data_binding;
pub mod defines;
pub mod dlc;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod dna;
pub mod effect_localization;
#[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator"))]
pub mod ethnicity;
pub mod events;
pub mod fonts;
#[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator"))]
pub mod genes;
pub mod gui;
pub mod localization;
pub mod music;
pub mod on_actions;
#[cfg(any(feature = "ck3", feature = "vic3", feature = "imperator"))]
pub mod portrait;
pub mod script_values;
pub mod scripted_effects;
pub mod scripted_guis;
pub mod scripted_lists;
pub mod scripted_modifiers;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod scripted_rules;
pub mod scripted_triggers;
pub mod trigger_localization;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod tutorials;
