#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::item::Item;
use crate::token::Token;

bitflags! {
    /// LAST UPDATED VERSION 1.6.2.2
    /// See `event_scopes.log` from the game data dumps.
    /// Keep in sync with the module constants below.
    pub struct Scopes: u32 {
        const None = 0x0000_0001;
        const Value = 0x0000_0002;
        const Bool = 0x0000_0004;
        const Flag = 0x0000_0008;
        const Character = 0x0000_0010;
        const LandedTitle = 0x0000_0020;
        const Activity = 0x0000_0040;
        const Secret = 0x0000_0080;
        const Province = 0x0000_0100;
        const Scheme = 0x0000_0200;
        const Combat = 0x0000_0400;
        const CombatSide = 0x0000_0800;
        const TitleAndVassalChange = 0x0000_1000;
        const Faith = 0x0000_2000;
        const GreatHolyWar = 0x0000_4000;
        const Religion = 0x0000_8000;
        const War = 0x0001_0000;
        const StoryCycle = 0x0002_0000;
        const CasusBelli = 0x0004_0000;
        const Dynasty = 0x0008_0000;
        const DynastyHouse = 0x0010_0000;
        const Faction = 0x0020_0000;
        const Culture = 0x0040_0000;
        const Army = 0x0080_0000;
        const HolyOrder = 0x0100_0000;
        const CouncilTask = 0x0200_0000;
        const MercenaryCompany = 0x0400_0000;
        const Artifact = 0x0800_0000;
        const Inspiration = 0x1000_0000;
        const Struggle = 0x2000_0000;
    }
}

impl Scopes {
    pub fn expect_scope(&mut self, key: &Token, expect: Scopes) {
        if self.intersects(expect) {
            *self &= expect;
        } else {
            let msg = format!(
                "{} is for {} but scope seems to be {}",
                key,
                Scopes::Character,
                self
            );
            warn(key, ErrorKey::Scopes, &msg);
        }
    }

