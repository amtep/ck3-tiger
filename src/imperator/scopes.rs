#![allow(non_upper_case_globals)]

use std::fmt::Formatter;

use fnv::FnvHashMap;
use once_cell::sync::Lazy;

use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::item::Item;
use crate::scopes::{ArgumentValue, Scopes};

pub fn scope_from_snake_case(s: &str) -> Option<Scopes> {
    Some(match s {
        "none" => Scopes::None,
        "value" => Scopes::Value,
        "bool" => Scopes::Bool,
        "flag" => Scopes::Flag,
        "color" => Scopes::Color,
        "country" => Scopes::Country,
        "character" => Scopes::Character,
        "province" => Scopes::Province,
        "siege" => Scopes::Siege,
        "unit" => Scopes::Unit,
        "pop" => Scopes::Pop,
        "family" => Scopes::Family,
        "party" => Scopes::Party,
        "religion" => Scopes::Religion,
        "culture" => Scopes::Culture,
        "job" => Scopes::Job,
        "culture_group" => Scopes::CultureGroup,
        "country_culture" => Scopes::CountryCulture,
        "area" => Scopes::Area,
        "state" => Scopes::State,
        "subunit" => Scopes::SubUnit,
        "governorship" => Scopes::Governorship,
        "region" => Scopes::Region,
        "deity" => Scopes::Deity,
        "great_work" => Scopes::GreatWork,
        "treasure" => Scopes::Treasure,
        "war" => Scopes::War,
        "legion" => Scopes::Legion,
        "levy_template" => Scopes::LevyTemplate,
        _ => return None,
    })
}

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
    if s.contains(Scopes::Color) {
        vec.push("color");
    }
    if s.contains(Scopes::Country) {
        vec.push("country");
    }
    if s.contains(Scopes::Character) {
        vec.push("character");
    }
    if s.contains(Scopes::Province) {
        vec.push("province");
    }
    if s.contains(Scopes::Siege) {
        vec.push("siege");
    }
    if s.contains(Scopes::Unit) {
        vec.push("unit");
    }
    if s.contains(Scopes::Pop) {
        vec.push("pop");
    }
    if s.contains(Scopes::Family) {
        vec.push("family");
    }
    if s.contains(Scopes::Party) {
        vec.push("party");
    }
    if s.contains(Scopes::Religion) {
        vec.push("religion");
    }
    if s.contains(Scopes::Culture) {
        vec.push("culture");
    }
    if s.contains(Scopes::Job) {
        vec.push("job");
    }
    if s.contains(Scopes::CultureGroup) {
        vec.push("culture group");
    }
    if s.contains(Scopes::CountryCulture) {
        vec.push("country culture");
    }
    if s.contains(Scopes::Area) {
        vec.push("area");
    }
    if s.contains(Scopes::State) {
        vec.push("state");
    }
    if s.contains(Scopes::SubUnit) {
        vec.push("subunit");
    }
    if s.contains(Scopes::Governorship) {
        vec.push("governorship");
    }
    if s.contains(Scopes::Region) {
        vec.push("region");
    }
    if s.contains(Scopes::Deity) {
        vec.push("deity");
    }
    if s.contains(Scopes::GreatWork) {
        vec.push("great_work");
    }
    if s.contains(Scopes::Treasure) {
        vec.push("treasure");
    }
    if s.contains(Scopes::War) {
        vec.push("war");
    }
    if s.contains(Scopes::Legion) {
        vec.push("legion");
    }
    if s.contains(Scopes::LevyTemplate) {
        vec.push("levy_template");
    }
    display_choices(f, &vec, "or")
}

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    // TODO: - imperator - add this when Item::Family exists
    // if scopes == Scopes::Family && data.item_exists(Item::Family, arg) {
    //     return Some("fam");
    // }
    // TODO: - imperator - add this when Item::Character exists
    // if scopes == Scopes::Character && data.item_exists(Item::Character, arg) {
    //     return Some("char");
    // }
    if scopes == Scopes::Party && data.item_exists(Item::PartyType, arg) {
        return Some("party");
    }
    if scopes == Scopes::Treasure && data.item_exists(Item::Treasure, arg) {
        return Some("treasure");
    }
    if scopes == Scopes::Region && data.item_exists(Item::Region, arg) {
        return Some("region");
    }
    if scopes == Scopes::Area && data.item_exists(Item::Area, arg) {
        return Some("area");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("culture");
    }
    if scopes == Scopes::Deity && data.item_exists(Item::Deity, arg) {
        return Some("deity");
    }
    if scopes == Scopes::Country {
        return Some("c");
    }
    if scopes == Scopes::Religion && data.item_exists(Item::Religion, arg) {
        return Some("religion");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::Province && data.item_exists(Item::Province, arg) {
        return Some("p");
    }
    None
}

