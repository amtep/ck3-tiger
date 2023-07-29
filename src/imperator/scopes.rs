#![allow(non_upper_case_globals)]

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::report::{warn_info, ErrorKey};
use crate::token::Token;

bitflags! {
    /// LAST UPDATED IR VERSION 2.0.4
    /// See `event_scopes.log` from the game data dumps.
    /// Keep in sync with the module constants below.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Scopes: u64 {
        const None = 0x0000_0001;
        const Value = 0x0000_0002;
        const Bool = 0x0000_0004;
        const Flag = 0x0000_0008;
        const Color = 0x0000_0010;
        const Country = 0x0000_0020;
        const Character = 0x0000_0040;
        const Province = 0x0000_0080;
        const Siege = 0x0000_0100;
        const Unit = 0x0000_0200;
        const Pop = 0x0000_0400;
        const Family = 0x0000_0800;
        const Party = 0x0000_1000;
        const Religion = 0x0000_2000;
        const Culture = 0x0000_4000;
        const Job = 0x0000_8000;
        const CultureGroup = 0x0001_0000;
        const CountryCulture = 0x0002_0000;
        const Area = 0x0004_0000;
        const State = 0x0008_0000;
        const SubUnit = 0x0010_0000;
        const Governorship = 0x0020_0000;
        const Region = 0x0040_0000;
        const Deity = 0x0080_0000;
        const GreatWork = 0x0100_0000;
        const Treasure = 0x0200_0000;
        const War = 0x0400_0000;
        const Legion = 0x0800_0000;
        const LevyTemplate = 0x1000_0000;
    }
}

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
        "culture group" => Scopes::CultureGroup,
        "country culture" => Scopes::CountryCulture,
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

pub fn scope_to_scope(name: &Token, _inscopes: Scopes) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if name.is(s) {
            return Some((*from, *to));
        }
    }
    for (s, version, explanation) in SCOPE_TO_SCOPE_REMOVED {
        if name.is(s) {
            let msg = format!("`{name}` was removed in {version}");
            warn_info(name, ErrorKey::Removed, &msg, explanation);
            return Some((Scopes::all(), Scopes::all_but_none()));
        }
    }
    None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((*from, *to));
        }
    }
    None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_ITERATOR {
        if name.is(s) {
            return Some((*from, *to));
        }
    }
    for (s, version, explanation) in SCOPE_REMOVED_ITERATOR {
        if name.is(s) {
            let msg = format!("`{name}` iterators were removed in {version}");
            warn_info(name, ErrorKey::Removed, &msg, explanation);
            return Some((Scopes::all(), Scopes::all()));
        }
    }
    if data.scripted_lists.exists(name.as_str()) {
        return data.scripted_lists.base(name).and_then(|base| scope_iterator(base, data));
    }
    None
}

