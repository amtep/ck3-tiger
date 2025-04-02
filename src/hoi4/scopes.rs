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
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    // TODO
];

pub fn scope_iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_ITERATOR_REMOVED: &[(&str, &str, &str)] = &[
    (
        "scope_cobelligerent",
        "1.4.0",
        "replaced with _cobelligerent_in_diplo_play, _cobelligerent_in_war",
    ),
    ("supporting_interest_group", "1.8", "replaced with `_influenced_interest_group`"),
];

pub fn scope_to_scope_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_TO_SCOPE_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[
    ("num_commanded_units", "1.6", ""),
    ("num_enemy_units", "1.6", ""),
    ("num_units_not_in_battle", "1.6", ""),
    ("supply", "1.6", ""),
    ("active_diplomatic_play", "1.7", ""),
];
