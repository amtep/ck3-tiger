#![allow(non_upper_case_globals)]

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::item::Item;
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

/// LAST UPDATED IR VERSION 2.0.4
/// See `event_scopes.log` from the game data dumps.
pub const None: u64 = 0x0000_0001;
pub const Value: u64 = 0x0000_0002;
pub const Bool: u64 = 0x0000_0004;
pub const Flag: u64 = 0x0000_0008;
pub const Color: u64 = 0x0000_0010;
pub const Country: u64 = 0x0000_0020;
pub const Character: u64 = 0x0000_0040;
pub const Province: u64 = 0x0000_0080;
pub const Siege: u64 = 0x0000_0100;
pub const Unit: u64 = 0x0000_0200;
pub const Pop: u64 = 0x0000_0400;
pub const Family: u64 = 0x0000_0800;
pub const Party: u64 = 0x0000_1000;
pub const Religion: u64 = 0x0000_2000;
pub const Culture: u64 = 0x0000_4000;
pub const Job: u64 = 0x0000_8000;
pub const CultureGroup: u64 = 0x0001_0000;
pub const CountryCulture: u64 = 0x0002_0000;
pub const Area: u64 = 0x0004_0000;
pub const State: u64 = 0x0008_0000;
pub const SubUnit: u64 = 0x0010_0000;
pub const Governorship: u64 = 0x0020_0000;
pub const Region: u64 = 0x0040_0000;
pub const Deity: u64 = 0x0080_0000;
pub const GreatWork: u64 = 0x0100_0000;
pub const Treasure: u64 = 0x0200_0000;
pub const War: u64 = 0x0400_0000;
pub const Legion: u64 = 0x0800_0000;
pub const LevyTemplate: u64 = 0x1000_0000;

pub const ALL: u64 = 0x7fff_ffff_ffff_ffff;
pub const ALL_BUT_NONE: u64 = 0x7fff_ffff_ffff_fffe;
#[allow(dead_code)]
pub const PRIMITIVE: u64 = 0x0000_000e;

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
        "subunit" => Scopes::Subunit,
        "governorship" => Scopes::Governorship,
        "region" => Scopes::Region,
        "deity" => Scopes::Deity,
        "great_work" => Scopes::GreatWork,
        "treasure" => Scopes::Treasure,
        "war" => Scopes::War,
        "legion" => Scopes::Legion,
        "levy_template" => Scopes::LevyTemplate,
        _ => return std::option::Option::None,
    })
}

