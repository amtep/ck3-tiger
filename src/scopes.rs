#![allow(non_upper_case_globals)]

pub type Scopes = u32;

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_scopes.log` from the game data dumps.
pub const None: Scopes = 0x0000_0001;
pub const Value: Scopes = 0x0000_0002;
pub const Bool: Scopes = 0x0000_0004;
pub const Flag: Scopes = 0x0000_0008;
pub const Character: Scopes = 0x0000_0010;
pub const LandedTitle: Scopes = 0x0000_0020;
pub const Activity: Scopes = 0x0000_0040;
pub const Secret: Scopes = 0x0000_0080;
pub const Province: Scopes = 0x0000_0100;
pub const Scheme: Scopes = 0x0000_0200;
pub const Combat: Scopes = 0x0000_0400;
pub const CombatSide: Scopes = 0x0000_0800;
pub const TitleAndVassalChange: Scopes = 0x0000_1000;
pub const Faith: Scopes = 0x0000_2000;
pub const GreatHolyWar: Scopes = 0x0000_4000;
pub const Religion: Scopes = 0x0000_8000;
pub const War: Scopes = 0x0001_0000;
pub const StoryCycle: Scopes = 0x0002_0000;
pub const CasusBelli: Scopes = 0x0004_0000;
pub const Dynasty: Scopes = 0x0008_0000;
pub const DynastyHouse: Scopes = 0x0010_0000;
pub const Faction: Scopes = 0x0020_0000;
pub const Culture: Scopes = 0x0040_0000;
pub const Army: Scopes = 0x0080_0000;
pub const HolyOrder: Scopes = 0x0100_0000;
pub const CouncilTask: Scopes = 0x0200_0000;
pub const MercenaryCompany: Scopes = 0x0400_0000;
pub const Artifact: Scopes = 0x0800_0000;
pub const Inspiration: Scopes = 0x1000_0000;
pub const Struggle: Scopes = 0x2000_0000;
pub const ALL: Scopes = 0x3fff_ffff;

pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if *s == name {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

pub fn scope_value(name: &str) -> Option<Scopes> {
    for (from, s) in SCOPE_VALUE {
        if *s == name {
            return Some(*from);
        }
    }
    std::option::Option::None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_ITERATOR {
        if *s == name {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
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
    (Character, "faith", Faith),
    (LandedTitle | Province | GreatHolyWar, "faith", Faith),
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
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_FROM_PREFIX: &[(Scopes, &str, Scopes)] = &[
    (Character, "vassal_contract_obligation_level", Value),
    (Character, "aptitude", Value),
    (Character, "council_task", CouncilTask),
    (Character, "court_position", Character),
    (Character, "cp", Character), // councillor
    (None, "array_define", Value),
    (None, "character", Character),
    (Value, "compare_value", Value), // ?? needs more investigation
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
const SCOPE_VALUE: &[(Scopes, &str)] = &[
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
    (GreatHolyWar, "days_until_ghx_launch"),
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
// TODO Special:
// <legacy>_track_perks
// perks_in_<lifestyle>
// <lifestyle>_perk_points
// <lifestyle>_perks
// <lifestyle>_unlockable_perks
// <lifestyle>_xp
// num_of_relation_<relation>

/// LAST UPDATED VERSION 1.6.2.2
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
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
        "title_to_title_neighoring_and_across_water_county",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighoring_and_across_water_duchy",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighoring_and_across_water_empire",
        LandedTitle,
    ),
    (
        LandedTitle,
        "title_to_title_neighoring_and_across_water_kingdom",
        LandedTitle,
    ),
    (LandedTitle, "title_to_title_neighoring_county", LandedTitle),
    (LandedTitle, "title_to_title_neighoring_duchy", LandedTitle),
    (LandedTitle, "title_to_title_neighoring_empire", LandedTitle),
    (
        LandedTitle,
        "title_to_title_neighoring_kingdom",
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