impl Display for Scopes {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if *self == Scopes::all() {
            write!(f, "any scope")
        } else if *self == Scopes::primitive() {
            write!(f, "any primitive scope")
        } else if *self == Scopes::non_primitive() {
            write!(f, "non-primitive scope")
        } else if *self == Scopes::all_but_none() {
            write!(f, "any except none scope")
        } else {
            let mut vec = Vec::new();
            if self.contains(Scopes::None) {
                vec.push("none")
            }
            if self.contains(Scopes::Value) {
                vec.push("value")
            }
            if self.contains(Scopes::Bool) {
                vec.push("bool")
            }
            if self.contains(Scopes::Flag) {
                vec.push("flag")
            }
            if self.contains(Scopes::Color) {
                vec.push("color")
            }
            if self.contains(Scopes::Country) {
                vec.push("country")
            }
            if self.contains(Scopes::Character) {
                vec.push("character")
            }
            if self.contains(Scopes::Province) {
                vec.push("province")
            }
            if self.contains(Scopes::Siege) {
                vec.push("siege")
            }
            if self.contains(Scopes::Unit) {
                vec.push("unit")
            }
            if self.contains(Scopes::Pop) {
                vec.push("pop")
            }
            if self.contains(Scopes::Family) {
                vec.push("family")
            }
            if self.contains(Scopes::Party) {
                vec.push("party")
            }
            if self.contains(Scopes::Religion) {
                vec.push("religion")
            }
            if self.contains(Scopes::Culture) {
                vec.push("culture")
            }
            if self.contains(Scopes::Job) {
                vec.push("job")
            }
            if self.contains(Scopes::CultureGroup) {
                vec.push("culture group")
            }
            if self.contains(Scopes::CountryCulture) {
                vec.push("country culture")
            }
            if self.contains(Scopes::Area) {
                vec.push("area")
            }
            if self.contains(Scopes::State) {
                vec.push("state")
            }
            if self.contains(Scopes::SubUnit) {
                vec.push("subunit")
            }
            if self.contains(Scopes::Governorship) {
                vec.push("governorship")
            }
            if self.contains(Scopes::Region) {
                vec.push("region")
            }
            if self.contains(Scopes::Deity) {
                vec.push("deity")
            }
            if self.contains(Scopes::GreatWork) {
                vec.push("great_work")
            }
            if self.contains(Scopes::Treasure) {
                vec.push("treasure")
            }
            if self.contains(Scopes::War) {
                vec.push("war")
            }
            if self.contains(Scopes::Legion) {
                vec.push("legion")
            }
            if self.contains(Scopes::LevyTemplate) {
                vec.push("levy_template")
            }
            display_choices(f, &vec, "or")
        }
    }
}

pub fn validate_prefix_reference(
    prefix: &Token,
    arg: &Token,
    data: &Everything,
    _sc: &mut ScopeContext,
) {
    // DEMENTIVE - TODO add these once all Item types have been implmented
    match prefix.as_str() {
        // "accolade_type" => data.verify_exists(Item::AccoladeType, arg),
        &_ => (),
    }
}

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
    (Scopes::Country, "party", Scopes::Party),
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
    (Scopes::Family, "head_of_family", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::Pop)
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

/// LAST UPDATED VERSION 2.0.4
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
/// TODO: add the Item type here, so that it can be checked for existence.

// Basically just search the log for "Requires Data: yes" and put all that here.
const SCOPE_FROM_PREFIX: &[(Scopes, &str, Scopes)] = &[
    (Scopes::None, "array_define", Scopes::Value),
    (Scopes::Country, "fam", Scopes::Family),
    (Scopes::Country, "party", Scopes::Party),
    (
        Scopes::Country.union(Scopes::Province).union(Scopes::State).union(Scopes::Governorship),
        "job",
        Scopes::Job,
    ),
    (
        Scopes::Country.union(Scopes::Province).union(Scopes::State).union(Scopes::Governorship),
        "job_holder",
        Scopes::Job,
    ),
    (Scopes::Treasure, "treasure", Scopes::Treasure),
    (Scopes::None, "character", Scopes::Character),
    (Scopes::None, "region", Scopes::Region),
    (Scopes::None, "area", Scopes::Area),
    (Scopes::None, "culture", Scopes::Culture),
    (Scopes::None, "deity", Scopes::Deity),
    (Scopes::None, "c", Scopes::Country),
    (Scopes::None, "char", Scopes::Character),
    (Scopes::None, "define", Scopes::Value),
    (Scopes::None, "religion", Scopes::Religion),
    (Scopes::None, "flag", Scopes::Flag),
    (Scopes::None, "global_var", Scopes::all()),
    (Scopes::None, "local_var", Scopes::all()),
    (Scopes::None, "p", Scopes::Province),
    (Scopes::None, "religion", Scopes::Religion),
    (Scopes::None, "scope", Scopes::all()),
    (Scopes::all(), "var", Scopes::all()),
];

// Special:
// <lifestyle>_perk_points
// <lifestyle>_perks
// <lifestyle>_unlockable_perks
// <lifestyle>_xp
//
// TODO Special:
// <legacy>_track_perks

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

/// LAST UPDATED VERSION 2.0.4
/// Every entry represents a every_, ordered_, random_, and any_ version.
const SCOPE_REMOVED_ITERATOR: &[(&str, &str, &str)] = &[];

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
