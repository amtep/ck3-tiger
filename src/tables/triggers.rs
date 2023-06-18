use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;

/// A version of Trigger that uses u64 to represent Scopes values, because
/// constructing bitfield types in const values is not allowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RawTrigger {
    Boolean,            // trigger = no or trigger = yes
    CompareValue,       // can be a script value
    CompareValueWarnEq, // can be a script value; warn if =
    SetValue,           // can be a script value; no < or >
    CompareDate,        // value must be a valid date
    Scope(u64),         // trigger is compared to a scope object
    Item(Item),         // value is chosen from an item type
    ScopeOrItem(u64, Item),
    Choice(&'static [&'static str]), // value is chosen from a list given here
    // For Block, if a field name in the array starts with ? it means that field is optional
    Block(&'static [(&'static str, RawTrigger)]), // trigger takes a block with these fields
    ScopeOrBlock(u64, &'static [(&'static str, RawTrigger)]), // trigger takes a block with these fields
    ItemOrBlock(Item, &'static [(&'static str, RawTrigger)]), // trigger takes a block with these fields
    CompareValueOrBlock(&'static [(&'static str, RawTrigger)]), // can be part of a scope chain but also a standalone trigger
    ScopeList(u64),      // trigger takes a block of values of this scope type
    ScopeCompare(u64),   // trigger takes a block comparing two scope objects
    CompareToScope(u64), // this is for inside a Block, where a key is compared to a scope object

    Control, // this key opens another trigger block
    Special, // this has specific code for validation

    UncheckedValue,
}

