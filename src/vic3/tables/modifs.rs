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

// Redeclare the `ModifKinds` enums as bare numbers, so that we can do | on them in const tables.
const NoneModifKind: u16 = 0x0001;
const Battle: u16 = 0x0002;
const Building: u16 = 0x0004;
const Character: u16 = 0x0008;
const Country: u16 = 0x0010;
const Front: u16 = 0x0020;
const InterestGroup: u16 = 0x0040;
const Market: u16 = 0x0080;
const PoliticalMovement: u16 = 0x0100;
const State: u16 = 0x0200;
const Tariff: u16 = 0x0400;
const Tax: u16 = 0x0800;
const Unit: u16 = 0x1000;

/// LAST UPDATED VERSION 1.9.2
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, u16)] = &[];