    pub fn non_primitive() -> Scopes {
        Scopes::all() ^ (Scopes::None | Scopes::Value | Scopes::Bool | Scopes::Flag)
    }
}

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_scopes.log` from the game data dumps.
const None: u32 = 0x0000_0001;
const Value: u32 = 0x0000_0002;
const Bool: u32 = 0x0000_0004;
const Flag: u32 = 0x0000_0008;
const Character: u32 = 0x0000_0010;
const LandedTitle: u32 = 0x0000_0020;
const Activity: u32 = 0x0000_0040;
const Secret: u32 = 0x0000_0080;
const Province: u32 = 0x0000_0100;
const Scheme: u32 = 0x0000_0200;
const Combat: u32 = 0x0000_0400;
const CombatSide: u32 = 0x0000_0800;
const TitleAndVassalChange: u32 = 0x0000_1000;
const Faith: u32 = 0x0000_2000;
const GreatHolyWar: u32 = 0x0000_4000;
const Religion: u32 = 0x0000_8000;
const War: u32 = 0x0001_0000;
const StoryCycle: u32 = 0x0002_0000;
const CasusBelli: u32 = 0x0004_0000;
const Dynasty: u32 = 0x0008_0000;
const DynastyHouse: u32 = 0x0010_0000;
const Faction: u32 = 0x0020_0000;
const Culture: u32 = 0x0040_0000;
const Army: u32 = 0x0080_0000;
const HolyOrder: u32 = 0x0100_0000;
const CouncilTask: u32 = 0x0200_0000;
const MercenaryCompany: u32 = 0x0400_0000;
const Artifact: u32 = 0x0800_0000;
const Inspiration: u32 = 0x1000_0000;
const Struggle: u32 = 0x2000_0000;
const ALL: u32 = 0x3fff_ffff;

pub fn scope_from_snake_case(s: &str) -> Option<Scopes> {
    Some(match s {
        "none" => Scopes::None,
        "value" => Scopes::Value,
        "bool" => Scopes::Bool,
        "flag" => Scopes::Flag,
        "character" => Scopes::Character,
        "landed_title" => Scopes::LandedTitle,
        "activity" => Scopes::Activity,
        "secret" => Scopes::Secret,
        "province" => Scopes::Province,
        "scheme" => Scopes::Scheme,
        "combat" => Scopes::Combat,
        "combat_side" => Scopes::CombatSide,
        "title_and_vassal_change" => Scopes::TitleAndVassalChange,
        "faith" => Scopes::Faith,
        "ghw" => Scopes::GreatHolyWar, // Warning, this is an exception to the general rule
        "religion" => Scopes::Religion,
        "war" => Scopes::War,
        "story_cycle" => Scopes::StoryCycle,
        "casus_belli" => Scopes::CasusBelli,
        "dynasty" => Scopes::Dynasty,
        "dynasty_house" => Scopes::DynastyHouse,
        "faction" => Scopes::Faction,
        "culture" => Scopes::Culture,
        "army" => Scopes::Army,
        "holy_order" => Scopes::HolyOrder,
        "council_task" => Scopes::CouncilTask,
        "mercenary_company" => Scopes::MercenaryCompany,
        "artifact" => Scopes::Artifact,
        "inspiration" => Scopes::Inspiration,
        "struggle" => Scopes::Struggle,
        _ => return std::option::Option::None,
    })
}

pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if *s == name {
            return Some((
                Scopes::from_bits_truncate(*from),
                Scopes::from_bits_truncate(*to),
            ));
        }
    }
    std::option::Option::None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((
                Scopes::from_bits_truncate(*from),
                Scopes::from_bits_truncate(*to),
            ));
        }
    }
    std::option::Option::None
}

pub fn scope_value(name: &Token, data: &Everything) -> Option<Scopes> {
    for (from, s) in SCOPE_VALUE {
        if name.is(s) {
            return Some(Scopes::from_bits_truncate(*from));
        }
    }
    if let Some(relation) = name.as_str().strip_prefix("num_of_relation_") {
        if data.relations.exists(relation) {
            return Some(Scopes::Character);
        }
    } else if let Some(lifestyle) = name.as_str().strip_prefix("perks_in_") {
        if data.lifestyles.exists(lifestyle) {
            return Some(Scopes::Character);
        }
    } else if let Some(lifestyle) = name.as_str().strip_suffix("_perk_points") {
        if data.lifestyles.exists(lifestyle) {
            return Some(Scopes::Character);
        }
    } else if let Some(lifestyle) = name.as_str().strip_suffix("_perks") {
        if data.lifestyles.exists(lifestyle) {
            return Some(Scopes::Character);
        }
    } else if let Some(lifestyle) = name.as_str().strip_suffix("_unlockable_perks") {
        if data.lifestyles.exists(lifestyle) {
            return Some(Scopes::Character);
        }
    } else if let Some(lifestyle) = name.as_str().strip_suffix("_xp") {
        if data.lifestyles.exists(lifestyle) {
            return Some(Scopes::Character);
        }
    }
    std::option::Option::None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_ITERATOR {
        if name.is(s) {
            return Some((
                Scopes::from_bits_truncate(*from),
                Scopes::from_bits_truncate(*to),
            ));
        }
    }
    if data.scripted_lists.exists(name.as_str()) {
        return data
            .scripted_lists
            .base(name)
            .and_then(|name| scope_iterator(name, data));
    }
    std::option::Option::None
}

pub fn scope_trigger_target(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TRIGGER_TARGET {
        if name.is(s) {
            return Some((
                Scopes::from_bits_truncate(*from),
                Scopes::from_bits_truncate(*to),
            ));
        }
    }
    if let Some(relation) = name.as_str().strip_prefix("has_relation_") {
        if data.relations.exists(relation) {
            return Some((Scopes::Character, Scopes::Character));
        }
    }
    if let Some(relation) = name.as_str().strip_prefix("has_secret_relation_") {
        if data.relations.exists(relation) {
            return Some((Scopes::Character, Scopes::Character));
        }
    }
    std::option::Option::None
}

pub fn scope_trigger_bool(name: &str) -> Option<Scopes> {
    for (from, s) in SCOPE_TRIGGER_BOOL {
        if *s == name {
            return Some(Scopes::from_bits_truncate(*from));
        }
    }
    std::option::Option::None
}

pub fn scope_trigger_item(name: &str) -> Option<(Scopes, Item)> {
    for (from, s, item) in SCOPE_TRIGGER_ITEM {
        if *s == name {
            return Some((Scopes::from_bits_truncate(*from), *item));
        }
    }
    std::option::Option::None
}

impl Display for Scopes {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if *self == Scopes::all() {
            write!(f, "any scope")
        } else if *self == Scopes::non_primitive() {
            write!(f, "non-primitive scope")
        } else {
            let mut vec = Vec::new();
            if self.contains(Scopes::None) {
                vec.push("none");
            }
            if self.contains(Scopes::Value) {
                vec.push("value");
            }
            if self.contains(Scopes::Bool) {
                vec.push("bool");
            }
            if self.contains(Scopes::Flag) {
                vec.push("flag");
            }
            if self.contains(Scopes::Character) {
                vec.push("character");
            }
            if self.contains(Scopes::LandedTitle) {
                vec.push("landed title");
            }
            if self.contains(Scopes::Activity) {
                vec.push("activity");
            }
            if self.contains(Scopes::Secret) {
                vec.push("secret");
            }
            if self.contains(Scopes::Province) {
                vec.push("province");
            }
            if self.contains(Scopes::Scheme) {
                vec.push("scheme");
            }
            if self.contains(Scopes::Combat) {
                vec.push("combat");
            }
            if self.contains(Scopes::CombatSide) {
                vec.push("combat side");
            }
            if self.contains(Scopes::TitleAndVassalChange) {
                vec.push("title and vassal change");
            }
            if self.contains(Scopes::Faith) {
                vec.push("faith");
            }
            if self.contains(Scopes::GreatHolyWar) {
                vec.push("great holy war");
            }
            if self.contains(Scopes::Religion) {
                vec.push("religion");
            }
            if self.contains(Scopes::War) {
                vec.push("war");
            }
            if self.contains(Scopes::StoryCycle) {
                vec.push("story cycle");
            }
            if self.contains(Scopes::CasusBelli) {
                vec.push("casus belli");
            }
            if self.contains(Scopes::Dynasty) {
                vec.push("dynasty");
            }
            if self.contains(Scopes::DynastyHouse) {
                vec.push("dynasty house");
            }
            if self.contains(Scopes::Faction) {
                vec.push("faction");
            }
            if self.contains(Scopes::Culture) {
                vec.push("culture");
            }
            if self.contains(Scopes::Army) {
                vec.push("army");
            }
            if self.contains(Scopes::HolyOrder) {
                vec.push("holy order");
            }
            if self.contains(Scopes::CouncilTask) {
                vec.push("council task");
            }
            if self.contains(Scopes::MercenaryCompany) {
                vec.push("mercenary company");
            }
            if self.contains(Scopes::Artifact) {
                vec.push("artifact");
            }
            if self.contains(Scopes::Inspiration) {
                vec.push("inspiration");
            }
            if self.contains(Scopes::Struggle) {
                vec.push("struggle");
            }
            for i in 0..vec.len() {
                write!(f, "{}", vec[i])?;
                if i + 1 == vec.len() {
                } else if i + 2 == vec.len() {
                    write!(f, " or ")?;
                } else {
                    write!(f, ", ")?;
                }
            }
            Ok(())
        }
    }
}

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(u32, &str, u32)] = &[
    (Faction, "faction_leader", Character),
    (Faction, "faction_target", Character),
    (Faction, "faction_war", War),
    (Faction, "special_character", Character),
    (Faction, "special_title", LandedTitle),
    (War, "casus_belli", CasusBelli),
    (Activity, "activity_owner", Character),
    (Activity, "activity_province", Province),
    (Army, "army_commander", Character),
    (Army, "army_owner", Character),
    (Artifact, "artifact_age", Value),
    (Artifact, "artifact_owner", Character),
    (Artifact, "creator", Character),
    (Artifact, "previous_owner", Character),
    (Artifact, "previous_owner_level_2", Character),
    (Artifact, "previous_owner_level_3", Character),
    (LandedTitle, "capital_vassal", LandedTitle),
    (LandedTitle, "current_heir", Character),
    (LandedTitle, "de_facto_liege", LandedTitle),
    (LandedTitle, "de_jure_liege", LandedTitle),
    (LandedTitle, "holder", Character),
    (LandedTitle, "lessee", Character),
    (LandedTitle, "lessee_title", LandedTitle),
    (LandedTitle, "previous_holder", Character),
    (LandedTitle, "title_capital_county", LandedTitle),
    (LandedTitle, "title_province", Province),
    (GreatHolyWar, "ghw_designated_winner", Character),
    (GreatHolyWar, "ghw_target_character", Character),
    (GreatHolyWar, "ghw_target_title", LandedTitle),
    (GreatHolyWar, "ghw_title_recipient", Character),
    (GreatHolyWar, "ghw_war", War),
    (GreatHolyWar, "ghw_war_declarer", Character),
    (Province, "province_owner", Character),
    (LandedTitle | Province, "barony", LandedTitle),
    (LandedTitle | Province, "barony_controller", Character),
    (LandedTitle | Province, "county", LandedTitle),
    (LandedTitle | Province, "county_controller", Character),
    (LandedTitle | Province, "duchy", LandedTitle),
    (LandedTitle | Province, "empire", LandedTitle),
    (LandedTitle | Province, "kingdom", LandedTitle),
    (Scheme, "scheme_artifact", Artifact),
    (Scheme, "scheme_defender", Character),
    (Scheme, "scheme_owner", Character),
    (Scheme, "scheme_target", Character),
    (Character | Combat | Army, "location", Province),
    (CouncilTask, "councillor", Character),
    (HolyOrder, "holy_order_patron", Character),
    (HolyOrder, "leader", Character),
    (HolyOrder, "title", LandedTitle),
    (War | CasusBelli, "claimant", Character),
    (War | CasusBelli, "primary_attacker", Character),
    (War | CasusBelli, "primary_defender", Character),
    (Character, "activity", Activity),
    (Character, "betrothed", Character),
    (Character, "capital_barony", LandedTitle),
    (Character, "capital_county", LandedTitle),
    (Character, "capital_province", Province),
    (Character, "commanding_army", Army),
    (Character, "concubinist", Character),
    (Character, "council_task", CouncilTask), // also has a prefix form
    (Character, "councillor_task_target", ALL), // output scope depends on task
    (Character, "court_owner", Character),
    (Character, "designated_heir", Character),
    (Character, "dynasty", Dynasty),
    (Character, "employer", Character),
    (Character, "father", Character),
    (Character, "ghw_beneficiary", Character),
    (Character, "host", Character),
    (Character, "house", DynastyHouse),
    (Character, "imprisoner", Character),
    (Character, "inspiration", Inspiration),
    (Character, "joined_faction", Faction),
    (Character, "killer", Character),
    (Character, "knight_army", Army),
    (Character, "last_played_character", Character),
    (Character, "liege", Character),
    (Character, "liege_or_court_owner", Character),
    (Character, "matchmaker", Character),
    (Character, "mother", Character),
    (Character, "player_heir", Character),
    (Character, "pregnancy_assumed_father", Character),
    (Character, "pregnancy_real_father", Character),
    (Character, "primary_heir", Character),
    (Character, "primary_partner", Character),
    (Character, "primary_spouse", Character),
    (Character, "primary_title", LandedTitle),
    (Character, "real_father", Character),
    (Character, "realm_priest", Character),
    (Character, "top_liege", Character),
    (DynastyHouse, "house_founder", Character),
    (DynastyHouse, "house_head", Character),
    (DynastyHouse, "last_house_head", Character),
    (Combat, "combat_attacker", CombatSide),
    (Combat, "combat_defender", CombatSide),
    (Combat, "combat_war", War),
    (CombatSide, "combat", Combat),
    (CombatSide, "enemy_side", CombatSide),
    (CombatSide, "side_commander", Character),
    (CombatSide, "side_primary_participant", Character),
    (
        Character | LandedTitle | Province | GreatHolyWar,
        "faith",
        Faith,
    ),
    (Character | LandedTitle | Province, "culture", Culture),
    (
        Character | LandedTitle | Province | Faith | GreatHolyWar,
        "religion",
        Religion,
    ),
    (CasusBelli, "war", War),
    (Culture, "calc_culture_dominant_faith", Faith),
    (Culture, "calc_culture_dominant_religion", Religion),
    (Culture, "culture_head", Character),
    (StoryCycle, "story_owner", Character),
    (Faith, "founder", Character),
    (Faith, "great_holy_war", GreatHolyWar),
    (Faith, "religious_head", Character),
    (Faith, "religious_head_title", LandedTitle),
    (Inspiration, "inspiration_owner", Character),
    (Inspiration, "inspiration_sponsor", Character),
    (Secret, "secret_owner", Character),
    (Secret, "secret_target", Character),
    (Dynasty, "dynast", Character),
    (None, "dummy_female", Character),
    (None, "dummy_male", Character),
    // named_script_value special
    (None, "no", Bool),
    // "prev" special
    // "root" special
    // "this" special
    (None, "yes", Bool),
    (Value, "compare_value", Value), // special
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_FROM_PREFIX: &[(u32, &str, u32)] = &[
    (Character, "vassal_contract_obligation_level", Value),
    (Character, "aptitude", Value),
    (Character, "council_task", CouncilTask),
    (Character, "court_position", Character),
    (Character, "cp", Character), // councillor
    (None, "array_define", Value),
    (None, "character", Character),
    (None, "culture", Culture),
    (None, "define", Value),
    (None, "dynasty", Dynasty),
    (None, "event_id", Flag),
    (None, "faith", Faith),
    (None, "flag", Flag),
    (None, "global_var", ALL),
    (None, "house", DynastyHouse),
    (None, "local_var", ALL),
    (None, "province", Province),
    (None, "religion", Religion),
    (None, "scope", ALL),
    (None, "struggle", Struggle),
    (None, "title", LandedTitle),
    (ALL, "var", ALL),
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` from the game data dumps
/// These are 'triggers' that return a value.
const SCOPE_VALUE: &[(u32, &str)] = &[
    (Faction, "average_faction_opinion"),
    (Faction, "average_faction_opinion_not_powerful_vassal"),
    (Faction, "average_faction_opinion_powerful_vassal"),
    (Faction, "discontent_per_month"),
    (Faction, "faction_discontent"),
    (Faction, "faction_power"),
    (Faction, "faction_power_threshold"),
    (Faction, "months_until_max_discontent"),
    (Faction, "number_of_faction_members_in_council"),
    (War, "attacker_war_score"),
    (War, "days_since_max_war_score"),
    (War, "defender_war_score"),
    (War, "war_days"),
    (Combat, "num_total_troops"),
    (Combat, "warscore_value"),
    (MercenaryCompany, "mercenary_company_expiration_days"),
    (Activity, "number_of_participants"),
    (CombatSide, "num_enemies_killed"),
    (CombatSide, "percent_enemies_killed"),
    (CombatSide, "side_soldiers"),
    (CombatSide, "side_strength"),
    (CombatSide, "troops_ratio"),
    (Army, "army_max_size"),
    (Army, "army_size"),
    (Army, "raid_loot"),
    (Army, "total_army_damage"),
    (Army, "total_army_pursuit"),
    (Army, "total_army_screen"),
    (Army, "total_army_siege_value"),
    (Army, "total_army_toughness"),
    (Artifact, "artifact_durability"),
    (Artifact, "artifact_max_durability"),
    (Artifact, "num_artifact_kills"),
    (LandedTitle, "active_de_jure_drift_progress"),
    (LandedTitle, "county_control"),
    (LandedTitle, "county_control_rate"),
    (LandedTitle, "county_control_rate_modifier"),
    (LandedTitle, "county_holder_opinion"),
    (LandedTitle, "county_opinion"),
    (LandedTitle, "county_opinion_target"),
    (LandedTitle, "development_level"),
    (LandedTitle, "development_rate"),
    (LandedTitle, "development_rate_modifier"),
    (LandedTitle, "development_towards_level_increase"),
    (LandedTitle, "tier"),
    (LandedTitle, "title_held_years"), // TODO: warn if this is compared with =
    (Culture, "culture_age"),
    (Culture, "culture_number_of_counties"),
    (GreatHolyWar, "days_until_ghw_launch"),
    (GreatHolyWar, "ghw_attackers_strength"),
    (GreatHolyWar, "ghw_defenders_strength"),
    (GreatHolyWar, "war_chest_gold"),
    (GreatHolyWar, "war_chest_piety"),
    (GreatHolyWar, "war_chest_prestige"),
    (Faith, "estimated_faith_strength"),
    (Faith, "fervor"),
    (Faith, "holy_sites_controlled"),
    (Faith, "num_character_followers"),
    (Faith, "num_county_followers"),
    (Province, "available_loot"),
    (Province, "building_slots"),
    (Province, "combined_building_level"),
    (Province, "fort_level"),
    (Province, "free_building_slots"),
    (Province, "monthly_income"),
    (Province, "num_buildings"),
    (Province, "number_of_characters_in_pool"),
    (LandedTitle | Province, "building_levies"),
    (LandedTitle | Province, "building_max_garrison"),
    (Scheme, "scheme_duration_days"),
    (Scheme, "scheme_monthly_progress"),
    (Scheme, "scheme_number_of_agents"),
    (Scheme, "scheme_number_of_exposed_agents"),
    (Scheme, "scheme_power"),
    (Scheme, "scheme_power_resistance_difference"),
    (Scheme, "scheme_power_resistance_ratio"),
    (Scheme, "scheme_progress"),
    (Scheme, "scheme_resistance"),
    (Scheme, "scheme_secrecy"),
    (Scheme, "scheme_success_chance"),
    (Inspiration, "base_inspiration_gold_cost"),
    (Inspiration, "days_since_creation"),
    (Inspiration, "days_since_sponsorship"),
    (Inspiration, "inspiration_gold_invested"),
    (Inspiration, "inspiration_progress"),
    (Dynasty, "dynasty_num_unlocked_perks"),
    (Dynasty, "dynasty_prestige"),
    (Dynasty, "dynasty_prestige_level"),
    (HolyOrder, "num_leased_titles"),
    (None, "current_computer_date_day"),
    (None, "current_computer_date_month"),
    (None, "current_computer_date_year"),
    (None, "current_day"),
    (None, "current_month"),
    (None, "current_tooltip_depth"),
    (None, "current_year"),
    (None, "years_from_game_start"),
    (Character, "age"),
    (Character, "ai_boldness"),
    (Character, "ai_compassion"),
    (Character, "ai_energy"),
    (Character, "ai_greed"),
    (Character, "ai_honor"),
    (Character, "ai_rationality"),
    (Character, "ai_reserved_gold"),
    (Character, "ai_sociability"),
    (Character, "ai_vengefulness"),
    (Character, "ai_war_chest"),
    (Character, "ai_zeal"),
    (Character, "attraction"),
    (Character, "average_amenity_level"),
    (Character, "base_weight"),
    (Character, "council_task_monthly_progress"),
    (Character, "court_grandeur_base"),
    (Character, "court_grandeur_current"),
    (Character, "court_grandeur_current_level"),
    (Character, "court_grandeur_minimum_expected"),
    (Character, "court_grandeur_minimum_expected_level"),
    (Character, "court_positions_currently_avaiable"),
    (Character, "court_positions_currently_filled"),
    (Character, "current_weight"),
    (Character, "current_weight_for_portrait"),
    (Character, "days_as_ruler"),
    (Character, "days_in_prison"),
    (Character, "days_of_continuous_peace"),
    (Character, "days_of_continuous_war"),
    (Character, "days_since_death"),
    (Character, "days_since_joined_court"),
    (Character, "debt_level"),
    (Character, "diplomacy"),
    (Character, "diplomacy_for_portrait"),
    (Character, "domain_limit"),
    (Character, "domain_limit_available"),
    (Character, "domain_limit_percentage"),
    (Character, "domain_size"),
    (Character, "domain_size_excluding_grace_period"),
    (Character, "dread"),
    (Character, "effective_age"),
    (Character, "fertility"),
    (Character, "focus_progress"),
    (Character, "gold"),
    (Character, "has_had_focus_for_days"),
    (Character, "health"),
    (Character, "highest_held_title_tier"),
    (Character, "intrigue"),
    (Character, "intrigue_for_portrait"),
    (Character, "learning"),
    (Character, "learning_for_portrait"),
    (Character, "long_term_gold"),
    (Character, "martial"),
    (Character, "martial_for_portrait"),
    (Character, "max_military_strength"),
    (Character, "max_number_of_concubines"),
    (Character, "max_number_of_knights"),
    (Character, "missing_unique_ancestors"),
    (Character, "monthly_character_balance"),
    (Character, "monthly_character_expenses"),
    (Character, "monthly_character_income"),
    (Character, "monthly_character_income_long_term"),
    (Character, "monthly_character_income_reserved"),
    (Character, "monthly_character_income_short_term"),
    (Character, "monthly_character_income_war_chest"),
    (Character, "months_as_ruler"),
    (Character, "num_of_bad_genetic_traits"),
    (Character, "num_of_good_genetic_traits"),
    (Character, "num_of_known_languages"),
    (Character, "num_sinful_traits"),
    (Character, "num_virtuous_traits"),
    (Character, "number_of_commander_traits"),
    (Character, "number_of_concubines"),
    (Character, "number_of_desired_concubines"),
    (Character, "number_of_fertile_concubines"),
    (Character, "number_of_knights"),
    (Character, "number_of_lifestyle_traits"),
    (Character, "number_of_maa_regiments"),
    (Character, "number_of_personality_traits"),
    (Character, "number_of_powerful_vassals"),
    (Character, "number_of_traits"),
    (Character, "perk_points"),
    (Character, "perk_points_assigned"),
    (Character, "piety"),
    (Character, "piety_level"),
    (Character, "pregnancy_days"),
    (Character, "prestige"),
    (Character, "prestige_level"),
    (Character, "prowess"),
    (Character, "prowess_for_portrait"),
    (Character, "ransom_cost"),
    (Character, "realm_size"),
    (Character, "short_term_gold"),
    (Character, "stewardship"),
    (Character, "stewardship_for_portrait"),
    (Character, "stress"),
    (Character, "stress_level"),
    (Character, "sub_realm_size"),
    (Character, "target_weight"),
    (Character, "tyranny"),
    (Character, "vassal_count"),
    (Character, "vassal_limit"),
    (Character, "vassal_limit_available"),
    (Character, "vassal_limit_percentage"),
    (Character, "yearly_character_balance"),
    (Character, "yearly_character_expenses"),
    (Character, "yearly_character_income"),
    (Character, "years_as_ruler"),
];
// Special:
// num_of_relation_<relation>
// perks_in_<lifestyle>
// <lifestyle>_perk_points
// <lifestyle>_perks
// <lifestyle>_unlockable_perks
// <lifestyle>_xp
//
// TODO Special:
// <legacy>_track_perks

