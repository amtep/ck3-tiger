#![allow(non_upper_case_globals)]

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::modif::ModifKinds;
use crate::report::{report, ErrorKey, Severity};
use crate::token::Token;

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<ModifKinds> {
    let name_lc = Lowercase::new(name.as_str());

    if let result @ Some(_) = MODIF_MAP.get(&name_lc).copied() {
        return result;
    }

    if let Some(info) = MODIF_REMOVED_MAP.get(&name_lc).copied() {
        if let Some(sev) = warn {
            let msg = format!("{name} has been removed");
            report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
        }
        return Some(ModifKinds::all());
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // TODO

    None
}

fn maybe_warn(itype: Item, s: &Lowercase, name: &Token, data: &Everything, warn: Option<Severity>) {
    if let Some(sev) = warn {
        if !data.item_exists_lc(itype, s) {
            let msg = format!("could not find {itype} {s}");
            let info = format!("so the modifier {name} does not exist");
            report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
        }
    }
}

/// Return the modifier localization keys.
/// It's usually just the name, but there are known exceptions.
pub fn modif_loc(name: &Token, data: &Everything) -> (Cow<'static, str>, Cow<'static, str>) {
    let name_lc = Lowercase::new(name.as_str());

    // TODO: check hoi4 format

    if MODIF_MAP.contains_key(&name_lc) {
        let desc_loc = format!("{name_lc}_desc");
        return (name_lc.into_cow(), Cow::Owned(desc_loc));
    }

    let desc_loc = format!("{name}_desc");
    (Cow::Borrowed(name.as_str()), Cow::Owned(desc_loc))
}

static MODIF_MAP: LazyLock<TigerHashMap<Lowercase<'static>, ModifKinds>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), kind);
    }
    hash
});

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `modifiers.log` from the game data dumps.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    // TODO
];

static MODIF_REMOVED_MAP: LazyLock<TigerHashMap<Lowercase<'static>, &'static str>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (s, info) in MODIF_REMOVED_TABLE.iter().copied() {
            hash.insert(Lowercase::new_unchecked(s), info);
        }
        hash
    });

const MODIF_REMOVED_TABLE: &[(&str, &str)] = &[];
