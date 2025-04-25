//! Validators for game item types which are generic across all supported games.
//! Each sub-mod handles a specific item type or group of related item types.

#[cfg(feature = "jomini")]
pub mod accessory;
pub mod achievements;
pub mod assets;
#[cfg(feature = "jomini")]
pub mod coa;
#[cfg(feature = "jomini")]
pub mod coadesigner;
#[cfg(feature = "jomini")]
pub mod colors;
#[cfg(feature = "jomini")]
pub mod customloca;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod data_binding;
pub mod defines;
pub mod dlc;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod dna;
#[cfg(feature = "jomini")]
pub mod effect_localization;
#[cfg(feature = "jomini")]
pub mod ethnicity;
#[cfg(feature = "jomini")]
pub mod events;
#[cfg(feature = "jomini")]
pub mod fonts;
#[cfg(feature = "jomini")]
pub mod genes;
pub mod gui;
pub mod localization;
#[cfg(feature = "jomini")]
pub mod music;
pub mod on_actions;
#[cfg(feature = "jomini")]
pub mod portrait;
#[cfg(feature = "jomini")]
pub mod script_values;
pub mod scripted_effects;
#[cfg(feature = "jomini")]
pub mod scripted_guis;
#[cfg(feature = "jomini")]
pub mod scripted_lists;
#[cfg(feature = "jomini")]
pub mod scripted_modifiers;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod scripted_rules;
pub mod scripted_triggers;
#[cfg(feature = "jomini")]
pub mod trigger_localization;
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub mod tutorials;
