use std::sync::LazyLock;

use crate::helpers::TigerHashMap;
use crate::scopes::Scopes;

#[inline]
pub fn iterator(name: &str) -> Option<(Scopes, Scopes)> {
    ITERATOR_MAP.get(name).copied()
}

static ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in ITERATOR.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED VERSION 2.0.4
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::State, "state_province", Scopes::Province),
    (Scopes::Character, "character_treasure", Scopes::Treasure),
    (Scopes::Character, "character_unit", Scopes::Unit),
    (Scopes::Character, "child", Scopes::Character),
    (Scopes::Character, "friend", Scopes::Character),
    (Scopes::Character, "governor_state", Scopes::State),
    (Scopes::Character, "holdings", Scopes::Province),
    (Scopes::Character, "parent", Scopes::Character),
    (Scopes::Character, "rival", Scopes::Character),
    (Scopes::Character, "sibling", Scopes::Character),
    (Scopes::Character, "support_as_heir", Scopes::Character),
    (Scopes::Governorship, "governorship_state", Scopes::State),
    (Scopes::Country, "allied_country", Scopes::Country),
    (Scopes::Country, "army", Scopes::Unit),
    (Scopes::Country, "available_deity", Scopes::Deity),
    (Scopes::Country, "character", Scopes::Character),
    (Scopes::Country, "commander", Scopes::Character),
    (Scopes::Country, "countries_at_war_with", Scopes::Country),
    (Scopes::Country, "country_culture", Scopes::CountryCulture),
    (Scopes::Country, "country_state", Scopes::State),
    (Scopes::Country, "country_sub_unit", Scopes::SubUnit),
    (Scopes::Country, "country_treasure", Scopes::Treasure),
    (Scopes::Country, "current_war", Scopes::War),
    (Scopes::Country, "family", Scopes::Family),
    (Scopes::Country, "governorships", Scopes::Governorship),
    (Scopes::Country, "integrated_culture", Scopes::CountryCulture),
    (Scopes::Country, "legion", Scopes::Legion),
    (Scopes::Country, "navy", Scopes::Unit),
    (Scopes::Country, "neighbour_country", Scopes::Country),
    (Scopes::Country, "owned_holy_site", Scopes::Province),
    (Scopes::Country, "owned_province", Scopes::Province),
    (Scopes::Country, "pantheon_deity", Scopes::Deity),
    (Scopes::Country, "party", Scopes::Party),
    (Scopes::Country, "subject", Scopes::Country),
    (Scopes::Country, "successor", Scopes::Character),
    (Scopes::Country, "unit", Scopes::Unit),
    (Scopes::Party, "party_member", Scopes::Character),
    (Scopes::Legion, "legion_commander", Scopes::Character),
    (Scopes::Legion, "legion_unit", Scopes::Unit),
    (Scopes::Region, "neighbor_region", Scopes::Region),
    (Scopes::Region, "region_area", Scopes::Area),
    (Scopes::Region, "region_province", Scopes::Province),
    (Scopes::Region, "region_province_including_unownable", Scopes::Province),
    (Scopes::Region, "region_state", Scopes::State),
    (Scopes::Area, "area_including_unownable_province", Scopes::Province),
    (Scopes::Area, "area_province", Scopes::Province),
    (Scopes::Area, "area_state", Scopes::State),
    (Scopes::Area, "neighbor_area", Scopes::Area),
    (Scopes::Unit, "sub_unit", Scopes::SubUnit),
    (Scopes::Family, "family_member", Scopes::Character),
    (Scopes::War, "war_attacker", Scopes::Country),
    (Scopes::War, "war_defender", Scopes::Country),
    (Scopes::War, "war_participant", Scopes::Country),
    (Scopes::Province, "great_work_in_province", Scopes::GreatWork),
    (Scopes::Province, "neighbor_province", Scopes::Province),
    (Scopes::Province, "pops_in_province", Scopes::Pop),
    (Scopes::Province, "province_treasure", Scopes::Treasure),
    (Scopes::Province, "unit_in_province", Scopes::Unit),
    (Scopes::None, "active_war", Scopes::War),
    (Scopes::None, "area", Scopes::Area),
    (Scopes::None, "country", Scopes::Country),
    (Scopes::None, "deity", Scopes::Deity),
    (Scopes::None, "ended_war", Scopes::War),
    (Scopes::None, "holy_site", Scopes::Province),
    (Scopes::None, "living_character", Scopes::Character),
    (Scopes::None, "ownable_province", Scopes::Province),
    (Scopes::None, "province", Scopes::Province),
    (Scopes::None, "region", Scopes::Region),
    (Scopes::None, "sea_and_river_zone", Scopes::Province),
    (Scopes::None, "in_list", Scopes::all()),
    (Scopes::None, "in_global_list", Scopes::all()),
];

pub fn iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

/// LAST UPDATED VERSION 2.0.4
/// Every entry represents a every_, ordered_, random_, and any_ version.
const ITERATOR_REMOVED: &[(&str, &str, &str)] = &[];
