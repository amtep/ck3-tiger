#![allow(non_upper_case_globals)]

use crate::everything::Everything;
use crate::item::Item;
use crate::modif::ModifKinds;
use crate::report::{error_info, warn_info, ErrorKey};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_modif(name: &Token, data: &Everything, warn: bool) -> Option<ModifKinds> {
    for &(entry_name, mk) in MODIF_TABLE {
        if name.is(entry_name) {
            return Some(ModifKinds::from_bits_truncate(mk));
        }
    }

    // TODO Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    None
}

#[allow(clippy::unnecessary_wraps)]
fn modif_check(
    name: &Token,
    s: &str,
    itype: Item,
    mk: ModifKinds,
    data: &Everything,
    warn: bool,
) -> Option<ModifKinds> {
    if warn {
        data.verify_exists_implied(itype, s, name);
    }
    Some(mk)
}

// Redeclare the `ModifKinds` enums as bare numbers, so that we can to | on them in const tables.
const Character: u8 = 0x01;
const Province: u8 = 0x02;
const County: u8 = 0x04;
const Terrain: u8 = 0x08;
const Culture: u8 = 0x10;
const Scheme: u8 = 0x20;
const TravelPlan: u8 = 0x40;

/// LAST UPDATED VERSION 1.9.2
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, u8)] = &[];