#[inline]
pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_TO_SCOPE_MAP.get(name).copied()
}

static SCOPE_TO_SCOPE_MAP: Lazy<FnvHashMap<&'static str, (Scopes, Scopes)>> = Lazy::new(|| {
    let mut hash = FnvHashMap::default();
    for (from, s, to) in SCOPE_TO_SCOPE.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED VERSION 2.0.4
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Character, "character_party", Scopes::Party),
    (Scopes::Character, "employer", Scopes::Country),
    (Scopes::Character, "family", Scopes::Family),
    (Scopes::Character, "father", Scopes::Character),
    (Scopes::Character, "home_country", Scopes::Country),
    (Scopes::Character, "job", Scopes::Job),
    (Scopes::Character, "mother", Scopes::Character),
    (Scopes::Character, "next_in_family", Scopes::Character),
    (Scopes::Character, "preferred_heir", Scopes::Character),
    (Scopes::Character, "ruler", Scopes::Character),
    (Scopes::Character, "spouse", Scopes::Character),
    (
        Scopes::Treasure,
        "treasure_owner",
        Scopes::Country.union(Scopes::Character).union(Scopes::Province),
    ),
    (Scopes::Country, "color1", Scopes::Color),
    (Scopes::Country, "color2", Scopes::Color),
    (Scopes::Country, "color3", Scopes::Color),
    (Scopes::Country, "consort", Scopes::Character),
    (Scopes::Country, "current_co_ruler", Scopes::Character),
    (Scopes::Country, "current_heir", Scopes::Character),
    (Scopes::Country, "current_ruler", Scopes::Character),
    (Scopes::Country, "fam", Scopes::Family),
    (Scopes::Country, "overlord", Scopes::Country),
    (Scopes::Country.union(Scopes::Character), "party", Scopes::Party),
    (Scopes::Country, "primary_heir", Scopes::Character),
    (Scopes::Country, "secondary_heir", Scopes::Character),
    (Scopes::Character.union(Scopes::Pop).union(Scopes::Job), "country", Scopes::Country),
    (Scopes::Character.union(Scopes::Unit).union(Scopes::Governorship), "legion", Scopes::Legion),
    (
        Scopes::Country
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::Pop)
            .union(Scopes::Deity),
        "religion",
        Scopes::Religion,
    ),
    (Scopes::Party, "party_country", Scopes::Country),
    (Scopes::Party, "party_leader", Scopes::Character),
    (Scopes::Country.union(Scopes::Religion).union(Scopes::CultureGroup), "color", Scopes::Color),
    (Scopes::Siege, "siege_controller", Scopes::Country),
    (Scopes::Province.union(Scopes::State), "area", Scopes::Area),
    (Scopes::Province.union(Scopes::State), "governorship", Scopes::Governorship),
    (
        Scopes::Country.union(Scopes::Province).union(Scopes::Pop),
        "country_culture",
        Scopes::CountryCulture,
    ),
    (Scopes::Deity, "deified_ruler", Scopes::Character),
    (Scopes::Deity, "holy_site", Scopes::Province),
    (Scopes::Character.union(Scopes::Siege).union(Scopes::Pop), "location", Scopes::Province),
    (Scopes::Province.union(Scopes::Area).union(Scopes::State), "region", Scopes::Region),
    (Scopes::Color, "blue", Scopes::Value),
    (Scopes::Color, "brightness", Scopes::Value),
    (Scopes::Color, "green", Scopes::Value),
    (Scopes::Color, "hue", Scopes::Value),
    (Scopes::Color, "red", Scopes::Value),
    (Scopes::Color, "saturation", Scopes::Value),
    (Scopes::Unit, "commander", Scopes::Character),
    (Scopes::Unit, "unit_destination", Scopes::Province),
    (Scopes::Unit, "unit_location", Scopes::Province),
    (Scopes::Unit, "unit_next_location", Scopes::Province),
    (Scopes::Unit, "unit_objective_destination", Scopes::Province),
    (Scopes::Unit, "unit_owner", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::Pop)
            .union(Scopes::CountryCulture)
            .union(Scopes::Culture),
        "culture_group",
        Scopes::CultureGroup,
    ),
    (Scopes::SubUnit, "owning_unit", Scopes::Unit),
    (Scopes::SubUnit, "personal_loyalty", Scopes::Character),
    (Scopes::Job, "character", Scopes::Character),
    (
        Scopes::Province.union(Scopes::State).union(Scopes::Governorship).union(Scopes::Legion),
        "owner",
        Scopes::Country,
    ),
    (Scopes::Family, "family_country", Scopes::Country),
    (Scopes::Family, "head_of_family", Scopes::Character),
    (
        Scopes::Country
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::Pop)
            .union(Scopes::CultureGroup)
            .union(Scopes::CountryCulture),
        "culture",
        Scopes::Culture,
    ),
    (
        Scopes::Province.union(Scopes::State).union(Scopes::Governorship),
        "governor",
        Scopes::Character,
    ),
    (
        Scopes::Province.union(Scopes::State).union(Scopes::Governorship),
        "governor_or_ruler",
        Scopes::Character,
    ),
    (Scopes::War, "attacker_warleader", Scopes::Country),
    (Scopes::War, "defender_warleader", Scopes::Country),
    (Scopes::Province, "controller", Scopes::Country),
    (Scopes::Province, "dominant_province_culture", Scopes::Culture),
    (Scopes::Province, "dominant_province_culture_group", Scopes::CultureGroup),
    (Scopes::Province, "dominant_province_religion", Scopes::Religion),
    (Scopes::Province, "holding_owner", Scopes::Character),
    (Scopes::Province, "province_deity", Scopes::Deity),
    (Scopes::Province, "state", Scopes::State),
    (Scopes::Province.union(Scopes::Unit), "siege", Scopes::Siege),
    (
        Scopes::Country.union(Scopes::State).union(Scopes::Governorship),
        "capital_scope",
        Scopes::Province,
    ),
    (Scopes::None, "yes", Scopes::Bool),
    (Scopes::None, "no", Scopes::Bool),
];