/// LAST UPDATED VERSION 1.6.2.2
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(u32, &str, u32)] = &[
    (DynastyHouse, "house_claimed_artifact", Artifact),
    (DynastyHouse, "house_member", Character),
    (Faction, "faction_county_member", LandedTitle),
    (Faction, "faction_member", Character),
    (War, "war_attacker", Character),
    (War, "war_defender", Character),
    (War, "war_participant", Character),
    (Activity, "activity_declined", Character),
    (Activity, "activity_invited", Character),
    (Activity, "participant", Character),
    (CombatSide, "side_commander", Character),
    (CombatSide, "side_knight", Character),
    (CasusBelli, "target_title", LandedTitle),
    (Artifact, "artifact_claimant", Character),
    (Artifact, "artifact_house_claimant", DynastyHouse),
    (LandedTitle, "claimant", Character),
    (LandedTitle, "connected_county", LandedTitle),
    (LandedTitle, "controlled_faith", Faith),
    (LandedTitle, "county_province", Province),
    (LandedTitle, "county_struggle", Struggle),
    (LandedTitle, "de_jure_county", LandedTitle),
    (LandedTitle, "de_jure_county_holder", Character),
    (LandedTitle, "de_jure_top_liege", Character),
    (LandedTitle, "dejure_vassal_title_holder", Character),
    (LandedTitle, "direct_de_facto_vassal_title", LandedTitle),
    (LandedTitle, "direct_de_jure_vassal_title", LandedTitle),
    (LandedTitle, "election_candidate", Character),
    (LandedTitle, "elector", Character),
    (LandedTitle, "in_de_facto_hierarchy", LandedTitle), // TODO has continue section
    (LandedTitle, "in_de_jure_hierarchy", LandedTitle),  // TODO has continue section
    (LandedTitle, "neighboring_county", LandedTitle),
    (LandedTitle, "past_holder", Character),
    (LandedTitle, "past_holder_reversed", Character),
    (LandedTitle, "this_title_or_de_jure_above", LandedTitle),
    (LandedTitle, "title_heir", Character),
    (LandedTitle, "title_joined_faction", Faction),
    (
        LandedTitle,
        "title_to_title_neighboring_and_across_water_county",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighboring_and_across_water_duchy",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighboring_and_across_water_empire",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighboring_and_across_water_kingdom",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighboring_county",
        LandedTitle,
    ),
    (LandedTitle, "title_to_title_neighboring_duchy", LandedTitle),
    (
        LandedTitle,
        "title_to_title_neighboring_empire",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighboring_kingdom",
        LandedTitle,
    ),
    (Culture, "culture_county", LandedTitle),
    (Culture, "culture_duchy", LandedTitle),
    (Culture, "culture_empire", LandedTitle),
    (Culture, "culture_kingdom", LandedTitle),
    (Culture, "parent_culture", Culture),
    (Culture, "parent_culture_or_above", Culture),
    (GreatHolyWar, "pledged_attacker", Character),
    (GreatHolyWar, "pledged_defender", Character),
    (Faith, "defensive_great_holy_wars", GreatHolyWar),
    (Faith, "faith_character", Character),
    (Faith, "faith_holy_order", HolyOrder),
    (Faith, "faith_playable_ruler", Character),
    (Faith, "faith_ruler", Character),
    (Faith, "holy_site", LandedTitle),
    (Character | Artifact, "killed_character", Character),
    (Struggle, "interloper_ruler", Character),
    (Struggle, "involved_ruler", Character),
    (Scheme, "scheme_agent", Character),
    (Secret, "secret_knower", Character),
    (Secret, "secret_participant", Character),
    (Dynasty, "dynasty_member", Character),
    (HolyOrder, "leased_title", LandedTitle),
    (None, "artifact", Artifact),
    (None, "barony", LandedTitle),
    (None, "character_with_royal_court", Character),
    (None, "county", LandedTitle),
    (None, "county_in_region", LandedTitle), // TODO region = region_name inside it
    (None, "culture_global", Culture),
    (None, "duchy", LandedTitle),
    (None, "empire", LandedTitle),
    (None, "in_global_list", ALL), // TODO list = name or variable = name
    (None, "in_list", ALL),        // TODO list = name or variable = name
    (None, "in_local_list", ALL),  // TODO list = name or variable = name
    (None, "independent_ruler", Character),
    (None, "inspiration", Inspiration),
    (None, "inspired_character", Character),
    (None, "kingdom", LandedTitle),
    (None, "living_character", Character),
    (None, "player", Character),
    (None, "pool_character", Character), // TODO figure out how province is supplied
    (None, "province", Province),
    (None, "religion_global", Religion),
    (None, "ruler", Character),
    (Character, "alert_creatable_title", LandedTitle),
    (Character, "alert_usurpable_title", LandedTitle),
    (Character, "ally", Character),
    (Character, "ancestor", Character),
    (Character, "army", Army),
    (Character, "character_artifact", Artifact),
    (Character, "character_struggle", Struggle),
    (
        Character,
        "character_to_title_neighboring_and_across_water_county",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_and_across_water_duchy",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_and_across_water_empire",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_and_across_water_kingdom",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_county",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_duchy",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_empire",
        LandedTitle,
    ),
    (
        Character,
        "character_to_title_neighboring_kingdom",
        LandedTitle,
    ),
    (Character, "character_war", War),
    (Character, "child", Character),
    (Character, "claim", LandedTitle),
    (Character, "claimed_artifact", Artifact),
    (Character, "close_family_member", Character),
    (Character, "close_or_extended_family_member", Character),
    (Character, "concubine", Character),
    (Character, "consort", Character),
    (Character, "councillor", Character),
    (Character, "court_position_employer", Character),
    (Character, "court_position_holder", Character), // TODO find out how court position is supplied
    (Character, "courtier", Character),
    (Character, "courtier_away", Character),
    (Character, "courtier_or_guest", Character),
    (Character, "de_jure_claim", LandedTitle),
    (Character, "diplomacy_councillor", Character),
    (Character, "directly_owned_province", Province),
    (Character, "election_title", LandedTitle),
    (Character, "equipped_character_artifact", Artifact),
    (Character, "extended_family_member", Character),
    (Character, "foreign_court_guest", Character),
    (Character, "former_concubine", Character),
    (Character, "former_concubinist", Character),
    (Character, "former_spouse", Character),
    (Character, "general_councillor", Character),
    (Character, "heir", Character),
    // TODO one of these might be reversed
    (Character, "heir_title", LandedTitle),
    (Character, "heir_to_title", LandedTitle),
    (Character, "held_title", LandedTitle),
    (Character, "hired_mercenary", MercenaryCompany),
    (Character, "hooked_character", Character),
    (Character, "hostile_raider", Character),
    (Character, "intrigue_councillor", Character),
    (Character, "knight", Character),
    (Character, "known_secret", Secret),
    (Character, "learning_councillor", Character),
    (Character, "liege_or_above", Character),
    (Character, "martial_councillor", Character),
    (
        Character,
        "neighboring_and_across_water_realm_same_rank_owner",
        Character,
    ),
    (
        Character,
        "neighboring_and_across_water_top_liege_realm",
        LandedTitle,
    ),
    (
        Character,
        "neighboring_and_across_water_top_liege_realm_owner",
        Character,
    ),
    (Character, "neighboring_realm_same_rank_owner", Character),
    (Character, "neighboring_top_liege_realm", LandedTitle),
    (Character, "neighboring_top_liege_realm_owner", Character),
    (Character, "opposite_sex_spouse_candidate", Character),
    (Character, "owned_story", StoryCycle),
    (Character, "parent", Character),
    (Character, "patroned_holy_order", HolyOrder),
    (Character, "personal_claimed_artifact", Artifact),
    (Character, "pinned_character", Character),
    (Character, "pinning_character", Character),
    (Character, "played_character", Character),
    (Character, "player_heir", Character),
    (Character, "pool_guest", Character),
    (Character, "potential_marriage_option", Character),
    (Character, "pretender_title", LandedTitle),
    (Character, "primary_war_enemy", Character),
    (Character, "prisoner", Character),
    (Character, "prowess_councillor", Character),
    (Character, "raid_target", Character),
    (Character, "realm_county", LandedTitle),
    (Character, "realm_de_jure_duchy", LandedTitle),
    (Character, "realm_de_jure_empire", LandedTitle),
    (Character, "realm_de_jure_kingdom", LandedTitle),
    (Character, "realm_province", Province),
    (Character, "relation", Character), // TODO takes a type
    (Character, "same_sex_spouse_candidate", Character),
    (Character, "scheme", Scheme),
    (Character, "secret", Secret),
    (Character, "sibling", Character),
    (Character, "sponsored_inspiration", Inspiration),
    (Character, "spouse", Character),
    (Character, "spouse_candidate", Character),
    (Character, "stewardship_councillor", Character),
    (Character, "sub_realm_barony", LandedTitle),
    (Character, "sub_realm_county", LandedTitle),
    (Character, "sub_realm_duchy", LandedTitle),
    (Character, "sub_realm_empire", LandedTitle),
    (Character, "sub_realm_kingdom", LandedTitle),
    (Character, "sub_realm_title", LandedTitle),
    (Character, "targeting_faction", Faction),
    (Character, "targeting_scheme", Scheme),
    (Character, "targeting_secret", Secret),
    (Character, "traveling_family_member", Character),
    (Character, "truce_holder", Character),
    (Character, "truce_target", Character),
    (Character, "unspent_known_secret", Secret),
    (Character, "vassal", Character),
    (Character, "vassal_or_below", Character),
    (Character, "war_ally", Character),
    (Character, "war_enemy", Character),
    (Religion, "faith", Faith),
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` from the game data dumps
/// These are the triggers that do a simple comparison with a target scope item
const SCOPE_TRIGGER_TARGET: &[(u32, &str, u32)] = &[
    (DynastyHouse, "has_house_artifact_claim", Artifact),
    (War, "is_attacker", Character),
    (War, "is_defender", Character),
    (War, "is_participant", Character),
    (War, "is_war_leader", Character),
    (War, "was_called", Character),
    (Activity, "is_target_participating", Character),
    (Army, "is_army_in_siege_relevant_for", Character),
    (Artifact, "can_be_claimed_by", Character),
    (LandedTitle, "can_title_join_faction", Faction),
    (LandedTitle, "de_jure_drifting_towards", LandedTitle),
    (LandedTitle, "has_character_nominiated", Character), // sic
    (
        LandedTitle,
        "is_de_facto_liege_or_above_target",
        LandedTitle,
    ),
    (LandedTitle, "is_de_jure_liege_or_above_target", LandedTitle),
    (LandedTitle, "is_holy_site_controlled_by", Character),
    (LandedTitle, "is_holy_site_of", Faith),
    (LandedTitle, "is_neighbor_to_realm", Character),
    (
        LandedTitle,
        "target_is_de_facto_liege_or_above",
        LandedTitle,
    ),
    (LandedTitle, "target_is_de_jure_liege_or_above", LandedTitle),
    (
        LandedTitle,
        "title_will_leave_sub_realm_on_succession",
        Character,
    ),
    (Culture, "has_same_culture_ethos", Culture),
    (Culture, "has_same_culture_heritage", Culture),
    (Culture, "has_same_culture_language", Culture),
    (Culture, "has_same_culture_martial_tradition", Culture),
    (GreatHolyWar, "has_forced_defender", Character),
    (GreatHolyWar, "has_pledged_attacker", Character),
    (GreatHolyWar, "has_pledged_defender", Character),
    (Faith, "has_allowed_gender_for_clergy", Character),
    (Faith, "has_dominant_ruling_gender", Character),
    (Faith, "has_preferred_gender_for_clergy", Character),
    (Struggle, "is_culture_involved_in_struggle", Culture),
    (Struggle, "is_faith_involved_in_struggle", Faith),
    (Struggle, "was_player_joined", Character),
    (Scheme, "is_scheme_agent_exposed", Character),
    (Scheme, "scheme_is_character_agent", Character),
    (Secret, "can_be_exposed_by", Character),
    (Secret, "is_criminal_for", Character),
    (Secret, "is_known_by", Character),
    (Secret, "is_shunned_for", Character),
    (Secret, "is_shunned_or_criminal_for", Character),
    (Secret, "is_spent_by", Character),
    (Secret, "same_secret_type_as", Secret),
    (Character, "can_attack_in_hierarchy", Character),
    (Character, "can_be_child_of", Character),
    (Character, "can_be_parent_of", Character),
    (Character, "can_benefit_from_artifact", Artifact),
    (Character, "can_equip_artifact", Artifact),
    (Character, "can_hybridize", Culture),
    (Character, "can_hybridize_excluding_cost", Culture),
    (Character, "can_join_faction", Faction),
    (Character, "can_join_or_create_faction_against", Character),
    (Character, "can_sponsor_inspiration", Inspiration),
    (
        Character,
        "character_has_commander_trait_scope_does_not",
        Character,
    ),
    (Character, "character_is_land_realm_neighbor", Character),
    (Character, "character_is_realm_neighbor", Character),
    (Character, "completely_controls", LandedTitle),
    (Character, "has_any_cb_on", Character),
    (Character, "has_any_scripted_relation", Character),
    (Character, "has_any_secret_relation", Character),
    (Character, "has_artifact_claim", Artifact),
    (Character, "has_banish_reason", Character),
    (Character, "has_claim_on", LandedTitle),
    (Character, "has_court_language_of_culture", Culture),
    (Character, "has_culture", Culture),
    (Character, "has_de_jure_claim_on", Character),
    (Character, "has_disable_non_aggression_pacts", Character), // sic
    (Character, "has_divorce_reason", Character),
    (Character, "has_execute_reason", Character),
    (Character, "has_faith", Faith),
    (Character, "has_hook", Character),
    (Character, "has_hook_from_secret", Character),
    (Character, "has_imprisonment_reason", Character),
    (Character, "has_non_aggression_pact", Character),
    (Character, "has_non_interference", Character),
    (Character, "has_personal_artifact_claim", Artifact),
    (Character, "has_primary_title", LandedTitle),
    (Character, "has_raid_immunity_against", Character),
    (Character, "has_religion", Religion),
    (Character, "has_revoke_title_reason", Character),
    (Character, "has_same_court_language", Character),
    (Character, "has_same_court_type_as", Character),
    (Character, "has_same_culture_as", Character),
    (Character, "has_same_focus_as", Character),
    (Character, "has_same_government", Character),
    (Character, "has_strong_claim_on", LandedTitle),
    (Character, "has_strong_hook", Character),
    (Character, "has_strong_usable_hook", Character),
    (Character, "has_title", LandedTitle),
    (Character, "has_truce", Character),
    (Character, "has_usable_hook", Character),
    (Character, "has_weak_claim_on", LandedTitle),
    (Character, "has_weak_hook", Character),
    (Character, "in_activity_with", Character),
    (Character, "in_diplomatic_range", Character),
    (Character, "is_agent_exposed_in_scheme", Scheme),
    (Character, "is_allied_in_war", Character),
    (Character, "is_allied_to", Character),
    (Character, "is_at_location", Province),
    (Character, "is_at_same_location", Character),
    (Character, "is_at_war_with", Character),
    (Character, "is_attacker_in_war", War),
    (Character, "is_attracted_to_gender_of", Character),
    (Character, "is_causing_raid_hostility_towards", Character),
    (Character, "is_child_of", Character),
    (Character, "is_close_family_of", Character),
    (Character, "is_close_or_extended_family_of", Character),
    (Character, "is_concubine_of", Character),
    (Character, "is_consort_of", Character),
    (Character, "is_councillor_of", Character),
    (Character, "is_courtier_of", Character),
    (Character, "is_cousin_of", Character),
    (Character, "is_defender_in_war", War),
    (Character, "is_employer_of", Character),
    (Character, "is_extended_family_of", Character),
    (Character, "is_forbidden_from_scheme", Scheme),
    (Character, "is_forced_into_scheme", Scheme),
    (Character, "is_foreign_court_guest_of", Character),
    (Character, "is_foreign_court_or_pool_guest_of", Character),
    (Character, "is_grandchild_of", Character),
    (Character, "is_grandparent_of", Character),
    (Character, "is_great_grandchild_of", Character),
    (Character, "is_great_grandparent_of", Character),
    (Character, "is_heir_of", Character),
    (Character, "is_imprisoned_by", Character),
    (Character, "is_in_pool_at", Province),
    (Character, "is_in_target_activity", Activity),
    (Character, "is_in_the_same_court_as", Character),
    (Character, "is_in_the_same_court_as_or_guest", Character),
    (Character, "is_knight_of", Character),
    (Character, "is_leader_in_war", War),
    (Character, "is_liege_or_above_of", Character),
    (Character, "is_nibling_of", Character),
    (Character, "is_obedient", Character),
    (Character, "is_parent_of", Character),
    (Character, "is_participant_in_war", War),
    (Character, "is_player_heir_of", Character),
    (Character, "is_pool_guest_of", Character),
    (Character, "is_powerful_vassal_of", Character),
    (Character, "is_primary_heir_of", Character),
    (Character, "is_sibling_of", Character),
    (Character, "is_spouse_of", Character),
    (Character, "is_spouse_of_even_if_dead", Character),
    (Character, "is_twin_of", Character),
    (Character, "is_uncle_or_aunt_of", Character),
    (Character, "is_valid_as_agent_in_scheme", Scheme),
    (Character, "is_vassal_of", Character),
    (Character, "is_vassal_or_below_of", Character),
    (Character, "knows_court_language_of", Character),
    (Character, "knows_language_of_culture", Culture),
    (Character, "sex_opposite_of", Character),
    (Character, "sex_same_as", Character),
    (Character, "target_is_liege_or_above", Character),
    (Character, "target_is_same_character_or_above", Character),
    (Character, "target_is_vassal_or_below", Character),
];
// Special: has_relation_<relation> Character to Character
// Special: has_secret_relation_<relation> Character to Character

/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` from the game data dumps
/// These are the triggers that take a simple yes or no
const SCOPE_TRIGGER_BOOL: &[(u32, &str)] = &[
    (Faction, "faction_can_press_demands"),
    (Faction, "faction_is_at_war"),
    (Faction, "has_special_character"),
    (Faction, "has_special_title"),
    (War, "has_valid_casus_belli"),
    (War, "is_civil_war"),
    (War, "is_white_peace_possible"),
    (Activity, "activity_has_been_activated"),
    (CombatSide, "is_combat_side_attacker"),
    (CombatSide, "is_combat_side_pursuing"),
    (CombatSide, "is_combat_side_retreating"),
    (Army, "army_is_moving"),
    (Army, "can_disband_army"),
    (Army, "is_army_in_combat"),
    (Army, "is_army_in_raid"),
    (Army, "is_army_in_siege"),
    (Army, "is_raid_army"),
    (Artifact, "is_equipped"),
    (Artifact, "is_unique"),
    (Artifact, "should_decay"),
    (LandedTitle, "can_be_leased_out"),
    (LandedTitle, "has_disabled_building"),
    (LandedTitle, "has_revokable_lease"),
    (LandedTitle, "has_user_set_coa"),
    (LandedTitle, "has_wrong_holding_type"),
    (LandedTitle, "is_capital_barony"),
    (LandedTitle, "is_coastal_county"),
    (LandedTitle, "is_contested"),
    (LandedTitle, "is_head_of_faith"),
    (LandedTitle, "is_holy_order"),
    (LandedTitle, "is_holy_site"),
    (LandedTitle, "is_landless_type_title"),
    (LandedTitle, "is_leased_out"),
    (LandedTitle, "is_mercenary_company"),
    (LandedTitle, "is_riverside_county"),
    (LandedTitle, "is_title_created"),
    (LandedTitle, "is_titular"),
    (LandedTitle, "is_under_holy_order_lease"),
    (LandedTitle, "title_is_a_faction_member"),
    (Culture, "is_divergent_culture"),
    (Culture, "is_hybrid_culture"),
    (GreatHolyWar, "is_directed_ghw"),
    (Province, "has_free_building_slot"),
    (Province, "has_holding"),
    (Province, "has_ongoing_construction"),
    (Province, "has_special_building"),
    (Province, "has_special_building_slot"),
    (Province, "is_coastal"),
    (Province, "is_county_capital"),
    (Province, "is_raided"),
    (Province, "is_riverside_province"),
    (Province, "is_sea_province"),
    (Scheme, "is_hostile"),
    (Scheme, "is_scheme_exposed"),
    (CouncilTask, "can_fire_position"),
    (Secret, "local_player_knows_this_secret"),
    (Dynasty, "dynasty_can_unlock_relevant_perk"),
    (None, "always"),
    (None, "debug_only"),
    (None, "has_local_player_open_court_event"),
    (None, "has_local_player_seen_unopened_court_event"),
    (None, "has_local_player_unopened_court_event"),
    (None, "has_multiple_players"),
    (None, "is_gamestate_tutorial_active"),
    (None, "is_player_selected"),
    (None, "is_tutorial_active"),
    (None, "release_only"),
    (None, "scripted_tests"),
    (None, "should_show_disturbing_portrait_modifiers"),
    (None, "should_show_nudity"),
    (Character, "allowed_concubines"),
    (Character, "allowed_more_concubines"),
    (Character, "allowed_more_spouses"),
    (Character, "can_diverge"),
    (Character, "can_diverge_excluding_cost"),
    (Character, "can_have_children"),
    (Character, "can_join_activities"),
    (
        Character,
        "does_ai_liege_in_vassal_contract_desire_obligation_change",
    ),
    (
        Character,
        "does_ai_vassal_in_vassal_contract_desire_obligation_change",
    ),
    (Character, "has_any_artifact"),
    (Character, "has_any_court_position"),
    (Character, "has_any_focus"),
    (Character, "has_any_nickname"),
    (Character, "has_any_secrets"),
    (Character, "has_any_unequipped_artifact"),
    (Character, "has_bad_nickname"),
    (Character, "has_completed_inspiration"),
    (Character, "has_dynasty"),
    (Character, "has_employed_any_court_position"),
    (Character, "has_father"),
    (Character, "has_free_council_slot"),
    (Character, "has_mother"),
    (Character, "has_outstanding_artifact_claims"),
    (Character, "has_owned_scheme"),
    (Character, "has_pending_court_events"),
    (Character, "has_prisoners"),
    (Character, "has_raised_armies"),
    (Character, "has_royal_court"),
    (Character, "has_spawned_court_events"),
    (Character, "has_targeting_faction"),
    (Character, "holds_landed_title"),
    (Character, "is_a_faction_leader"),
    (Character, "is_a_faction_member"),
    (Character, "is_adult"),
    (Character, "is_ai"),
    (Character, "is_alive"),
    (Character, "is_at_home"),
    (Character, "is_at_war"),
    (Character, "is_at_war_as_attacker"),
    (Character, "is_at_war_as_defender"),
    (Character, "is_at_war_with_liege"),
    (Character, "is_attracted_to_men"),
    (Character, "is_attracted_to_women"),
    (Character, "is_away_from_court"),
    (Character, "is_betrothed"),
    (Character, "is_character_window_main_character"),
    (Character, "is_claimant"),
    (Character, "is_clergy"),
    (Character, "is_commanding_army"),
    (Character, "is_concubine"),
    (Character, "is_councillor"),
    (Character, "is_courtier"),
    (Character, "is_female"),
    (Character, "is_forced_into_faction"),
    (Character, "is_foreign_court_guest"),
    (Character, "is_foreign_court_or_pool_guest"),
    (Character, "is_from_ruler_designer"),
    (Character, "is_immortal"),
    (Character, "is_imprisoned"),
    (Character, "is_in_an_activity"),
    (Character, "is_in_army"),
    (Character, "is_in_civil_war"),
    (Character, "is_in_ongoing_great_holy_war"),
    (Character, "is_incapable"),
    (Character, "is_independent_ruler"),
    (Character, "is_knight"),
    (Character, "is_landed"),
    (Character, "is_landless_ruler"),
    (Character, "is_local_player"),
    (Character, "is_lowborn"),
    (Character, "is_male"),
    (Character, "is_married"),
    (Character, "is_normal_councillor"),
    (Character, "is_overriding_designated_winner"),
    (Character, "is_pledged_ghw_attacker"),
    (Character, "is_pool_character"),
    (Character, "is_pool_guest"),
    (Character, "is_powerful_vassal"),
    (Character, "is_pregnant"),
    (Character, "is_ruler"),
    (Character, "is_special_councillor"),
    (Character, "is_theocratic_lessee"),
    (Character, "is_unborn_child_of_concubine"),
    (Character, "is_unborn_known_bastard"),
    (Character, "is_visibly_fertile"),
    (Character, "matrilinear_betrothal"),
    (Character, "matrilinear_marriage"),
    (Character, "owns_a_story"),
    (Character, "owns_an_activity"),
    (Character, "patrilinear_betrothal"),
    (Character, "patrilinear_marriage"),
    (Character, "vassal_contract_has_modifiable_obligations"),
    (Character, "vassal_contract_is_blocked_from_modification"),
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `triggers.log` from the game data dumps
/// These are the triggers that compare to an item type
const SCOPE_TRIGGER_ITEM: &[(u32, &str, Item)] = &[
    (DynastyHouse, "has_house_modifier", Item::Modifier),
    (
        DynastyHouse,
        "has_house_modifier_duration_remaining",
        Item::Modifier,
    ),
    (Faction, "faction_is_type", Item::Faction),
    (War, "using_cb", Item::CasusBelli),
    (CombatSide, "has_maa_of_type", Item::MenAtArms),
    (Artifact, "artifact_slot_type", Item::ArtifactSlot),
    (Artifact, "artifact_type", Item::Artifact),
    (Artifact, "category", Item::ArtifactCategory),
    (Artifact, "has_artifact_feature", Item::ArtifactFeature),
    (
        Artifact,
        "has_artifact_feature_group",
        Item::ArtifactFeatureGroup,
    ),
    (Artifact, "has_artifact_modifier", Item::ArtifactModifier),
    (Artifact, "rarity", Item::ArtifactRarity),
    (LandedTitle, "has_county_modifier", Item::Modifier),
    (
        LandedTitle,
        "has_county_modifier_duration_remaining",
        Item::Modifier,
    ),
    (LandedTitle, "has_holy_site_flag", Item::HolySiteFlag),
    (LandedTitle, "has_title_law", Item::TitleLaw),
    (LandedTitle, "has_title_law_flag", Item::TitleLawFlag),
    (LandedTitle, "is_target_of_council_task", Item::CouncilTask),
    (
        Culture,
        "culture_overlaps_geographical_region",
        Item::Region,
    ),
    (Culture, "has_building_gfx", Item::BuildingGfx),
    (Culture, "has_clothing_gfx", Item::ClothingGfx),
    (Culture, "has_coa_gfx", Item::CoaGfx),
    (Culture, "has_cultural_era_or_later", Item::CultureEra),
    (Culture, "has_cultural_parameter", Item::CultureParameter),
    (Culture, "has_cultural_pillar", Item::CulturePillar),
    (Culture, "has_cultural_tradition", Item::CultureTradition),
    (Culture, "has_innovation", Item::Innovation),
    (Culture, "has_innovation_flag", Item::InnovationFlag),
    (Culture, "has_name_list", Item::NameList),
    (Culture, "has_primary_name_list", Item::NameList),
    (Culture, "has_unit_gfx", Item::UnitGfx),
    (StoryCycle, "story_type", Item::Story),
    (Faith, "controls_holy_site", Item::HolySite),
    (Faith, "controls_holy_site_with_flag", Item::HolySiteFlag),
    (Faith, "has_doctrine", Item::Doctrine),
    (Faith, "has_doctrine_parameter", Item::DoctrineParameter),
    (Faith, "has_graphical_faith", Item::GraphicalFaith),
    (Faith, "has_icon", Item::FaithIcon),
    (Faith, "religion_tag", Item::Religion),
    (Faith, "trait_is_sin", Item::Trait),
    (Faith, "trait_is_virtue", Item::Trait),
    (Province, "geographical_region", Item::Region),
    (Province, "has_building", Item::Building),
    (Province, "has_building_or_higher", Item::Building),
    (Province, "has_construction_with_flag", Item::BuildingFlag),
    (Province, "has_holding_type", Item::Holding),
    (Province, "has_province_modifier", Item::Modifier),
    (
        Province,
        "has_province_modifier_duration_remaining",
        Item::Modifier,
    ),
    (Province, "terrain", Item::Terrain),
    (
        Struggle,
        "has_struggle_phase_parameter",
        Item::StrugglePhaseParameter,
    ),
    (Struggle, "is_struggle_phase", Item::StrugglePhase),
    (Struggle, "is_struggle_type", Item::Struggle),
    (Struggle, "phase_has_catalyst", Item::Catalyst),
    (Scheme, "has_scheme_modifier", Item::Modifier),
    (Scheme, "scheme_skill", Item::Skill),
    (Scheme, "scheme_type", Item::Scheme),
    (Inspiration, "has_inspiration_type", Item::Inspiration),
    (Secret, "secret_type", Item::Secret),
    (Dynasty, "has_dynasty_modifier", Item::Modifier),
    (
        Dynasty,
        "has_dynasty_modifier_duration_remaining",
        Item::Modifier,
    ),
    (Dynasty, "has_dynasty_perk", Item::DynastyPerk),
    // ---
    (Character, "can_execute_decision", Item::Decision),
    (Character, "is_decision_on_cooldown", Item::Decision),
    (Character, "completely_controls_region", Item::Region),
    // ---
    (Character, "has_character_modifier", Item::Modifier),
    (
        Character,
        "has_character_modifier_duration_remaining",
        Item::Modifier,
    ),
    // ---
    (Character, "has_lifestyle", Item::Lifestyle),
    (Character, "has_trait", Item::Trait),
    (Character, "has_inactive_trait", Item::Trait),
    (Character, "has_opposite_relation", Item::Relation),
    (
        Character,
        "has_pending_interaction_of_type",
        Item::Interaction,
    ),
    (Character, "is_leading_faction_type", Item::Faction),
    (Character, "owns_story_of_type", Item::Story),
];
