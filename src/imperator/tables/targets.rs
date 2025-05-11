use std::sync::LazyLock;

use crate::helpers::TigerHashMap;
use crate::scopes::{ArgumentValue, Scopes};

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
    (Scopes::Unit.union(Scopes::Legion), "commander", Scopes::Character),
    (Scopes::Unit.union(Scopes::Legion), "unit_destination", Scopes::Province),
    (Scopes::Unit.union(Scopes::Legion), "unit_location", Scopes::Province),
    (Scopes::Unit.union(Scopes::Legion), "unit_next_location", Scopes::Province),
    (Scopes::Unit.union(Scopes::Legion), "unit_objective_destination", Scopes::Province),
    (Scopes::Unit.union(Scopes::Legion), "unit_owner", Scopes::Country),
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

static SCOPE_PREFIX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
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
            Scopes::Character,
            Item(Item::Office),
        ),
        (Scopes::None, "treasure", Scopes::Treasure, UncheckedValue),
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

pub fn scope_to_scope_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_TO_SCOPE_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
