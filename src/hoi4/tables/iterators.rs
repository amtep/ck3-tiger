use std::sync::LazyLock;

use crate::helpers::{expand_scopes_hoi4, TigerHashMap};
use crate::scopes::Scopes;

#[inline]
pub fn iterator(name: &str) -> Option<(Scopes, Scopes)> {
    ITERATOR_MAP.get(name).copied()
}

static ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in ITERATOR.iter().copied() {
        hash.insert(s, (expand_scopes_hoi4(from), to));
    }
    hash
});

/// LAST UPDATED HOI4 VERSION 1.16.4
/// See `documentation/effects_documentation.md` from the game files.
/// These are the list iterators. Every entry represents
/// a every_, random_, and any_ version.
/// TODO: Hoi4 does not have the ordered_ versions.
const ITERATOR: &[(Scopes, &str, Scopes)] = &[
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

pub fn iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const ITERATOR_REMOVED: &[(&str, &str, &str)] = &[];
