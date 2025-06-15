use std::sync::LazyLock;

use crate::helpers::{expand_scopes_hoi4, TigerHashMap};
use crate::scopes::{ArgumentValue, Scopes};

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
    (Scopes::None, "no", Scopes::Bool),
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
    (Scopes::None, "yes", Scopes::Bool),
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

pub fn scope_to_scope_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_TO_SCOPE_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
