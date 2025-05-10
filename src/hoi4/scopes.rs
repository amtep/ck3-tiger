#![allow(non_upper_case_globals)]

use std::fmt::Formatter;
use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::{display_choices, expand_scopes_hoi4, TigerHashMap};
use crate::scopes::{ArgumentValue, Scopes};

pub fn display_fmt(s: Scopes, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if s.contains(Scopes::None) {
        vec.push("none");
    }
    if s.contains(Scopes::Value) {
        vec.push("value");
    }
    if s.contains(Scopes::Bool) {
        vec.push("bool");
    }
    if s.contains(Scopes::Flag) {
        vec.push("flag");
    }
    if s.contains(Scopes::Character) {
        vec.push("character");
    }
    if s.contains(Scopes::Country) {
        vec.push("country");
    }
    if s.contains(Scopes::State) {
        vec.push("state");
    }
    if s.contains(Scopes::Ace) {
        vec.push("ace");
    }
    if s.contains(Scopes::Combatant) {
        vec.push("combatant");
    }
    if s.contains(Scopes::Division) {
        vec.push("division");
    }
    if s.contains(Scopes::IndustrialOrg) {
        vec.push("industrial org");
    }
    if s.contains(Scopes::Operation) {
        vec.push("operation");
    }
    if s.contains(Scopes::PurchaseContract) {
        vec.push("purchase contract");
    }
    if s.contains(Scopes::RaidInstance) {
        vec.push("raid instance");
    }
    if s.contains(Scopes::SpecialProject) {
        vec.push("special project");
    }
    if s.contains(Scopes::StrategicRegion) {
        vec.push("strategic region");
    }
    if s.contains(Scopes::CombinedCountryAndState) {
        vec.push("combined country and state");
    }
    if s.contains(Scopes::CombinedCountryAndCharacter) {
        vec.push("combined country and character");
    }
    display_choices(f, &vec, "or")
}

pub fn needs_prefix(_arg: &str, _data: &Everything, _scopes: Scopes) -> Option<&'static str> {
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
            hash.insert(s, (expand_scopes_hoi4(from), to));
        }
        hash
    });

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
// TODO
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Country, "capital", Scopes::State), // undocumented
    (Scopes::Country, "capital_scope", Scopes::State), // undocumented
    (Scopes::State, "controller", Scopes::Country),
    (Scopes::Country, "faction_leader", Scopes::Country),
    (Scopes::Country, "overlord", Scopes::Country),
    (
        Scopes::State
            .union(Scopes::Character)
            .union(Scopes::Division)
            .union(Scopes::IndustrialOrg)
            .union(Scopes::Ace),
        "owner",
        Scopes::Country,
    ),
];

#[inline]
pub fn scope_prefix(name: &str) -> Option<(Scopes, Scopes, ArgumentValue)> {
    SCOPE_PREFIX_MAP.get(name).copied()
}

static SCOPE_PREFIX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to, argument) in SCOPE_PREFIX.iter().copied() {
            hash.insert(s, (expand_scopes_hoi4(from), to, argument));
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
        (Scopes::None, "constant", Scopes::Value, Item(Item::ScriptedConstant)),
        (Scopes::all(), "event_target", Scopes::all(), UncheckedValue),
        (Scopes::Country, "mio", Scopes::IndustrialOrg, Item(Item::IndustrialOrg)),
        (Scopes::Country, "sp", Scopes::SpecialProject, Item(Item::SpecialProject)),
        // TODO: need special handling for token: prefix
        (Scopes::None, "token", Scopes::all(), UncheckedValue),
        (Scopes::all(), "var", Scopes::all(), UncheckedValue),
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
            hash.insert(s, (expand_scopes_hoi4(from), to));
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
    (Scopes::Country, "other_country", Scopes::Country),
    (Scopes::Country, "owned_state", Scopes::State),
    (Scopes::None, "state", Scopes::State),
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