#[inline]
pub fn scope_prefix(name: &str) -> Option<(Scopes, Scopes, ArgumentValue)> {
    SCOPE_PREFIX_MAP.get(name).copied()
}

static SCOPE_PREFIX_MAP: Lazy<FnvHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    Lazy::new(|| {
        let mut hash = FnvHashMap::default();
        for (from, s, to, argument) in SCOPE_PREFIX.iter().copied() {
            hash.insert(s, (from, to, argument));
        }
        hash
    });

/// LAST UPDATED VERSION 2.0.4
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)

// Basically just search the log for "Requires Data: yes" and put all that here.
const SCOPE_PREFIX: &[(Scopes, &str, Scopes, ArgumentValue)] = {
    use crate::item::Item;
    use ArgumentValue::*;
    // TODO - treasure and char need to be done when Character and Treasure types are implemented.
    &[
        (Scopes::None, "array_define", Scopes::Value, UncheckedValue),
        (Scopes::Country, "fam", Scopes::Family, UncheckedValue),
        (Scopes::Country, "party", Scopes::Party, Item(Item::PartyType)),
        (
            Scopes::Country
                .union(Scopes::Province)
                .union(Scopes::State)
                .union(Scopes::Governorship),
            "job",
            Scopes::Job,
            UncheckedValue,
        ),
        (
            Scopes::Country
                .union(Scopes::Province)
                .union(Scopes::State)
                .union(Scopes::Governorship),
            "job_holder",
            Scopes::Job,
            Item(Item::Office),
        ),
        (Scopes::Treasure, "treasure", Scopes::Treasure, UncheckedValue),
        (Scopes::None, "character", Scopes::Character, UncheckedValue),
        (Scopes::None, "region", Scopes::Region, Item(Item::Region)),
        (Scopes::None, "area", Scopes::Area, Item(Item::Area)),
        (Scopes::None, "culture", Scopes::Culture, Item(Item::Culture)),
        (Scopes::None, "culture_group", Scopes::CultureGroup, Item(Item::CultureGroup)),
        (Scopes::None, "deity", Scopes::Deity, Item(Item::Deity)),
        (Scopes::None, "c", Scopes::Country, UncheckedValue),
        (Scopes::None, "char", Scopes::Character, UncheckedValue),
        (Scopes::None, "define", Scopes::Value, UncheckedValue),
        (Scopes::None, "flag", Scopes::Flag, UncheckedValue),
        (Scopes::None, "global_var", Scopes::all(), UncheckedValue),
        (Scopes::None, "local_var", Scopes::all(), UncheckedValue),
        (Scopes::None, "p", Scopes::Province, UncheckedValue),
        (Scopes::None, "religion", Scopes::Religion, Item(Item::Religion)),
        (Scopes::None, "scope", Scopes::all(), UncheckedValue),
        (Scopes::all(), "var", Scopes::all(), UncheckedValue),
    ]
};

#[inline]
pub fn scope_iterator(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_ITERATOR_MAP.get(name).copied()
}

static SCOPE_ITERATOR_MAP: Lazy<FnvHashMap<&'static str, (Scopes, Scopes)>> = Lazy::new(|| {
    let mut hash = FnvHashMap::default();
    for (from, s, to) in SCOPE_ITERATOR.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED VERSION 2.0.4
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
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
    (Scopes::None, "region", Scopes::Province),
    (Scopes::None, "sea_and_river_zone", Scopes::Province),
];

pub fn scope_iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

/// LAST UPDATED VERSION 2.0.4
/// Every entry represents a every_, ordered_, random_, and any_ version.
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
