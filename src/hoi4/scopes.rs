#![allow(non_upper_case_globals)]

use std::fmt::Formatter;
use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::{display_choices, TigerHashMap};
use crate::scopes::{ArgumentValue, Scopes};

pub fn scope_from_snake_case(s: &str) -> Option<Scopes> {
    Some(match s {
        "none" => Scopes::None,
        // TODO
        _ => return std::option::Option::None,
    })
}

pub fn display_fmt(s: Scopes, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if s.contains(Scopes::None) {
        vec.push("none");
    }
    // TODO
    display_choices(f, &vec, "or")
}

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    // TODO
    None
}

#[inline]
pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_TO_SCOPE_MAP.get(name).copied()
}

static SCOPE_TO_SCOPE_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to) in SCOPE_TO_SCOPE.iter().copied() {
            hash.insert(s, (from, to));
        }
        hash
    });

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    // TODO
];

#[inline]
pub fn scope_prefix(name: &str) -> Option<(Scopes, Scopes, ArgumentValue)> {
    SCOPE_PREFIX_MAP.get(name).copied()
}

static SCOPE_PREFIX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to, argument) in SCOPE_PREFIX.iter().copied() {
            hash.insert(s, (from, to, argument));
        }
        hash
    });

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_PREFIX: &[(Scopes, &str, Scopes, ArgumentValue)] = {
    use crate::item::Item;
    use ArgumentValue::*;
    &[
        // TODO
    ]
};

#[inline]
pub fn scope_iterator(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_ITERATOR_MAP.get(name).copied()
}

static SCOPE_ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to) in SCOPE_ITERATOR.iter().copied() {
            hash.insert(s, (from, to));
        }
        hash
    });

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `documentation/effects_documentation.md` from the game files.
/// These are the list iterators. Every entry represents
/// a every_, random_, and any_ version.
/// TODO: Hoi4 does not have the ordered_ versions.
/// TODO: Hoi4 has any_ iterators that don't have corresponding every_ or random_+
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::None, "active_scientist", Scopes::Character),
    (Scopes::Country, "allied_country", Scopes::Country),
    (Scopes::Country, "army_leader", Scopes::Character),
    (Scopes::Country, "character", Scopes::Character),
    (Scopes::Country, "controlled_state", Scopes::State),
    (Scopes::Country, "core_state", Scopes::State),
    (Scopes::None, "country", Scopes::Country),
    (Scopes::Country, "country_division", Scopes::Division),
    (Scopes::Country, "country_with_original_tag", Scopes::Country),
    (Scopes::Country, "enemy_country", Scopes::Country),
    (Scopes::Country, "military_industrial_organization", Scopes::IndustrialOrg),
    (Scopes::Country, "navy_leader", Scopes::Character),
    (Scopes::Country, "neighbor_country", Scopes::Country),
    (Scopes::State, "neighbor_state", Scopes::State),
    (Scopes::Country, "occupied_country", Scopes::Country),
    // TODO: the any_ version is any_operative_leader
    (Scopes::Country.union(Scopes::Operation), "operative", Scopes::Character),
    (Scopes::Country, "other_country", Scopes::Country),
    (Scopes::Country, "owned_state", Scopes::State),
    (Scopes::None, "possible_country", Scopes::Country),
    (Scopes::None, "purchase_contract", Scopes::PurchaseContract),
    (Scopes::None, "scientist", Scopes::Character),
    (Scopes::None, "state", Scopes::State),
    (Scopes::State, "state_division", Scopes::Division),
    (Scopes::Country, "subject_country", Scopes::Country),
    (Scopes::Country, "unit_leader", Scopes::Character),
];

pub fn scope_iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_ITERATOR_REMOVED: &[(&str, &str, &str)] = &[];

pub fn scope_to_scope_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_TO_SCOPE_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