pub fn scope_to_scope(name: &Token) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if name.is(s) {
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
        }
    }
    for (s, version, explanation) in SCOPE_TO_SCOPE_REMOVED {
        if name.is(s) {
            let msg = format!("`{name}` was removed in {version}");
            warn_info(name, ErrorKey::Removed, &msg, explanation);
            return Some((Scopes::all(), Scopes::all_but_none()));
        }
    }
    std::option::Option::None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
        }
    }
    std::option::Option::None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_ITERATOR {
        if name.is(s) {
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
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
    std::option::Option::None
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
            if self.contains(Scopes::Subunit) {
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
const SCOPE_TO_SCOPE: &[(u64, &str, u64)] = &[
    (Character, "character_party", Party),
    (Character, "employer", Country),
    (Character, "family", Family),
    (Character, "father", Character),
    (Character, "home_country", Country),
    (Character, "job", Job),
    (Character, "mother", Character),
    (Character, "next_in_family", Character),
    (Character, "preferred_heir", Character),
    (Character, "ruler", Character),
    (Character, "spouse", Character),
    (Treasure, "treasure_owner", Country | Character | Province),
    (Country, "color1", Color),
    (Country, "color2", Color),
    (Country, "color3", Color),
    (Country, "consort", Character),
    (Country, "current_co_ruler", Character),
    (Country, "current_heir", Character),
    (Country, "current_ruler", Character),
    (Country, "fam", Family),
    (Country, "overlord", Country),
    (Country, "party", Party),
    (Country, "primary_heir", Character),
    (Country, "secondary_heir", Character),
    (Character | Pop | Job, "country", Country),
    (Character | Unit | Governorship, "legion", Legion),
    (Country | Character | Province | Pop | Deity, "religion", Religion),
    (Party, "party_country", Country),
    (Party, "party_leader", Character),
    (Country | Religion | CultureGroup, "color", Color),
    (Siege, "siege_controller", Country),
    (Province | State, "area", Area),
    (Province | State, "governorship", Governorship),
    (Country | Province | Pop, "country_culture", CountryCulture),
    (Deity, "deified_ruler", Character),
    (Deity, "holy_site", Province),
    (Character | Siege | Pop, "location", Province),
    (Province | Area | State, "region", Region),
    (Color, "blue", Value),
    (Color, "brightness", Value),
    (Color, "green", Value),
    (Color, "hue", Value),
    (Color, "red", Value),
    (Color, "saturation", Value),
    (Unit, "commander", Character),
    (Unit, "unit_destination", Province),
    (Unit, "unit_location", Province),
    (Unit, "unit_next_location", Province),
    (Unit, "unit_objective_destination", Province),
    (Unit, "unit_owner", Country),
    (Country | Character | Province | Pop | Culture, "culture_group", CultureGroup),
    (Subunit, "owning_unit", Unit),
    (Subunit, "personal_loyalty", Character),
    (Job, "character", Character),
    (Province | State | Governorship | Legion, "owner", Country),
    (Family, "family_country", Country),
    (Family, "head_of_family", Country),
    (Country | Character | Province | Pop | CountryCulture, "culture", Culture),
    (Province | State | Governorship, "governor", Character),
    (Province | State | Governorship, "governor_or_ruler", Character),
    (War, "attacker_warleader", Country),
    (War, "defender_warleader", Country),
    (Province, "controller", Country),
    (Province, "dominant_province_culture", Culture),
    (Province, "dominant_province_culture_group", CultureGroup),
    (Province, "dominant_province_religion", Religion),
    (Province, "holding_owner", Character),
    (Province, "province_deity", Deity),
    (Province, "state", State),
    (Province | Unit, "siege", Siege),
    (Country | State | Governorship, "capital_scope", Province),
    (None, "yes", Bool),
    (None, "no", Bool),
];

/// LAST UPDATED VERSION 2.0.4
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
/// TODO: add the Item type here, so that it can be checked for existence.

// Basically just search the log for "Requires Data: yes" and put all that here.
const SCOPE_FROM_PREFIX: &[(u64, &str, u64)] = &[
    (None, "array_define", Value),
    (Country, "fam", Family),
    (Country, "party", Party),
    (Country | Province | State | Governorship, "job", Job),
    (Country | Province | State | Governorship, "job_holder", Job),
    (Treasure, "treasure", Treasure),
    (None, "character", Character),
    (None, "region", Region),
    (None, "area", Area),
    (None, "culture", Culture),
    (None, "deity", Deity),
    (None, "c", Country),
    (None, "char", Character),
    (None, "define", Value),
    (None, "religion", Religion),
    (None, "flag", Flag),
    (None, "global_var", ALL),
    (None, "local_var", ALL),
    (None, "p", Province),
    (None, "religion", Religion),
    (None, "scope", ALL),
    (ALL, "var", ALL),
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
const SCOPE_ITERATOR: &[(u64, &str, u64)] = &[
    (State, "state_province", Province),
    (Character, "character_treasure", Treasure),
    (Character, "character_unit", Unit),
    (Character, "child", Character),
    (Character, "friend", Character),
    (Character, "governor_state", State),
    (Character, "holdings", Province),
    (Character, "parent", Character),
    (Character, "rival", Character),
    (Character, "sibling", Character),
    (Character, "support_as_heir", Character),
    (Governorship, "governorship_state", State),
    (Country, "allied_country", Country),
    (Country, "army", Unit),
    (Country, "available_deity", Deity),
    (Country, "character", Character),
    (Country, "commander", Character),
    (Country, "countries_at_war_with", Country),
    (Country, "country_culture", CountryCulture),
    (Country, "country_state", State),
    (Country, "country_sub_unit", Subunit),
    (Country, "country_treasure", Treasure),
    (Country, "current_war", War),
    (Country, "family", Family),
    (Country, "governorships", Governorship),
    (Country, "integrated_culture", CountryCulture),
    (Country, "legion", Legion),
    (Country, "navy", Unit),
    (Country, "neighbour_country", Country),
    (Country, "owned_holy_site", Province),
    (Country, "owned_province", Province),
    (Country, "pantheon_deity", Deity),
    (Country, "party", Party),
    (Country, "subject", Country),
    (Country, "successor", Character),
    (Country, "unit", Unit),
    (Party, "party_member", Character),
    (Legion, "legion_commander", Character),
    (Legion, "legion_unit", Unit),
    (Region, "neighbor_region", Region),
    (Region, "region_area", Area),
    (Region, "region_province", Province),
    (Region, "region_province_including_unownable", Province),
    (Region, "region_state", State),
    (Area, "area_including_unownable_province", Province),
    (Area, "area_province", Province),
    (Area, "area_state", State),
    (Area, "neighbor_area", Area),
    (Unit, "sub_unit", Subunit),
    (Family, "family_member", Character),
    (War, "war_attacker", Country),
    (War, "war_defender", Country),
    (War, "war_participant", Country),
    (Province, "great_work_in_province", GreatWork),
    (Province, "neighbor_province", Province),
    (Province, "pops_in_province", Pop),
    (Province, "province_treasure", Treasure),
    (Province, "unit_in_province", Unit),
    (None, "active_war", War),
    (None, "area", Area),
    (None, "country", Country),
    (None, "deity", Deity),
    (None, "ended_war", War),
    (None, "holy_site", Province),
    (None, "living_character", Character),
    (None, "ownable_province", Province),
    (None, "province", Province),
    (None, "region", Province),
    (None, "sea_and_river_zone", Province),
];

/// LAST UPDATED VERSION 2.0.4
/// Every entry represents a every_, ordered_, random_, and any_ version.
const SCOPE_REMOVED_ITERATOR: &[(&str, &str, &str)] = &[

];

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[

];