/// A version of Trigger that has real Scopes values instead of u64 bitfields
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    Boolean,
    CompareValue,
    CompareValueWarnEq,
    SetValue,
    CompareDate,
    Scope(Scopes),
    Item(Item),
    ScopeOrItem(Scopes, Item),
    Choice(&'static [&'static str]),
    Block(Vec<(&'static str, Trigger)>),
    ScopeOrBlock(Scopes, Vec<(&'static str, Trigger)>),
    ItemOrBlock(Item, Vec<(&'static str, Trigger)>),
    CompareValueOrBlock(Vec<(&'static str, Trigger)>),
    ScopeList(Scopes),
    ScopeCompare(Scopes),
    CompareToScope(Scopes),

    Control,
    Special,

    UncheckedValue,
}

impl Trigger {
    fn from_raw(raw: &RawTrigger) -> Self {
        match raw {
            RawTrigger::Boolean => Trigger::Boolean,
            RawTrigger::CompareValue => Trigger::CompareValue,
            RawTrigger::CompareValueWarnEq => Trigger::CompareValueWarnEq,
            RawTrigger::SetValue => Trigger::SetValue,
            RawTrigger::CompareDate => Trigger::CompareDate,
            RawTrigger::Scope(s) => Trigger::Scope(Scopes::from_bits_truncate(*s)),
            RawTrigger::Item(i) => Trigger::Item(*i),
            RawTrigger::ScopeOrItem(s, i) => {
                Trigger::ScopeOrItem(Scopes::from_bits_truncate(*s), *i)
            }
            RawTrigger::Choice(choices) => Trigger::Choice(choices),
            RawTrigger::Block(fields) => Trigger::Block(Trigger::from_raw_fields(fields)),
            RawTrigger::ScopeOrBlock(s, fields) => Trigger::ScopeOrBlock(
                Scopes::from_bits_truncate(*s),
                Trigger::from_raw_fields(fields),
            ),
            RawTrigger::ItemOrBlock(i, fields) => {
                Trigger::ItemOrBlock(*i, Trigger::from_raw_fields(fields))
            }
            RawTrigger::CompareValueOrBlock(fields) => {
                Trigger::CompareValueOrBlock(Trigger::from_raw_fields(fields))
            }
            RawTrigger::ScopeList(s) => Trigger::ScopeList(Scopes::from_bits_truncate(*s)),
            RawTrigger::ScopeCompare(s) => Trigger::ScopeCompare(Scopes::from_bits_truncate(*s)),
            RawTrigger::CompareToScope(s) => {
                Trigger::CompareToScope(Scopes::from_bits_truncate(*s))
            }
            RawTrigger::Control => Trigger::Control,
            RawTrigger::Special => Trigger::Special,
            RawTrigger::UncheckedValue => Trigger::UncheckedValue,
        }
    }

    fn from_raw_fields(
        fields: &'static [(&'static str, RawTrigger)],
    ) -> Vec<(&'static str, Trigger)> {
        fields
            .iter()
            .map(|(field, trigger)| (*field, Trigger::from_raw(trigger)))
            .collect()
    }
}

pub fn scope_trigger(name: &Token, data: &Everything) -> Option<(Scopes, Trigger)> {
    let name_lc = name.as_str().to_lowercase();

    for (from, s, trigger) in TRIGGER {
        if name_lc == *s {
            return Some((
                Scopes::from_bits_truncate(*from),
                Trigger::from_raw(trigger),
            ));
        }
    }
    if let Some(relation) = name_lc.strip_prefix("has_relation_") {
        data.verify_exists_implied(Item::Relation, relation, name);
        return Some((Scopes::Character, Trigger::Scope(Scopes::Character)));
    }
    if let Some(relation) = name_lc.strip_prefix("has_secret_relation_") {
        data.verify_exists_implied(Item::Relation, relation, name);
        return Some((Scopes::Character, Trigger::Scope(Scopes::Character)));
    }
    if let Some(relation) = name_lc.strip_prefix("num_of_relation_") {
        data.verify_exists_implied(Item::Relation, relation, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    if let Some(lifestyle) = name_lc.strip_prefix("perks_in_") {
        data.verify_exists_implied(Item::Lifestyle, lifestyle, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    if let Some(lifestyle) = name_lc.strip_suffix("_perk_points") {
        data.verify_exists_implied(Item::Lifestyle, lifestyle, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    if let Some(lifestyle) = name_lc.strip_suffix("_unlockable_perks") {
        data.verify_exists_implied(Item::Lifestyle, lifestyle, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    if let Some(legacy) = name_lc.strip_suffix("_track_perks") {
        data.verify_exists_implied(Item::DynastyLegacy, legacy, name);
        return Some((Scopes::Dynasty, Trigger::CompareValue));
    }
    if let Some(lifestyle) = name_lc.strip_suffix("_perks") {
        data.verify_exists_implied(Item::Lifestyle, lifestyle, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    if let Some(lifestyle) = name_lc.strip_suffix("_xp") {
        data.verify_exists_implied(Item::Lifestyle, lifestyle, name);
        return Some((Scopes::Character, Trigger::CompareValue));
    }
    std::option::Option::None
}

pub fn trigger_comparevalue(name: &Token, data: &Everything) -> Option<Scopes> {
    match scope_trigger(name, data) {
        Some((s, Trigger::CompareValue)) => Some(s),
        Some((s, Trigger::CompareValueWarnEq)) => Some(s),
        Some((s, Trigger::CompareDate)) => Some(s),
        Some((s, Trigger::SetValue)) => Some(s),
        Some((s, Trigger::CompareValueOrBlock(_))) => Some(s),
        _ => std::option::Option::None,
    }
}

use RawTrigger::*;

/// LAST UPDATED VERSION 1.9.2
/// See `triggers.log` from the game data dumps
/// special:
///    `<legacy>_track_perks`
///    `<lifestyle>_perk_points`
///    `<lifestyle>_perks`
///    `<lifestyle>_unlockable_perks`
///    `<lifestyle>_xp`
///    `has_relation_<relation>`
///    `has_secret_relation_<relation>`
///    `num_of_relation_<relation>`
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(u64, &str, RawTrigger)] = &[
    (Accolade, "accolade_rank", CompareValue),
    (AccoladeType, "accolade_type_tier", Scope(AccoladeType)),
    (LandedTitle, "active_de_jure_drift_progress", CompareValue),
    // TODO: warn if this is in an any_ iterator and not at the end
    (ALL_BUT_NONE, "add_to_temporary_list", UncheckedValue),
    (Character, "age", CompareValue),
    (Character, "ai_boldness", CompareValue),
    (Character, "ai_compassion", CompareValue),
    (
        Character,
        "ai_diplomacy_stance",
        Block(&[
            ("target", Scope(Character)),
            ("stance", Choice(&["neutral", "threat", "enemy", "friend"])),
        ]),
    ),
    (Character, "ai_energy", CompareValue),
    (Character, "ai_greed", CompareValue),
    (Character, "ai_honor", CompareValue),
    (Character, "ai_rationality", CompareValue),
    (Character, "ai_sociability", CompareValue),
    (
        Character,
        "ai_values_divergence",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "ai_vengefulness", CompareValue),
    (Character, "ai_zeal", CompareValue),
    (
        Character,
        "all_court_artifact_slots",
        Choice(&["empty", "full"]),
    ),
    (None, "all_false", Control),
    (
        Character,
        "all_inventory_artifact_slots",
        Choice(&["empty", "full"]),
    ),
    (Character, "allowed_concubines", Boolean),
    (Character, "allowed_more_concubines", Boolean),
    (Character, "allowed_more_spouses", Boolean),
    (None, "always", Boolean),
    (
        Character,
        "amenity_level",
        Block(&[("type", Item(Item::Amenity)), ("value", CompareValue)]),
    ),
    (None, "and", Control),
    (None, "any_false", Control),
    (
        Character,
        "aptitude",
        Block(&[
            ("court_position", Item(Item::CourtPosition)),
            ("value", CompareValue),
        ]),
    ),
    (Army, "army_is_moving", Boolean),
    (Army, "army_max_size", CompareValue),
    (Army, "army_size", CompareValue),
    (Artifact, "artifact_durability", CompareValue),
    (Artifact, "artifact_max_durability", CompareValue),
    (Artifact, "artifact_slot_type", Item(Item::ArtifactSlotType)),
    (Artifact, "artifact_type", Item(Item::ArtifactType)),
    (
        None,
        "assert_if",
        Block(&[("limit", Control), ("?text", UncheckedValue)]),
    ),
    (None, "assert_read", UncheckedValue),
    (War, "attacker_war_score", CompareValue),
    (Character, "attraction", CompareValue),
    (Province, "available_loot", CompareValueWarnEq),
    (Character, "average_amenity_level", CompareValue),
    (Faction, "average_faction_opinion", CompareValue),
    (
        Faction,
        "average_faction_opinion_not_powerful_vassal",
        CompareValue,
    ),
    (
        Faction,
        "average_faction_opinion_powerful_vassal",
        CompareValue,
    ),
    (Inspiration, "base_inspiration_gold_cost", CompareValue),
    (Character, "base_weight", CompareValue),
    (LandedTitle | Province, "building_levies", CompareValue),
    (
        LandedTitle | Province,
        "building_max_garrison",
        CompareValue,
    ),
    (Province, "building_slots", CompareValue),
    (None, "calc_true_if", Control),
    // TODO: the can_add_hook documentation says it can also take days/months/year but there are no examples in vanilla and it's not clear what it would mean.
    (
        Character,
        "can_add_hook",
        Block(&[("target", Scope(Character)), ("type", Item(Item::Hook))]),
    ),
    (
        Character,
        "can_arrive_in_time_to_activity_minimum",
        Scope(Activity),
    ),
    (Character, "can_attack_in_hierarchy", Scope(Character)),
    (Character, "can_be_acclaimed", Boolean),
    (Character, "can_be_child_of", Scope(Character)),
    (Artifact, "can_be_claimed_by", Scope(Character)),
    (Character, "can_be_employed_as", Item(Item::CourtPosition)),
    (Secret, "can_be_exposed_by", Scope(Character)),
    (LandedTitle, "can_be_leased_out", Boolean),
    (Character, "can_be_parent_of", Scope(Character)),
    (Character, "can_benefit_from_artifact", Scope(Artifact)),
    (TravelPlan, "can_cancel", Boolean),
    (
        Character,
        "can_create_faction",
        Block(&[("type", Item(Item::Faction)), ("target", Scope(Character))]),
    ),
    (
        Character,
        "can_declare_war",
        Block(&[
            ("defender", Scope(Character)),
            ("casus_belli", Item(Item::CasusBelli)),
            ("target_titles", ScopeList(LandedTitle)),
            ("claimant", Scope(Character)),
        ]),
    ),
    (Army, "can_disband_army", Boolean),
    (Character, "can_diverge", Boolean),
    (Character, "can_diverge_excluding_cost", Boolean),
    (
        Character,
        "can_employ_court_position_type",
        Item(Item::CourtPosition),
    ),
    (Character, "can_equip_artifact", Scope(Artifact)),
    (
        Character,
        "can_execute_decision",
        ScopeOrItem(Decision, Item::Decision),
    ),
    (CouncilTask, "can_fire_position", Boolean),
    (Culture, "can_get_innovation_from", Scope(Culture)),
    (Character, "can_have_children", Boolean),
    (
        Character,
        "can_host_activity",
        ScopeOrItem(ActivityType, Item::ActivityType),
    ),
    (Character, "can_hybridize", Scope(Culture)),
    (Character, "can_hybridize_excluding_cost", Scope(Culture)),
    (Character, "can_join_activity", Scope(Activity)),
    (Character, "can_join_faction", Scope(Faction)),
    (
        Character,
        "can_join_or_create_faction_against",
        ScopeOrBlock(
            Character,
            &[
                ("who", Scope(Character)),
                ("?faction", Item(Item::Faction)),
                ("?check_in_a_faction", Boolean),
            ],
        ),
    ),
    (Character, "can_sponsor_inspiration", Scope(Inspiration)),
    (
        Character,
        "can_start_scheme",
        Block(&[("type", Item(Item::Scheme)), ("target", Scope(Character))]),
    ),
    (None, "can_start_tutorial_lesson", UncheckedValue), // TODO
    (
        LandedTitle,
        "can_title_create_faction",
        Block(&[("type", Item(Item::Faction)), ("target", Scope(Character))]),
    ),
    (LandedTitle, "can_title_join_faction", Scope(Faction)),
    (Artifact, "category", Choice(&["inventory", "court"])),
    (
        Character,
        "character_has_commander_trait_scope_does_not",
        Scope(Character),
    ),
    (
        Character,
        "character_is_land_realm_neighbor",
        Scope(Character),
    ),
    (Character, "character_is_realm_neighbor", Scope(Character)),
    (Province, "combined_building_level", CompareValue),
    (Character, "completely_controls", Scope(LandedTitle)),
    (Character, "completely_controls_region", Item(Item::Region)),
    (Faith, "controls_holy_site", Item(Item::HolySite)),
    (
        Faith,
        "controls_holy_site_with_flag",
        Item(Item::HolySiteFlag),
    ),
    (Character, "council_task_monthly_progress", CompareValue),
    (LandedTitle, "county_control", CompareValue),
    (LandedTitle, "county_control_rate", CompareValue),
    (LandedTitle, "county_control_rate_modifier", CompareValue),
    (LandedTitle, "county_holder_opinion", CompareValue),
    (LandedTitle, "county_opinion", CompareValue),
    (
        LandedTitle,
        "county_opinion_target",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "court_grandeur_base", CompareValue),
    (Character, "court_grandeur_current", CompareValue),
    (Character, "court_grandeur_current_level", CompareValue),
    (Character, "court_grandeur_minimum_expected", CompareValue),
    (
        Character,
        "court_grandeur_minimum_expected_level",
        CompareValue,
    ),
    (
        Character,
        "court_positions_currently_avaiable",
        CompareValue,
    ),
    (Character, "court_positions_currently_filled", CompareValue),
    (
        Character,
        "create_faction_type_chance",
        Block(&[
            ("type", Item(Item::Faction)),
            ("target", Scope(Character)),
            ("value", CompareValue),
        ]),
    ),
    (
        Culture,
        "cultural_acceptance",
        Block(&[("target", Scope(Culture)), ("value", CompareValue)]),
    ),
    (Culture, "culture_age", CompareValueWarnEq),
    (Culture, "culture_number_of_counties", CompareValue),
    (
        Culture,
        "culture_overlaps_geographical_region",
        Item(Item::Region),
    ),
    (None, "current_computer_date", CompareDate),
    (None, "current_computer_date_day", CompareValue),
    (None, "current_computer_date_month", CompareValue),
    (None, "current_computer_date_year", CompareValue),
    (TravelPlan, "current_danger_value", CompareValue),
    (None, "current_date", CompareDate),
    (None, "current_day", CompareValue),
    (Character, "current_military_strength", CompareValue),
    (None, "current_month", CompareValue),
    (None, "current_tooltip_depth", CompareValue),
    (Character, "current_weight", CompareValue),
    (Character, "current_weight_for_portrait", CompareValue),
    (ALL_BUT_NONE, "current_year", CompareValue), // should be None scope, but current_year is buggy
    (None, "custom_description", Control),
    (None, "custom_tooltip", Special),
    (Character, "days_as_ruler", CompareValue),
    (Character, "days_in_prison", CompareValue),
    (Character, "days_of_continuous_peace", CompareValue),
    (Character, "days_of_continuous_war", CompareValue),
    (Inspiration, "days_since_creation", CompareValue),
    (Character, "days_since_death", CompareValue),
    (Character, "days_since_joined_court", CompareValue),
    (War, "days_since_max_war_score", CompareValue),
    (Inspiration, "days_since_sponsorship", CompareValue),
    (TravelPlan, "days_travelled", CompareValue),
    (GreatHolyWar, "days_until_ghw_launch", CompareValue),
    (
        LandedTitle,
        "de_jure_drift_progress",
        Block(&[("target", Scope(LandedTitle)), ("value", CompareValue)]),
    ),
    (LandedTitle, "de_jure_drifting_towards", Scope(LandedTitle)),
    (Character, "death_reason", Item(Item::DeathReason)),
    (Character, "debt_level", CompareValue),
    (None, "debug_log", UncheckedValue),
    (None, "debug_log_details", UncheckedValue),
    (None, "debug_only", Boolean),
    (War, "defender_war_score", CompareValue),
    (TravelPlan, "departure_date", CompareValue),
    (LandedTitle, "development_level", CompareValue),
    (LandedTitle, "development_rate", CompareValue),
    (LandedTitle, "development_rate_modifier", CompareValue),
    (
        LandedTitle,
        "development_towards_level_increase",
        CompareValue,
    ),
    (Character, "diarch_aptitude", CompareValue),
    (Character, "diarch_loyalty", CompareValue),
    (Character, "diarchy_swing", CompareValue),
    (Character, "diplomacy", CompareValue),
    (
        Character,
        "diplomacy_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "diplomacy_for_portrait", CompareValue),
    (Faction, "discontent_per_month", CompareValue),
    (
        Character,
        "does_ai_liege_in_vassal_contract_desire_obligation_change",
        Boolean,
    ),
    (
        Character,
        "does_ai_vassal_in_vassal_contract_desire_obligation_change",
        Boolean,
    ),
    (Character, "domain_limit", CompareValue),
    (Character, "domain_limit_available", CompareValue),
    (Character, "domain_limit_percentage", CompareValue),
    (Character, "domain_size", CompareValue),
    (
        Character,
        "domain_size_excluding_grace_period",
        CompareValue,
    ),
    // NOTE: documentation says `character` but it's `dreaded_character`
    (Character, "dread", CompareValue),
    (
        Character,
        "dread_modified_ai_boldness",
        Block(&[
            ("dreaded_character", Scope(Character)),
            ("value", CompareValue),
        ]),
    ),
    (Dynasty, "dynasty_can_unlock_relevant_perk", Boolean),
    (Dynasty, "dynasty_num_unlocked_perks", CompareValue),
    (Dynasty, "dynasty_prestige", CompareValueWarnEq),
    (Dynasty, "dynasty_prestige_level", CompareValue),
    (Character, "effective_age", CompareValue),
    (
        Character,
        "employs_court_position",
        Item(Item::CourtPosition),
    ),
    (Faith, "estimated_faith_strength", CompareValue),
    (None, "exists", Special),
    (Faction, "faction_can_press_demands", Boolean),
    (Faction, "faction_discontent", CompareValue),
    (Faction, "faction_is_at_war", Boolean),
    (Faction, "faction_is_type", Item(Item::Faction)),
    (Faction, "faction_power", CompareValue),
    (Faction, "faction_power_threshold", CompareValue),
    (
        Faith,
        "faith_hostility_level",
        Block(&[("target", Scope(Faith)), ("value", CompareValue)]),
    ),
    (
        Faith,
        "faith_hostility_level_comparison",
        ScopeCompare(Faith),
    ),
    (Character, "fertility", CompareValue),
    (Faith, "fervor", CompareValue),
    (TravelPlan, "final_destination_arrival_date", CompareDate),
    (TravelPlan, "final_destination_arrival_days", CompareValue),
    (TravelPlan, "final_destination_progress", CompareValue),
    (Character, "focus_progress", CompareValue),
    (Province, "fort_level", CompareValue),
    (Province, "free_building_slots", CompareValue),
    (None, "game_start_date", CompareDate),
    (Province, "geographical_region", Item(Item::Region)),
    (GreatHolyWar, "ghw_attackers_strength", CompareValue),
    (GreatHolyWar, "ghw_defenders_strength", CompareValue),
    (GreatHolyWar, "ghw_war_chest_gold", CompareValueWarnEq),
    (GreatHolyWar, "ghw_war_chest_piety", CompareValueWarnEq),
    (GreatHolyWar, "ghw_war_chest_prestige", CompareValueWarnEq),
    (
        None,
        "global_variable_list_size",
        Block(&[("name", UncheckedValue), ("target", CompareValue)]),
    ),
    (Character, "gold", CompareValueWarnEq),
    (
        Character,
        "government_allows",
        Choice(&["create_cadet_branches"]),
    ),
    (
        Character,
        "government_disallows",
        Choice(&["create_cadet_branches"]),
    ),
    (Character, "government_has_flag", Item(Item::GovernmentFlag)),
    (
        Accolade,
        "has_accolade_category",
        Item(Item::AccoladeCategory),
    ),
    (
        Accolade,
        "has_accolade_parameter",
        Item(Item::AccoladeParameter),
    ),
    (Accolade, "has_accolade_type", Item(Item::AccoladeType)),
    (Character, "has_active_diarchy", Boolean),
    (Activity, "has_active_locale", Item(Item::ActivityLocale)),
    (Character, "has_active_mandate", Item(Item::DiarchyMandate)),
    (Character, "has_activity_intent", Item(Item::ActivityIntent)),
    // TODO: check that the given ActivityOption belongs to the given ActivityOptionCategory
    (
        Activity,
        "has_activity_option",
        Block(&[
            ("category", Item(Item::ActivityOptionCategory)),
            ("option", Item(Item::ActivityOption)),
        ]),
    ),
    (Character, "has_activity_state", Item(Item::ActivityState)),
    (
        Activity,
        "has_activity_type",
        ScopeOrItem(ActivityType, Item::ActivityType),
    ),
    (
        Culture,
        "has_all_innovations",
        Block(&[
            ("?with_flag", Item(Item::InnovationFlag)),
            ("?without_flag", Item(Item::InnovationFlag)),
            ("?culture_era", Item(Item::CultureEra)),
        ]),
    ),
    (Faith, "has_allowed_gender_for_clergy", Scope(Character)),
    (Character, "has_any_artifact", Boolean),
    (Character, "has_any_artifact_claim", Boolean),
    (Character, "has_any_cb_on", Scope(Character)),
    (Character, "has_any_court_position", Boolean),
    (Character, "has_any_display_cb_on", Scope(Character)),
    (Character, "has_any_focus", Boolean),
    (Character, "has_any_nickname", Boolean),
    (Character, "has_any_scripted_relation", Scope(Character)),
    (Character, "has_any_secret_relation", Scope(Character)),
    (Character, "has_any_secrets", Boolean),
    (Character, "has_any_unequipped_artifact", Boolean),
    (Character, "has_artifact_claim", Scope(Artifact)),
    (
        Artifact,
        "has_artifact_feature",
        Item(Item::ArtifactFeature),
    ),
    (
        Artifact,
        "has_artifact_feature_group",
        Item(Item::ArtifactFeatureGroup),
    ),
    (Artifact, "has_artifact_modifier", Item(Item::Modifier)),
    (Character, "has_bad_nickname", Boolean),
    (Character, "has_banish_reason", Scope(Character)),
    (Province, "has_building", Item(Item::Building)),
    (Culture, "has_building_gfx", Item(Item::BuildingGfx)),
    (Province, "has_building_or_higher", Item(Item::Building)),
    (
        Province,
        "has_building_with_flag",
        ItemOrBlock(
            Item::BuildingFlag,
            &[("flag", Item(Item::BuildingFlag)), ("count", CompareValue)],
        ),
    ),
    (
        Character,
        "has_cb_on",
        Block(&[("target", Scope(Character)), ("cb", Item(Item::CasusBelli))]),
    ),
    (Character, "has_character_flag", UncheckedValue),
    (Character, "has_character_modifier", Item(Item::Modifier)),
    (
        Character,
        "has_character_modifier_duration_remaining",
        Item(Item::Modifier),
    ),
    (LandedTitle, "has_character_nominiated", Scope(Character)), // sic
    (Character, "has_claim_on", Scope(LandedTitle)),
    (Culture, "has_clothing_gfx", Item(Item::ClothingGfx)),
    (Culture, "has_coa_gfx", Item(Item::CoaGfx)),
    (
        Character,
        "has_completed_activity_intent",
        ItemOrBlock(
            Item::ActivityIntent,
            &[
                ("type", Item(Item::ActivityIntent)),
                ("?target", Scope(Character)),
            ],
        ),
    ),
    (Character, "has_completed_inspiration", Boolean),
    (
        Province,
        "has_construction_with_flag",
        Item(Item::BuildingFlag),
    ),
    (
        Character,
        "has_council_position",
        Item(Item::CouncilPosition),
    ),
    (
        Character,
        "has_councillor_for_skill",
        Choice(&[
            "diplomacy",
            "intrigue",
            "learning",
            "martial",
            "prowess",
            "stewardship",
            "general",
        ]),
    ),
    (LandedTitle, "has_county_modifier", Item(Item::Modifier)),
    (
        LandedTitle,
        "has_county_modifier_duration_remaining",
        Item(Item::Modifier),
    ),
    (Character, "has_court_language", Item(Item::Language)),
    (Character, "has_court_language_of_culture", Scope(Culture)),
    (Character, "has_court_position", Item(Item::CourtPosition)),
    (Character, "has_court_type", Item(Item::CourtType)),
    (Culture, "has_cultural_era_or_later", Item(Item::CultureEra)),
    (
        Culture,
        "has_cultural_parameter",
        Item(Item::CultureParameter),
    ),
    (Culture, "has_cultural_pillar", Item(Item::CulturePillar)),
    (
        Culture,
        "has_cultural_tradition",
        Item(Item::CultureTradition),
    ),
    (Character, "has_culture", Scope(Culture)),
    (Activity, "has_current_phase", Item(Item::ActivityPhase)),
    (Character, "has_de_jure_claim_on", Scope(Character)),
    (
        Character,
        "has_diarchy_active_parameter",
        Item(Item::DiarchyParameter),
    ),
    (
        Character,
        "has_diarchy_parameter",
        Item(Item::DiarchyParameter),
    ),
    (Character, "has_diarchy_type", Item(Item::DiarchyType)),
    (LandedTitle, "has_disabled_building", Boolean),
    (Character, "has_divorce_reason", Scope(Character)),
    (None, "has_dlc", Item(Item::Dlc)),
    (None, "has_dlc_feature", Item(Item::DlcFeature)),
    (Faith, "has_doctrine", ScopeOrItem(Doctrine, Item::Doctrine)),
    (
        Faith,
        "has_doctrine_parameter",
        Item(Item::DoctrineParameter),
    ),
    (Faith, "has_dominant_ruling_gender", Scope(Character)),
    (
        Character,
        "has_dread_level_towards",
        Block(&[("target", Scope(Character)), ("level", CompareValue)]),
    ),
    (Character, "has_dynasty", Boolean),
    (Dynasty, "has_dynasty_modifier", Item(Item::Modifier)),
    (
        Dynasty,
        "has_dynasty_modifier_duration_remaining",
        Item(Item::Modifier),
    ),
    (Dynasty, "has_dynasty_perk", Item(Item::DynastyPerk)),
    (
        Character,
        "has_election_vote_of",
        Block(&[("who", Scope(Character)), ("title", Scope(LandedTitle))]),
    ),
    (Character, "has_employed_any_court_position", Boolean),
    (Character, "has_execute_reason", Scope(Character)),
    (Character, "has_faith", Scope(Faith)),
    (Character, "has_father", Boolean),
    (Character, "has_focus", Item(Item::Focus)),
    (GreatHolyWar, "has_forced_defender", Scope(Character)),
    (Province, "has_free_building_slot", Boolean),
    (Character, "has_free_council_slot", Boolean),
    (None, "has_game_rule", Item(Item::GameRule)),
    (Character, "has_gene", Special),
    (None, "has_global_variable", UncheckedValue),
    (None, "has_global_variable_list", UncheckedValue),
    (Character, "has_government", Item(Item::GovernmentType)),
    (Faith, "has_graphical_faith", Item(Item::GraphicalFaith)),
    (Character, "has_had_focus_for_days", CompareValue),
    (Province, "has_holding", Boolean),
    (Province, "has_holding_type", Item(Item::Holding)),
    (LandedTitle, "has_holy_site_flag", Item(Item::HolySiteFlag)),
    (Character, "has_hook", Scope(Character)),
    (Character, "has_hook_from_secret", Scope(Secret)),
    (
        Character,
        "has_hook_of_type",
        Block(&[("target", Scope(Character)), ("type", Item(Item::Hook))]),
    ),
    (DynastyHouse, "has_house_artifact_claim", Scope(Artifact)),
    (DynastyHouse, "has_house_modifier", Item(Item::Modifier)),
    (
        DynastyHouse,
        "has_house_modifier_duration_remaining",
        Item(Item::Modifier),
    ),
    (Faith, "has_icon", Item(Item::FaithIcon)),
    (Character, "has_imprisonment_reason", Scope(Character)),
    (Character, "has_inactive_trait", Item(Item::Trait)),
    (Culture, "has_innovation", Item(Item::Innovation)),
    (Culture, "has_innovation_flag", Item(Item::InnovationFlag)),
    (Inspiration, "has_inspiration_type", Item(Item::Inspiration)),
    (Character, "has_lifestyle", Item(Item::Lifestyle)),
    (None, "has_local_player_open_court_event", Boolean),
    (None, "has_local_player_seen_unopened_court_event", Boolean),
    (None, "has_local_player_unopened_court_event", Boolean),
    (None, "has_local_variable", UncheckedValue),
    (None, "has_local_variable_list", UncheckedValue),
    (CombatSide, "has_maa_of_type", Item(Item::MenAtArms)),
    (None, "has_map_mode", Item(Item::MapMode)),
    (
        CharacterMemory,
        "has_memory_category",
        Item(Item::MemoryCategory),
    ),
    (CharacterMemory, "has_memory_participant", Scope(Character)),
    (CharacterMemory, "has_memory_type", Item(Item::MemoryType)),
    (Character, "has_mother", Boolean),
    (None, "has_multiple_players", Boolean),
    (Culture, "has_name_list", Item(Item::NameList)),
    (Character, "has_nickname", Item(Item::Nickname)),
    (Province, "has_ongoing_construction", Boolean),
    (
        Character,
        "has_opinion_modifier",
        Block(&[
            ("target", Scope(Character)),
            ("modifier", Item(Item::OpinionModifier)),
            ("?value", CompareValue),
        ]),
    ),
    (Character, "has_opposite_relation", Item(Item::Relation)),
    // TODO: figure out what else this can be
    (
        LandedTitle,
        "has_order_of_succession",
        Choice(&["election"]),
    ),
    (Character, "has_outstanding_artifact_claims", Boolean),
    (Character, "has_owned_scheme", Boolean),
    (Character, "has_pending_court_events", Boolean),
    (
        Character,
        "has_pending_interaction_of_type",
        Item(Item::Interaction),
    ),
    (Character, "has_perk", Item(Item::Perk)),
    (Character, "has_personal_artifact_claim", Scope(Artifact)),
    (
        Activity,
        "has_phase",
        ItemOrBlock(
            Item::ActivityPhase,
            &[
                ("?type", Item(Item::ActivityPhase)),
                ("?location", Scope(Province)),
            ],
        ),
    ),
    (
        Activity,
        "has_phase_future",
        ItemOrBlock(
            Item::ActivityPhase,
            &[
                ("?type", Item(Item::ActivityPhase)),
                ("?location", Scope(Province)),
            ],
        ),
    ),
    (
        Activity,
        "has_phase_past",
        ItemOrBlock(
            Item::ActivityPhase,
            &[
                ("?type", Item(Item::ActivityPhase)),
                ("?location", Scope(Province)),
            ],
        ),
    ),
    (GreatHolyWar, "has_pledged_attacker", Scope(Character)),
    (GreatHolyWar, "has_pledged_defender", Scope(Character)),
    (Character, "has_potential_acclaimed_knights", Boolean),
    (Accolade, "has_potential_accolade_successors", Boolean),
    (Faith, "has_preferred_gender_for_clergy", Scope(Character)),
    (Culture, "has_primary_name_list", Item(Item::NameList)),
    (Character, "has_primary_title", Scope(LandedTitle)),
    (Character, "has_prisoners", Boolean),
    (Province, "has_province_modifier", Item(Item::Modifier)),
    (
        Province,
        "has_province_modifier_duration_remaining",
        Item(Item::Modifier),
    ),
    (Character, "has_raid_immunity_against", Scope(Character)),
    (Character, "has_raised_armies", Boolean),
    (Character, "has_realm_law", Item(Item::Law)),
    (Character, "has_realm_law_flag", Item(Item::LawFlag)),
    (
        Character,
        "has_relation_flag",
        Block(&[
            ("target", Scope(Character)),
            ("relation", Item(Item::Relation)),
            ("flag", Item(Item::RelationFlag)),
        ]),
    ),
    (Character, "has_relation_to", Scope(Character)),
    (Character, "has_religion", Scope(Religion)),
    (LandedTitle, "has_revokable_lease", Boolean),
    (Character, "has_revoke_title_reason", Scope(Character)),
    (None, "has_reward_item", Item(Item::RewardItem)),
    (Character, "has_royal_court", Boolean),
    (Character, "has_same_court_language", Scope(Character)),
    (Character, "has_same_court_type_as", Scope(Character)),
    (Character, "has_same_culture_as", Scope(Character)),
    (Culture, "has_same_culture_ethos", Scope(Culture)),
    (Culture, "has_same_culture_heritage", Scope(Culture)),
    (Culture, "has_same_culture_language", Scope(Culture)),
    (
        Culture,
        "has_same_culture_martial_tradition",
        Scope(Culture),
    ),
    (Character, "has_same_focus_as", Scope(Character)),
    (Character, "has_same_government", Scope(Character)),
    (Character, "has_same_sinful_trait", Scope(Character)),
    (Character, "has_same_virtue_trait", Scope(Character)),
    (Scheme, "has_scheme_modifier", Item(Item::Modifier)),
    (
        Character,
        "has_selected_mandate",
        Item(Item::DiarchyMandate),
    ),
    (Character, "has_sexuality", Item(Item::Sexuality)),
    (Character, "has_spawned_court_events", Boolean),
    (Province, "has_special_building", Boolean),
    (Province, "has_special_building_slot", Boolean),
    (Faction, "has_special_character", Boolean),
    (Faction, "has_special_title", Boolean),
    (Province, "has_stationed_regiment", Boolean),
    (
        Province,
        "has_stationed_regiment_of_base_type",
        Item(Item::MenAtArmsBase),
    ),
    (Character, "has_strong_claim_on", Scope(LandedTitle)),
    (Character, "has_strong_hook", Scope(Character)),
    (Character, "has_strong_usable_hook", Scope(Character)),
    (
        Struggle,
        "has_struggle_phase_parameter",
        Item(Item::StrugglePhaseParameter),
    ),
    (Character, "has_targeting_faction", Boolean),
    (Character, "has_title", Scope(LandedTitle)),
    (LandedTitle, "has_title_law", Item(Item::TitleLaw)),
    (LandedTitle, "has_title_law_flag", Item(Item::TitleLawFlag)),
    (Character, "has_trait", Item(Item::Trait)),
    (Trait, "has_trait_category", Item(Item::TraitCategory)),
    (Trait, "has_trait_flag", Item(Item::TraitFlag)),
    (
        Character,
        "has_trait_rank",
        Block(&[
            ("trait", Item(Item::Trait)),
            ("*rank", CompareValue),
            ("*character", CompareToScope(Character)),
        ]),
    ),
    (Character, "has_trait_with_flag", Item(Item::TraitFlag)),
    // TODO: "track name is required if the trait has multiple tracks, otherwise should not be provided."
    (
        Character,
        "has_trait_xp",
        Block(&[
            ("trait", Item(Item::Trait)),
            ("?track", Item(Item::TraitTrack)),
            ("value", CompareValue),
        ]),
    ),
    (TravelPlan, "has_travel_option", Item(Item::TravelOption)),
    (
        TravelPlan,
        "has_travel_plan_modifier",
        Item(Item::TravelPlanModifier),
    ),
    (
        Province,
        "has_travel_point_of_interest",
        Item(Item::PointOfInterest),
    ),
    (Character, "has_truce", Scope(Character)),
    (Culture, "has_unit_gfx", Item(Item::UnitGfx)),
    (Character, "has_usable_hook", Scope(Character)),
    (LandedTitle, "has_user_set_coa", Boolean),
    (War, "has_valid_casus_belli", Boolean),
    (None, "has_variable", UncheckedValue),
    (None, "has_variable_list", UncheckedValue),
    (Character, "has_vassal_stance", Item(Item::VassalStance)),
    (
        None,
        "has_war_result_message_with_outcome",
        Choice(&["victory", "defeat", "white_peace", "invalidated", "any"]),
    ),
    (Character, "has_weak_claim_on", Scope(LandedTitle)),
    (Character, "has_weak_hook", Scope(Character)),
    (LandedTitle, "has_wrong_holding_type", Boolean),
    (Character, "health", CompareValue),
    (Character, "highest_held_title_tier", CompareValue),
    (Character, "highest_skill", Item(Item::Skill)),
    (Character, "holds_landed_title", Boolean),
    (Faith, "holy_sites_controlled", CompareValue),
    (
        Character,
        "important_action_is_valid_but_invisible",
        Item(Item::ImportantAction),
    ),
    (
        Character,
        "important_action_is_visible",
        Item(Item::ImportantAction),
    ),
    (Character, "in_diplomatic_range", Scope(Character)),
    (Inspiration, "inspiration_gold_invested", CompareValueWarnEq),
    (Inspiration, "inspiration_progress", CompareValue),
    (Character, "intrigue", CompareValue),
    (
        Character,
        "intrigue_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "intrigue_for_portrait", CompareValue),
    (Character, "is_a_faction_leader", Boolean),
    (Character, "is_a_faction_member", Boolean),
    (TravelPlan, "is_aborted", Boolean),
    (Character, "is_acclaimed", Boolean),
    (Accolade, "is_accolade_active", Boolean),
    (Character, "is_accolade_successor", Boolean),
    (Activity, "is_activity_complete", Boolean),
    (
        Character,
        "is_activity_type_on_cooldown",
        Scope(ActivityType),
    ),
    (Character, "is_adult", Boolean),
    (Character, "is_agent_exposed_in_scheme", Scope(Scheme)),
    (Character, "is_ai", Boolean),
    (Character, "is_alive", Boolean),
    (Character, "is_allied_in_war", Scope(Character)),
    (Character, "is_allied_to", Scope(Character)),
    (Army, "is_army_in_combat", Boolean),
    (Army, "is_army_in_raid", Boolean),
    (Army, "is_army_in_siege", Boolean),
    (Army, "is_army_in_siege_relevant_for", Scope(Character)),
    (Character, "is_at_home", Boolean),
    (Character, "is_at_location", Scope(Province)),
    (Character, "is_at_same_location", Scope(Character)),
    (Character, "is_at_war", Boolean),
    (Character, "is_at_war_as_attacker", Boolean),
    (Character, "is_at_war_as_defender", Boolean),
    (Character, "is_at_war_with", Scope(Character)),
    (Character, "is_at_war_with_liege", Boolean),
    (War, "is_attacker", Scope(Character)),
    (Character, "is_attacker_in_war", Scope(War)),
    (Character, "is_attracted_to_gender_of", Scope(Character)),
    (Character, "is_attracted_to_men", Boolean),
    (Character, "is_attracted_to_women", Boolean),
    (Character, "is_away_from_court", Boolean),
    (None, "is_bad_nickname", Item(Item::Nickname)),
    (Character, "is_betrothed", Boolean),
    (TravelPlan, "is_cancelled", Boolean),
    (LandedTitle, "is_capital_barony", Boolean),
    (
        Character,
        "is_causing_raid_hostility_towards",
        Scope(Character),
    ),
    (
        Character,
        "is_character_interaction_potentially_accepted",
        Block(&[
            ("recipient", Scope(Character)),
            ("interaction", Item(Item::Interaction)),
            ("?secondary_actor", Scope(Character)),
            ("?secondary_recipient", Scope(Character)),
            ("?target_title", Scope(LandedTitle)),
        ]),
    ),
    (
        Character,
        "is_character_interaction_shown",
        Block(&[
            ("recipient", Scope(Character)),
            ("interaction", Item(Item::Interaction)),
        ]),
    ),
    (
        Character,
        "is_character_interaction_valid",
        Block(&[
            ("recipient", Scope(Character)),
            ("interaction", Item(Item::Interaction)),
        ]),
    ),
    (Character, "is_character_window_main_character", Boolean),
    (Character, "is_child_of", Scope(Character)),
    (War, "is_civil_war", Boolean),
    (Character, "is_claimant", Boolean),
    (Character, "is_clergy", Boolean),
    (Character, "is_close_family_of", Scope(Character)),
    (
        Character,
        "is_close_or_extended_family_of",
        Scope(Character),
    ),
    (Province, "is_coastal", Boolean),
    (LandedTitle, "is_coastal_county", Boolean),
    (CombatSide, "is_combat_side_attacker", Boolean),
    (CombatSide, "is_combat_side_pursuing", Boolean),
    (CombatSide, "is_combat_side_retreating", Boolean),
    (Character, "is_commanding_army", Boolean),
    (TravelPlan, "is_completed", Boolean),
    (Character, "is_concubine", Boolean),
    (Character, "is_concubine_of", Scope(Character)),
    // TODO: check that the landed titles are counties
    (
        LandedTitle,
        "is_connected_to",
        Block(&[
            ("?max_naval_distance", SetValue),
            ("?allow_one_county_land_gap", Boolean),
            ("target", Scope(LandedTitle)),
        ]),
    ),
    (Character, "is_consort_of", Scope(Character)),
    (LandedTitle, "is_contested", Boolean),
    (
        Character,
        "is_council_task_valid",
        Block(&[
            ("task_type", Item(Item::CouncilTask)),
            ("?target", Scope(Character | LandedTitle)),
        ]),
    ),
    (Character, "is_councillor", Boolean),
    (Character, "is_councillor_of", Scope(Character)),
    (Province, "is_county_capital", Boolean),
    (
        Character,
        "is_court_position_employer",
        Block(&[
            ("court_position", Item(Item::CourtPosition)),
            ("who", Scope(Character)),
        ]),
    ),
    (Character, "is_courtier", Boolean),
    (Character, "is_courtier_of", Scope(Character)),
    (Character, "is_cousin_of", Scope(Character)),
    (Secret, "is_criminal_for", Scope(Character)),
    (Struggle, "is_culture_involved_in_struggle", Scope(Culture)),
    (Activity, "is_current_phase_active", Boolean),
    (
        LandedTitle,
        "is_de_facto_liege_or_above_target",
        Scope(LandedTitle),
    ),
    (
        LandedTitle,
        "is_de_jure_liege_or_above_target",
        Scope(LandedTitle),
    ),
    (
        Character,
        "is_decision_on_cooldown",
        ScopeOrItem(Decision, Item::Decision),
    ),
    (War, "is_defender", Scope(Character)),
    (Character, "is_defender_in_war", Scope(War)),
    (Character, "is_designated_diarch", Boolean),
    (Character, "is_diarch", Boolean),
    (Character, "is_diarch_of_target", Scope(Character)),
    (Character, "is_diarchy_successor", Boolean),
    (GreatHolyWar, "is_directed_ghw", Boolean),
    (Culture, "is_divergent_culture", Boolean),
    (Character, "is_employer_of", Scope(Character)),
    (Artifact, "is_equipped", Boolean),
    (Character, "is_extended_family_of", Scope(Character)),
    (Struggle, "is_faith_involved_in_struggle", Scope(Faith)),
    (Character, "is_female", Boolean),
    (Character, "is_forbidden_from_scheme", Scope(Scheme)),
    (Character, "is_forced_into_faction", Boolean),
    (Character, "is_forced_into_scheme", Scope(Scheme)),
    (Character, "is_foreign_court_guest", Boolean),
    (Character, "is_foreign_court_guest_of", Scope(Character)),
    (Character, "is_foreign_court_or_pool_guest", Boolean),
    (
        Character,
        "is_foreign_court_or_pool_guest_of",
        Scope(Character),
    ),
    (Character, "is_from_ruler_designer", Boolean),
    (None, "is_game_view_open", UncheckedValue), // TODO
    (None, "is_gamestate_tutorial_active", Boolean),
    (Character, "is_grandchild_of", Scope(Character)),
    (Character, "is_grandparent_of", Scope(Character)),
    (Character, "is_great_grandchild_of", Scope(Character)),
    (Character, "is_great_grandparent_of", Scope(Character)),
    (LandedTitle, "is_head_of_faith", Boolean),
    (Character, "is_heir_of", Scope(Character)),
    (LandedTitle, "is_holy_order", Boolean),
    (LandedTitle, "is_holy_site", Boolean),
    (LandedTitle, "is_holy_site_controlled_by", Scope(Character)),
    (LandedTitle, "is_holy_site_of", Scope(Faith)),
    (Scheme, "is_hostile", Boolean),
    (Culture, "is_hybrid_culture", Boolean),
    (Character, "is_immortal", Boolean),
    (Character, "is_important_decision", Scope(Decision)),
    (Character, "is_imprisoned", Boolean),
    (Character, "is_imprisoned_by", Scope(Character)),
    (Character, "is_in_army", Boolean),
    (Character, "is_in_civil_war", Boolean),
    (Religion, "is_in_family", Item(Item::ReligionFamily)),
    (
        Character,
        "is_in_guest_subset",
        Block(&[
            ("name", Item(Item::GuestSubset)),
            ("?phase", Item(Item::ActivityPhase)),
        ]),
    ),
    (None, "is_in_list", UncheckedValue),
    (Character, "is_in_ongoing_great_holy_war", Boolean),
    (Character, "is_in_pool_at", Scope(Province)),
    (Character, "is_in_prison_type", Item(Item::PrisonType)),
    (Character, "is_in_the_same_court_as", Scope(Character)),
    (
        Character,
        "is_in_the_same_court_as_or_guest",
        Scope(Character),
    ),
    (Character, "is_incapable", Boolean),
    (Character, "is_independent_ruler", Boolean),
    (Character, "is_knight", Boolean),
    (Character, "is_knight_of", Scope(Character)),
    (Secret, "is_known_by", Scope(Character)),
    (Character, "is_landed", Boolean),
    (Character, "is_landless_ruler", Boolean),
    (LandedTitle, "is_landless_type_title", Boolean),
    (Character, "is_leader_in_war", Scope(War)),
    (Character, "is_leading_faction_type", Item(Item::Faction)),
    (LandedTitle, "is_leased_out", Boolean),
    (Character, "is_liege_or_above_of", Scope(Character)),
    (Character, "is_local_player", Boolean),
    (Character, "is_lowborn", Boolean),
    (Character, "is_male", Boolean),
    (Character, "is_married", Boolean),
    (CharacterMemory, "is_memory_of_travel", Scope(TravelPlan)),
    (LandedTitle, "is_mercenary_company", Boolean),
    (Character, "is_mercenary_in_hire_range", Scope(Character)),
    (LandedTitle, "is_neighbor_to_realm", Scope(Character)),
    (Character, "is_nibling_of", Scope(Character)),
    (Character, "is_normal_councillor", Boolean),
    (Province, "is_occupied", Boolean),
    (Activity, "is_open_invite_activity", Boolean),
    (Trait, "is_opposite_of_trait", Scope(Trait)),
    (Character, "is_overriding_designated_winner", Boolean),
    (Character, "is_parent_of", Scope(Character)),
    (War, "is_participant", Scope(Character)),
    (Character, "is_participant_in_activity", Scope(Activity)),
    (Character, "is_participant_in_war", Scope(War)),
    (TravelPlan, "is_paused", Boolean),
    (
        Character,
        "is_performing_council_task",
        Item(Item::CouncilTask),
    ),
    (Character, "is_player_heir_of", Scope(Character)),
    (None, "is_player_selected", Boolean),
    (Character, "is_pledged_ghw_attacker", Boolean),
    (Character, "is_pool_character", Boolean),
    (Character, "is_pool_guest", Boolean),
    (Character, "is_pool_guest_of", Scope(Character)),
    (Character, "is_potential_knight", Boolean),
    (Character, "is_powerful_vassal", Boolean),
    (Character, "is_powerful_vassal_of", Scope(Character)),
    (Character, "is_pregnant", Boolean),
    (Character, "is_primary_heir_of", Scope(Character)),
    (Army, "is_raid_army", Boolean),
    (Province, "is_raided", Boolean),
    (Activity, "is_required_special_guest", Scope(Character)),
    (Province, "is_river_province", Boolean),
    (LandedTitle, "is_riverside_county", Boolean),
    (Province, "is_riverside_province", Boolean),
    (Character, "is_ruler", Boolean),
    (Scheme, "is_scheme_agent_exposed", Scope(Character)),
    (Scheme, "is_scheme_exposed", Boolean),
    // TODO: the documentation says scheme_skill but the single example in vanilla uses just skill. Should verify.
    (
        Character,
        "is_scheming_against",
        Block(&[
            ("target", Scope(Character)),
            ("?type", Item(Item::Scheme)),
            ("?scheme_skill", Item(Item::Skill)),
        ]),
    ),
    (Province, "is_sea_province", Boolean),
    (None, "is_set", Scope(ALL_BUT_NONE)),
    (Secret, "is_shunned_for", Scope(Character)),
    (Secret, "is_shunned_or_criminal_for", Scope(Character)),
    (Character, "is_sibling_of", Scope(Character)),
    (
        Activity,
        "is_special_guest",
        ScopeOrBlock(
            Character,
            &[
                ("target", Scope(Character)),
                ("type", Item(Item::GuestSubset)),
            ],
        ),
    ),
    (Secret, "is_spent_by", Scope(Character)),
    (Character, "is_spouse_of", Scope(Character)),
    (Character, "is_spouse_of_even_if_dead", Scope(Character)),
    (Struggle, "is_struggle_phase", Item(Item::StrugglePhase)),
    (Struggle, "is_struggle_type", Item(Item::Struggle)),
    (Character, "is_successor_of_accolade", Scope(Accolade)),
    (
        None,
        "is_target_in_global_variable_list",
        Block(&[("name", UncheckedValue), ("*target", Scope(ALL_BUT_NONE))]),
    ),
    (
        None,
        "is_target_in_local_variable_list",
        Block(&[("name", UncheckedValue), ("*target", Scope(ALL_BUT_NONE))]),
    ),
    (
        None,
        "is_target_in_variable_list",
        Block(&[("name", UncheckedValue), ("*target", Scope(ALL_BUT_NONE))]),
    ),
    (
        LandedTitle,
        "is_target_of_council_task",
        Item(Item::CouncilTask),
    ),
    (Character, "is_theocratic_lessee", Boolean),
    (LandedTitle, "is_title_created", Boolean),
    (LandedTitle, "is_titular", Boolean),
    (None, "is_tooltip_with_name_open", UncheckedValue), // TODO
    (Character, "is_travel_entourage_character", Boolean),
    (Character, "is_travel_leader", Boolean),
    (Character, "is_travelling", Boolean),
    (None, "is_tutorial_active", Boolean),
    (None, "is_tutorial_lesson_active", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_chain_completed", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_completed", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_step_completed", UncheckedValue), // TODO
    (Character, "is_twin_of", Scope(Character)),
    (Scheme, "is_type_secret", Boolean),
    (Character, "is_unborn_child_of_concubine", Boolean),
    (Character, "is_unborn_known_bastard", Boolean),
    (Character, "is_uncle_or_aunt_of", Scope(Character)),
    (LandedTitle, "is_under_holy_order_lease", Boolean),
    (Artifact, "is_unique", Boolean),
    (Character, "is_valid_as_agent_in_scheme", Scope(Scheme)),
    (Character, "is_valid_for_event_debug", Item(Item::Event)), // will not work in release mode
    (
        Character,
        "is_valid_for_event_debug_cooldown",
        Item(Item::Event),
    ), // will not work in release mode
    (
        Character,
        "is_valid_successor_for_accolade",
        Scope(Accolade),
    ),
    (Character, "is_vassal_of", Scope(Character)),
    (Character, "is_vassal_or_below_of", Scope(Character)),
    (Character, "is_visibly_fertile", Boolean),
    (War, "is_war_leader", Scope(Character)),
    (None, "is_war_overview_tab_open", UncheckedValue), // TODO
    (War, "is_white_peace_possible", Boolean),
    (None, "is_widget_open", UncheckedValue), // TODO
    (
        Character,
        "join_faction_chance",
        Block(&[("target", Scope(Faction)), ("value", CompareValue)]),
    ),
    // Documentation says `target` but it's `scheme`
    (
        Character,
        "join_scheme_chance",
        Block(&[
            ("scheme", Scope(Scheme)),
            ("max", SetValue),
            ("min", SetValue),
        ]),
    ),
    (Character, "knows_court_language_of", Scope(Character)),
    (Character, "knows_language", Item(Item::Language)),
    (Character, "knows_language_of_culture", Scope(Culture)),
    (Character, "learning", CompareValue),
    (
        Character,
        "learning_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "learning_for_portrait", CompareValue),
    (
        None,
        "list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (Secret, "local_player_knows_this_secret", Boolean),
    (
        None,
        "local_variable_list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (Character, "long_term_gold", CompareValueWarnEq),
    (Character, "long_term_gold_maximum", CompareValueWarnEq),
    (Character, "martial", CompareValue),
    (
        Character,
        "martial_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "martial_for_portrait", CompareValue),
    (Character, "matrilinear_betrothal", Boolean),
    (Character, "matrilinear_marriage", Boolean),
    (Character, "max_active_accolades", CompareValue),
    (Character, "max_military_strength", CompareValue),
    (
        Character,
        "max_number_maa_soldiers_of_base_type",
        Block(&[("type", Item(Item::MenAtArmsBase)), ("value", CompareValue)]),
    ),
    (
        Character,
        "max_number_maa_soldiers_of_type",
        Block(&[("type", Item(Item::MenAtArms)), ("value", CompareValue)]),
    ),
    (Character, "max_number_of_concubines", CompareValue),
    (Character, "max_number_of_knights", CompareValue),
    (CharacterMemory, "memory_age_years", CompareValueWarnEq),
    (CharacterMemory, "memory_creation_date", CompareDate),
    (CharacterMemory, "memory_end_date", CompareDate),
    (
        MercenaryCompany,
        "mercenary_company_expiration_days",
        CompareValue,
    ),
    (Character, "missing_unique_ancestors", CompareValue),
    (Character, "monthly_character_balance", CompareValueWarnEq),
    (Character, "monthly_character_expenses", CompareValueWarnEq),
    (Character, "monthly_character_income", CompareValueWarnEq),
    (
        Character,
        "monthly_character_income_long_term",
        CompareValueWarnEq,
    ),
    (
        Character,
        "monthly_character_income_reserved",
        CompareValueWarnEq,
    ),
    (
        Character,
        "monthly_character_income_short_term",
        CompareValueWarnEq,
    ),
    (
        Character,
        "monthly_character_income_war_chest",
        CompareValueWarnEq,
    ),
    (Province, "monthly_income", CompareValueWarnEq),
    (Character, "months_as_ruler", CompareValueWarnEq),
    (Faction, "months_until_max_discontent", CompareValueWarnEq),
    (
        Character,
        "morph_gene_attribute",
        Block(&[
            ("category", Item(Item::GeneCategory)),
            ("attribute", Item(Item::GeneAttribute)),
            ("value", CompareValue),
        ]),
    ),
    (
        Character,
        "morph_gene_value",
        Block(&[
            ("category", Item(Item::GeneCategory)),
            ("value", CompareValue),
        ]),
    ),
    (None, "nand", Control),
    (TravelPlan, "next_destination_arrival_date", CompareDate),
    (
        TravelPlan,
        "next_destination_arrival_days",
        CompareValueWarnEq,
    ),
    (TravelPlan, "next_destination_progress", CompareValue),
    (None, "nor", Control),
    (None, "not", Control), // TODO: warn about multiple triggers in a NOT ?
    (Character, "num_active_accolades", CompareValue),
    (Artifact, "num_artifact_kills", CompareValue),
    (Province, "num_buildings", CompareValue),
    (Faith, "num_character_followers", CompareValue),
    (Faith, "num_county_followers", CompareValue),
    (CombatSide, "num_enemies_killed", CompareValue),
    (TravelPlan, "num_entourage_characters", CompareValue),
    (Activity, "num_future_phases", CompareValue),
    (Character, "num_inactive_accolades", CompareValue),
    (HolyOrder, "num_leased_titles", CompareValue),
    (Character, "num_of_bad_genetic_traits", CompareValue),
    (Character, "num_of_good_genetic_traits", CompareValue),
    (Character, "num_of_known_languages", CompareValue),
    (TravelPlan, "num_options", CompareValue),
    (Activity, "num_past_phases", CompareValue),
    (Activity, "num_phases", CompareValue),
    (
        Character,
        "num_sinful_traits",
        CompareValueOrBlock(&[("value", CompareValue), ("faith", Scope(Faith))]),
    ),
    (Combat, "num_total_troops", CompareValueWarnEq),
    (
        Character,
        "num_virtuous_traits",
        CompareValueOrBlock(&[("value", CompareValue), ("faith", Scope(Faith))]),
    ),
    (
        Character,
        "number_maa_regiments_of_base_type",
        Block(&[("type", Item(Item::MenAtArmsBase)), ("value", CompareValue)]),
    ),
    (
        Character,
        "number_maa_regiments_of_type",
        Block(&[("type", Item(Item::MenAtArms)), ("value", CompareValue)]),
    ),
    (
        Character,
        "number_maa_soldiers_of_base_type",
        Block(&[("type", Item(Item::MenAtArmsBase)), ("value", CompareValue)]),
    ),
    (
        Character,
        "number_maa_soldiers_of_type",
        Block(&[("type", Item(Item::MenAtArms)), ("value", CompareValue)]),
    ),
    (Province, "number_of_characters_in_pool", CompareValue),
    (Character, "number_of_commander_traits", CompareValue),
    (
        Character,
        "number_of_commander_traits_in_common",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "number_of_concubines", CompareValue),
    (Character, "number_of_desired_concubines", CompareValue),
    (
        Character,
        "number_of_election_votes",
        Block(&[("title", Scope(Character)), ("value", CompareValue)]),
    ),
    (
        Faction,
        "number_of_faction_members_in_council",
        CompareValue,
    ),
    (Character, "number_of_fertile_concubines", CompareValue),
    (Character, "number_of_knights", CompareValue),
    (Character, "number_of_lifestyle_traits", CompareValue),
    (Character, "number_of_maa_regiments", CompareValue),
    (
        Character,
        "number_of_opposing_personality_traits",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (
        Character,
        "number_of_opposing_traits",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "number_of_personality_traits", CompareValue),
    (
        Character,
        "number_of_personality_traits_in_common",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "number_of_powerful_vassals", CompareValue),
    (
        Character,
        "number_of_sinful_traits_in_common",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "number_of_stationed_maa_regiments", CompareValue),
    (Character, "number_of_traits", CompareValue),
    (
        Character,
        "number_of_traits_in_common",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (
        Character,
        "number_of_virtue_traits_in_common",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (
        VassalContractObligationLevel,
        "obligation_level_score",
        CompareValue,
    ),
    (
        Character,
        "opinion",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (None, "or", Control),
    (Character, "owns_a_story", Boolean),
    (Character, "owns_story_of_type", Item(Item::Story)),
    (Character, "patrilinear_betrothal", Boolean),
    (Character, "patrilinear_marriage", Boolean),
    (CombatSide, "percent_enemies_killed", CompareValue),
    (Character, "perk_points", CompareValue),
    (Character, "perk_points_assigned", CompareValue),
    // perks_in_<lifestyle>
    // TODO: is "tree" here a lifestyle? No examples in vanilla
    (
        Character,
        "perks_in_tree",
        Block(&[("tree", Item(Item::PerkTree)), ("value", CompareValue)]),
    ),
    (Struggle, "phase_has_catalyst", Item(Item::Catalyst)),
    (Character, "piety", CompareValueWarnEq),
    (Character, "piety_level", CompareValue),
    (
        LandedTitle,
        "place_in_line_of_succession",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    // TODO: documentation says the field is `position`, but it's really `value`
    (
        Character,
        "player_heir_position",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Character, "pregnancy_days", CompareValue),
    (Character, "prestige", CompareValueWarnEq),
    (Character, "prestige_level", CompareValue),
    (Accolade, "primary_tier", CompareValue),
    (Character, "prowess", CompareValue),
    (
        Character,
        "prowess_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "prowess_for_portrait", CompareValue),
    (Character, "prowess_no_portrait", CompareValue),
    (Army, "raid_loot", CompareValue),
    (Character, "ransom_cost", CompareValue),
    (Artifact, "rarity", Item(Item::ArtifactRarity)),
    (Character, "realm_size", CompareValue),
    (
        Character,
        "realm_to_title_distance_squared",
        Block(&[("title", Scope(LandedTitle)), ("value", CompareValue)]),
    ),
    (
        LandedTitle,
        "recent_history",
        Block(&[
            ("?type", Item(Item::TitleHistoryType)),
            ("?days", SetValue),
            ("?months", SetValue),
            ("?years", SetValue),
        ]),
    ),
    (None, "release_only", Boolean),
    (Faith, "religion_tag", Item(Item::Religion)),
    (Character, "reserved_gold", CompareValueWarnEq),
    (Character, "reserved_gold_maximum", CompareValueWarnEq),
    (
        Character,
        "reverse_has_opinion_modifier",
        Block(&[
            ("target", Scope(Character)),
            ("modifier", Item(Item::OpinionModifier)),
            ("?value", CompareValue),
        ]),
    ),
    (
        Character,
        "reverse_opinion",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Secret, "same_secret_type_as", Scope(Secret)),
    (Character, "save_temporary_opinion_value_as", Special),
    (ALL_BUT_NONE, "save_temporary_scope_as", Special),
    (None, "save_temporary_scope_value_as", Special),
    (Scheme, "scheme_duration_days", CompareValue),
    (Scheme, "scheme_is_character_agent", Scope(Character)),
    (Scheme, "scheme_monthly_progress", CompareValue),
    (Scheme, "scheme_number_of_agents", CompareValue),
    (Scheme, "scheme_number_of_exposed_agents", CompareValue),
    (Scheme, "scheme_power", CompareValue),
    (Scheme, "scheme_power_resistance_difference", CompareValue),
    (Scheme, "scheme_power_resistance_ratio", CompareValue),
    (Scheme, "scheme_progress", CompareValue),
    (Scheme, "scheme_resistance", CompareValue),
    (Scheme, "scheme_secrecy", CompareValue),
    (Scheme, "scheme_skill", Item(Item::Skill)),
    (Scheme, "scheme_success_chance", CompareValue),
    (Scheme, "scheme_type", Item(Item::Scheme)),
    (None, "scripted_tests", Boolean),
    (
        Character,
        "scriptedtests_can_marry_character",
        Scope(Character),
    ),
    (Character, "scriptedtests_dread_base", CompareValue),
    (
        Character,
        "scriptedtests_gold_income_no_theocracy",
        CompareValue,
    ),
    (Character, "scriptedtests_piety_income", CompareValue),
    (Accolade, "secondary_tier", CompareValue),
    (Secret, "secret_type", Item(Item::Secret)),
    (Character, "sex_opposite_of", Scope(Character)),
    (Character, "sex_same_as", Scope(Character)),
    (Character, "short_term_gold", CompareValueWarnEq),
    (Character, "short_term_gold_maximum", CompareValueWarnEq),
    (Artifact, "should_decay", Boolean),
    (
        Character,
        "should_decision_create_alert",
        ScopeOrItem(Decision, Item::Decision),
    ),
    (
        Character,
        "should_notify_can_host_activity",
        ScopeOrItem(ActivityType, Item::ActivityType),
    ),
    (
        Character,
        "should_notify_can_join_open_activity",
        ScopeOrItem(ActivityType, Item::ActivityType),
    ),
    (None, "should_show_disturbing_portrait_modifiers", Boolean),
    (None, "should_show_nudity", Boolean),
    (CombatSide, "side_army_size", CompareValue),
    (CombatSide, "side_max_army_size", CompareValue),
    (CombatSide, "side_soldiers", CompareValue),
    (CombatSide, "side_strength", CompareValue),
    (
        LandedTitle | Province,
        "squared_distance",
        Block(&[
            ("target", Scope(LandedTitle | Province)),
            ("value", CompareValue),
        ]),
    ),
    (LandedTitle | Province, "squared_distance(", CompareValue),
    (Character, "stewardship", CompareValue),
    (
        Character,
        "stewardship_diff",
        Block(&[
            ("target", Scope(Character)),
            ("value", CompareValue),
            ("?abs", Boolean),
        ]),
    ),
    (Character, "stewardship_for_portrait", CompareValue),
    (StoryCycle, "story_type", Item(Item::Story)),
    (Character, "stress", CompareValue),
    (Character, "stress_level", CompareValue),
    (Character, "strife_opinion", CompareValue),
    (Character, "sub_realm_size", CompareValue),
    (None, "switch", Special), // TODO
    (
        LandedTitle,
        "target_is_de_facto_liege_or_above",
        Scope(LandedTitle),
    ),
    (
        LandedTitle,
        "target_is_de_jure_liege_or_above",
        Scope(LandedTitle),
    ),
    (Character, "target_is_liege_or_above", Scope(Character)),
    (
        Character,
        "target_is_same_character_or_above",
        Scope(Character),
    ),
    (Character, "target_is_vassal_or_below", Scope(Character)),
    (Character, "target_weight", CompareValue),
    (Province, "terrain", Item(Item::Terrain)),
    (LandedTitle, "tier", CompareValue), // TODO: advice if this is compared to a bare number
    (
        Character,
        "tier_difference",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    // TODO: warn if more than one of days, months, years
    // TODO: check if "weeks" works in these
    (
        Character,
        "time_after_diarch_designated",
        Block(&[
            ("?days", CompareValue),
            ("?months", CompareValue),
            ("?years", CompareValue),
        ]),
    ),
    // TODO: "weeks" is used in vanilla but is not documented. Verify.
    (
        Character,
        "time_in_prison",
        Block(&[
            ("?days", CompareValue),
            ("?weeks", CompareValue),
            ("?months", CompareValue),
            ("?years", CompareValue),
        ]),
    ),
    // TODO: "weeks" is used in vanilla but is not documented. Verify.
    (
        Character,
        "time_in_prison_type",
        Block(&[
            ("?days", CompareValue),
            ("?weeks", CompareValue),
            ("?months", CompareValue),
            ("?years", CompareValue),
        ]),
    ),
    (None, "time_of_year", Special),
    (
        Character,
        "time_since_death",
        Block(&[
            ("?days", CompareValue),
            ("?months", CompareValue),
            ("?years", CompareValue),
        ]),
    ),
    (
        Character,
        "time_to_hook_expiry",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (
        LandedTitle,
        "title_create_faction_type_chance",
        Block(&[
            ("type", Item(Item::Faction)),
            ("target", Scope(Character)),
            ("value", CompareValue),
        ]),
    ),
    (LandedTitle, "title_held_years", CompareValueWarnEq),
    (LandedTitle, "title_is_a_faction_member", Boolean),
    (
        LandedTitle,
        "title_join_faction_chance",
        Block(&[("target", Scope(Faction)), ("value", CompareValue)]),
    ),
    (
        LandedTitle,
        "title_will_leave_sub_realm_on_succession",
        Scope(Character),
    ),
    (Army, "total_army_damage", CompareValue),
    (Army, "total_army_pursuit", CompareValue),
    (Army, "total_army_screen", CompareValue),
    (Army, "total_army_siege_value", CompareValue),
    (Army, "total_army_toughness", CompareValue),
    (
        Character,
        "trait_compatibility",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (Faith, "trait_is_sin", ScopeOrItem(Trait, Item::Trait)),
    (Faith, "trait_is_virtue", ScopeOrItem(Trait, Item::Trait)),
    (
        Province,
        "travel_danger_type",
        Block(&[
            ("travel_plan", Scope(TravelPlan)),
            ("?type", Item(Item::DangerType)),
            ("?terrain", Item(Item::Terrain)),
        ]),
    ),
    (
        Province,
        "travel_danger_value",
        Block(&[("target", Scope(TravelPlan)), ("value", CompareValue)]),
    ),
    (Character, "travel_leader_cost", CompareValue),
    (Character, "travel_leader_safety", CompareValue),
    (Character, "travel_leader_speed", CompareValue),
    (TravelPlan, "travel_safety", CompareValue),
    (TravelPlan, "travel_speed", CompareValue),
    (None, "trigger_else", Control),
    (None, "trigger_else_if", Control),
    (None, "trigger_if", Control),
    (CombatSide, "troops_ratio", CompareValue),
    (
        AccoladeType,
        "type_has_accolade_category",
        Item(Item::AccoladeCategory),
    ),
    (Character, "tyranny", CompareValue),
    (War, "using_cb", Item(Item::CasusBelli)),
    (
        None,
        "variable_list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (
        Character,
        "vassal_contract_has_flag",
        Item(Item::VassalContractFlag),
    ),
    (
        Character,
        "vassal_contract_has_modifiable_obligations",
        Boolean,
    ),
    (
        Character,
        "vassal_contract_is_blocked_from_modification",
        Boolean,
    ),
    (
        Character,
        "vassal_contract_obligation_level_can_be_decreased",
        Item(Item::VassalObligation),
    ),
    (
        Character,
        "vassal_contract_obligation_level_can_be_increased",
        Item(Item::VassalObligation),
    ),
    (
        Character,
        "vassal_contract_obligation_level_score(",
        CompareValue,
    ),
    (Character, "vassal_count", CompareValue),
    (Character, "vassal_limit", CompareValue),
    (Character, "vassal_limit_available", CompareValue),
    (Character, "vassal_limit_percentage", CompareValue),
    (Character, "war_chest_gold", CompareValueWarnEq),
    (Character, "war_chest_gold_maximum", CompareValueWarnEq),
    (
        War,
        "war_contribution",
        Block(&[("target", Scope(Character)), ("value", CompareValue)]),
    ),
    (War, "war_days", CompareValueWarnEq),
    (Combat, "warscore_value", CompareValue),
    (TravelPlan, "was_activity_completed", Boolean),
    (TravelPlan, "was_activity_invalidated", Boolean),
    (War, "was_called", Scope(Character)),
    (None, "weighted_calc_true_if", Special),
    (Character, "year_of_birth", CompareValue),
    (Character, "yearly_character_balance", CompareValueWarnEq),
    (Character, "yearly_character_expenses", CompareValueWarnEq),
    (Character, "yearly_character_income", CompareValueWarnEq),
    (Character, "years_as_diarch", CompareValueWarnEq),
    (Character, "years_as_ruler", CompareValueWarnEq),
    (None, "years_from_game_start", CompareValueWarnEq),
    (Character, "years_in_diarchy", CompareValueWarnEq),
    (
        Character,
        "yields_alliance",
        Block(&[
            ("candidate", Scope(Character)),
            ("target", Scope(Character)),
            ("target_candidate", Scope(Character)),
        ]),
    ),
];
