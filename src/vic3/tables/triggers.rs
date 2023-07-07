use RawTrigger::*;

use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;

/// A version of Trigger that uses u64 to represent Scopes values, because
/// constructing bitfield types in const values is not allowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RawTrigger {
    /// trigger = no or trigger = yes
    Boolean,
    /// can be a script value
    CompareValue,
    /// can be a script value; warn if =
    CompareValueWarnEq,
    /// can be a script value; no < or >
    SetValue,
    /// value must be a valid date
    CompareDate,
    /// value is a level from `very_low` to `very_high`
    CompareLevel,
    /// value is a stance from `strongly_disapprove` to `strongly_approve`
    CompareStance,
    /// trigger is compared to a scope object
    Scope(u64),
    /// trigger is compared to a scope object which may be `this`
    ScopeOkThis(u64),
    /// value is chosen from an item type
    Item(Item),
    ScopeOrItem(u64, Item),
    /// value is chosen from a list given here
    Choice(&'static [&'static str]),
    /// For Block, if a field name in the array starts with ? it means that field is optional
    /// trigger takes a block with these fields
    Block(&'static [(&'static str, RawTrigger)]),
    /// trigger takes a block with these fields
    ScopeOrBlock(u64, &'static [(&'static str, RawTrigger)]),
    /// trigger takes a block with these fields
    ItemOrBlock(Item, &'static [(&'static str, RawTrigger)]),
    /// can be part of a scope chain but also a standalone trigger
    CompareValueOrBlock(&'static [(&'static str, RawTrigger)]),
    /// trigger takes a block of values of this scope type
    ScopeList(u64),
    /// trigger takes a block comparing two scope objects
    ScopeCompare(u64),
    /// this is for inside a Block, where a key is compared to a scope object
    CompareToScope(u64),

    /// this key opens another trigger block
    Control,
    /// this has specific code for validation
    Special,

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
    CompareLevel,
    CompareStance,
    Scope(Scopes),
    ScopeOkThis(Scopes),
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
            RawTrigger::CompareLevel => Trigger::CompareLevel,
            RawTrigger::CompareStance => Trigger::CompareStance,
            RawTrigger::Scope(s) => Trigger::Scope(Scopes::from_bits_truncate(*s)),
            RawTrigger::ScopeOkThis(s) => Trigger::ScopeOkThis(Scopes::from_bits_truncate(*s)),
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
    std::option::Option::None
}

pub fn trigger_comparevalue(name: &Token, data: &Everything) -> Option<Scopes> {
    match scope_trigger(name, data) {
        Some((
            s,
            Trigger::CompareValue
            | Trigger::CompareValueWarnEq
            | Trigger::CompareDate
            | Trigger::SetValue
            | Trigger::CompareValueOrBlock(_),
        )) => Some(s),
        _ => std::option::Option::None,
    }
}

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `triggers.log` from the game data dumps
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(u64, &str, RawTrigger)] = &[
    (None, "active_lens", UncheckedValue),
    (None, "active_lens_option", UncheckedValue),
    // TODO: warn if this is in an any_ iterator and not at the end
    (ALL_BUT_NONE, "add_to_temporary_list", Special),
    (Character, "age", CompareValue),
    (Country, "aggressive_diplomatic_plays_permitted", Boolean),
    (None, "all_false", Control),
    (None, "always", Boolean),
    (None, "and", Control),
    (None, "any_false", Control),
    (Country, "approaching_bureaucracy_shortage", Boolean),
    (State, "arable_land", CompareValue),
    (Country, "arable_land_country", CompareValue),
    (Country, "army_reserves", CompareValue),
    (
        None,
        "assert_if",
        Block(&[("limit", Control), ("?text", UncheckedValue)]),
    ),
    (None, "assert_read", UncheckedValue),
    (Country, "authority", CompareValue),
    (State, "available_jobs", CompareValue),
    (Country, "average_country_infrastructure", CompareValue),
    (
        Country,
        "average_incorporated_country_infrastructure",
        CompareValue,
    ),
    (
        Battle,
        "battle_side_pm_usage",
        Block(&[
            ("target", Scope(Country)),
            ("production_method", Item(Item::ProductionMethod)),
            ("value", CompareValue),
        ]),
    ),
    (Building, "building_has_goods_shortage", Boolean),
    (Country, "bureaucracy", CompareValue),
    (None, "calc_true_if", Control),
    (
        State,
        "can_activate_production_method",
        Block(&[
            ("building_type", Item(Item::BuildingType)),
            ("production_method", Item(Item::ProductionMethod)),
        ]),
    ),
    (
        Country,
        "can_afford_diplomatic_action",
        Block(&[("target", Scope(Country)), ("type", UncheckedValue)]),
    ),
    (Character, "can_agitate", Scope(Country)),
    (Law, "can_be_enacted", Boolean),
    (
        Country,
        "can_establish_any_export_route",
        ScopeOrItem(Goods, Item::Goods),
    ),
    (
        Country,
        "can_establish_any_import_route",
        ScopeOrItem(Goods, Item::Goods),
    ),
    (Country, "can_form_nation", Item(Item::Country)),
    (
        Country,
        "can_have_as_subject",
        Block(&[("who", Scope(Country)), ("type", UncheckedValue)]),
    ),
    (Country, "can_have_subjects", Boolean),
    (Building, "can_queue_building_levels", CompareValue),
    (Country, "can_research", Item(Item::Technology)),
    (None, "can_start_tutorial_lesson", UncheckedValue), // TODO
    (
        Country,
        "can_take_on_scaled_debt",
        Block(&[("who", Scope(Country)), ("value", CompareValue)]),
    ),
    (Building, "cash_reserves_available", CompareValue),
    (Building, "cash_reserves_ratio", CompareValue),
    (Character, "character_supports_political_movement", Boolean),
    (
        Country | Province | State | StateRegion | StrategicRegion | Theater,
        "check_area",
        UncheckedValue,
    ),
    (CivilWar, "civil_war_progress", CompareValue),
    (Character, "commander_is_available", Boolean),
    (
        Character,
        "commander_pm_usage",
        Block(&[
            ("target", Scope(Country)),
            ("production_method", Item(Item::ProductionMethod)),
            ("value", CompareValue),
        ]),
    ),
    (Character, "commander_rank", CompareValue),
    (Country, "construction_queue_duration", CompareValue),
    (
        Country,
        "construction_queue_government_duration",
        CompareValue,
    ),
    (
        Country,
        "construction_queue_num_queued_government_levels",
        CompareValue,
    ),
    (
        Country,
        "construction_queue_num_queued_levels",
        CompareValue,
    ),
    (
        Country,
        "construction_queue_num_queued_private_levels",
        CompareValue,
    ),
    (Country, "construction_queue_private_duration", CompareValue),
    (
        StateRegion,
        "contains_capital_of",
        ScopeOrItem(Country, Item::Country),
    ),
    (
        CountryDefinition,
        "country_definition_has_culture",
        Scope(Culture),
    ),
    (Country, "country_has_primary_culture", Scope(Culture)),
    (Country, "country_has_state_religion", Scope(Religion)),
    (
        Country,
        "country_or_subject_owns_entire_state_region",
        Item(Item::StateRegion),
    ),
    (
        Country,
        "country_pm_usage",
        Block(&[
            ("target", Scope(Country)),
            ("production_method", Item(Item::ProductionMethod)),
            ("value", CompareValue),
        ]),
    ),
    (Country, "country_rank", CompareValue),
    (Country, "country_tier", CompareValue),
    (Pop, "culture_accepted", Boolean),
    (Culture, "culture_is_discriminated_in", Scope(Country)),
    (
        Country,
        "culture_percent_country",
        Block(&[("target", Scope(Culture)), ("value", CompareValue)]),
    ),
    (
        State,
        "culture_percent_state",
        Block(&[("target", Scope(Culture)), ("value", CompareValue)]),
    ),
    (Culture, "culture_secession_progress", CompareValue),
    (None, "current_tooltip_depth", CompareValue),
    (None, "custom_description", Control),
    (None, "custom_tooltip", Special),
    (None, "debug_log", UncheckedValue),
    (None, "debug_log_details", UncheckedValue),
    (State, "devastation", CompareValue),
    (
        DiplomaticPlay,
        "diplomatic_play_pm_usage",
        Block(&[
            ("target", Scope(Country)),
            ("production_method", Item(Item::ProductionMethod)),
            ("value", CompareValue),
        ]),
    ),
    (Building, "earnings", CompareValue),
    (Party, "election_momentum", CompareValue),
    (Country, "empty_agitator_slots", CompareValue),
    (Country, "enacting_any_law", Boolean),
    (Country, "enactment_chance", CompareValue),
    (
        Country,
        "enactment_chance_without_enactment_modifier",
        CompareValue,
    ),
    (Country, "enactment_phase", CompareValue),
    (Country, "enactment_setback_count", CompareValue),
    (None, "error_check", Special),
    (DiplomaticPlay, "escalation", CompareValue),
    (None, "exists", Special),
    (Country, "expanding_institution", UncheckedValue),
    (Character, "experience_level", CompareValue),
    (State, "free_arable_land", CompareValue),
    (
        Front,
        "front_side_pm_usage",
        Block(&[
            ("target", Scope(Country)),
            ("production_method", Item(Item::ProductionMethod)),
            ("value", CompareValue),
        ]),
    ),
    (None, "game_date", CompareDate),
    (Country, "global_country_ranking", CompareValue),
    (
        None,
        "global_variable_list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (Country, "gold_reserves", CompareValue),
    (Country, "gold_reserves_limit", CompareValue),
    (Country, "government_legitimacy", CompareValue),
    (Country, "government_transfer_of_power", CompareValue),
    (Country, "government_wage_level", CompareLevel),
    (Country, "government_wage_level_value", CompareValue),
    (
        Market | State,
        "has_active_building",
        Item(Item::BuildingType),
    ),
    (Country, "has_active_peace_deal", Boolean),
    (
        Building,
        "has_active_production_method",
        Item(Item::ProductionMethod),
    ),
    (Country, "has_any_secessionists_broken_out", Boolean),
    (Country, "has_any_secessionists_growing", Boolean),
    (Country, "has_any_secessionists_possible", Boolean),
    (State, "has_assimilating_pops", Boolean),
    (
        Country,
        "has_attitude",
        Block(&[("who", Scope(Country)), ("attitude", Item(Item::Attitude))]),
    ),
    (BattleSide, "has_battle_condition", UncheckedValue),
    (
        Country | Market | State | StateRegion,
        "has_building",
        Item(Item::BuildingType),
    ),
    (Country, "has_claim", Scope(State | StateRegion)),
    (State, "has_claim_by", Scope(Country)),
    (Character, "has_commander_order", UncheckedValue),
    (Country, "has_completed_subgoal", UncheckedValue),
    (Country, "has_consumption_tax", Scope(Goods)),
    (State, "has_converting_pops", Boolean),
    (Country, "has_convoys_being_sunk", Boolean),
    (None, "has_cosmetic_dlc", UncheckedValue),
    (Culture, "has_cultural_obsession", Item(Item::Goods)),
    (Character, "has_culture", Scope(Character | Culture)),
    (Culture, "has_culture_graphics", UncheckedValue),
    (Country, "has_decreasing_interests", Boolean),
    (State, "has_decree", UncheckedValue),
    (
        Country,
        "has_diplomatic_pact",
        Block(&[
            ("who", Scope(Country)),
            ("type", UncheckedValue),
            ("?is_initiator", Boolean),
        ]),
    ),
    (StrategicRegion, "has_diplomatic_play", Boolean),
    (Country, "has_diplomatic_relevance", Scope(Country)),
    (Country, "has_diplomats_expelled", Scope(Country)),
    (
        Culture | Religion,
        "has_discrimination_trait",
        UncheckedValue,
    ),
    (None, "has_dlc_feature", Item(Item::DlcFeature)),
    (
        Building,
        "has_employee_slots_filled",
        Block(&[("pop_type", UncheckedValue), ("percent", CompareValue)]),
    ),
    (Country, "has_export_priority_tariffs", UncheckedValue),
    (Building, "has_failed_hires", Boolean),
    (Country, "has_free_government_reform", Boolean),
    (None, "has_game_rule", UncheckedValue),
    (None, "has_game_started", Boolean),
    (None, "has_gameplay_dlc", UncheckedValue),
    (Country, "has_global_highest_gdp", Boolean),
    (Country, "has_global_highest_innovation", Boolean),
    (None, "has_global_variable", UncheckedValue),
    (None, "has_global_variable_list", UncheckedValue),
    (Country, "has_government_clout", CompareValue),
    (Country, "has_government_type", UncheckedValue),
    (Country, "has_healthy_economy", Boolean),
    (Character, "has_high_attrition", Boolean),
    (Culture, "has_homeland", Scope(State | StateRegion)),
    (Character | InterestGroup, "has_ideology", UncheckedValue),
    (Country, "has_import_priority_tariffs", UncheckedValue),
    (Country, "has_institution", UncheckedValue),
    (Country, "has_insurrectionary_interest_groups", Boolean),
    (
        Country,
        "has_interest_marker_in_region",
        ScopeOrItem(StrategicRegion, Item::StrategicRegion),
    ),
    (Country, "has_journal_entry", UncheckedValue),
    (Province, "has_label", UncheckedValue),
    (Country, "has_law", UncheckedValue),
    (None, "has_local_variable", UncheckedValue),
    (None, "has_local_variable_list", UncheckedValue),
    (None, "has_map_interaction", UncheckedValue),
    (
        None,
        "has_map_interaction_diplomatic_action",
        UncheckedValue,
    ),
    (None, "has_map_interaction_export_goods", UncheckedValue),
    (None, "has_map_interaction_import_goods", UncheckedValue),
    (
        Country
            | Building
            | Character
            | Front
            | Institution
            | InterestGroup
            | Journalentry
            | PoliticalMovement
            | State,
        "has_modifier",
        Item(Item::Modifier),
    ),
    (Country, "has_no_priority_tariffs", UncheckedValue),
    (Country, "has_objective", UncheckedValue),
    (Pop, "has_ongoing_assimilation", Boolean),
    (Pop, "has_ongoing_conversion", Boolean),
    (Country, "has_overlapping_interests", Scope(Country)),
    (InterestGroup, "has_party", Boolean),
    (Party, "has_party_member", Scope(InterestGroup)),
    (DiplomaticPlay, "has_play_goal", UncheckedValue),
    (Pop, "has_pop_culture", Item(Item::Culture)),
    (Pop, "has_pop_religion", Item(Item::Religion)),
    (Country | Market | State, "has_port", Boolean),
    (Country, "has_possible_decisions", Boolean),
    (State, "has_potential_resource", UncheckedValue),
    (Country, "has_potential_to_form_country", UncheckedValue),
    (None, "has_reached_end_date", Boolean),
    (Character, "has_religion", Scope(Religion)),
    (Country, "has_researchable_technology", Boolean),
    (Country, "has_revolution", Boolean),
    (Character, "has_role", UncheckedValue),
    (Country, "has_ruling_interest_group", UncheckedValue),
    (Country, "has_ruling_interest_group_count", CompareValue),
    (
        Country,
        "has_secret_goal",
        Block(&[("who", Scope(Country)), ("secret_goal", UncheckedValue)]),
    ),
    (
        Country,
        "has_state_in_state_region",
        Item(Item::StateRegion),
    ),
    (Pop, "has_state_religion", Boolean),
    (State, "has_state_trait", UncheckedValue),
    (Country, "has_strategic_adjacency", Scope(State | Country)),
    (
        Country,
        "has_strategic_land_adjacency",
        Scope(State | Country),
    ),
    (Country, "has_strategy", UncheckedValue),
    (Country, "has_subject_relation_with", Scope(Country)),
    (
        Country,
        "has_sufficient_construction_capacity_for_investment",
        Boolean,
    ),
    (
        Country,
        "has_technology_progress",
        Block(&[("technology", UncheckedValue), ("progress", CompareValue)]),
    ),
    (Country, "has_technology_researched", UncheckedValue),
    (Character, "has_template", UncheckedValue),
    (Province, "has_terrain", UncheckedValue),
    (Character, "has_trait", UncheckedValue),
    (Country, "has_treaty_port_in_country", Scope(Country)),
    (Country, "has_truce_with", Scope(Country)),
    (None, "has_unification_candidate", UncheckedValue),
    (None, "has_variable", UncheckedValue),
    (None, "has_variable_list", UncheckedValue),
    (
        War,
        "has_war_exhaustion",
        Block(&[("target", Scope(Country)), ("value", CompareValue)]),
    ),
    (War, "has_war_goal", UncheckedValue),
    (War, "has_war_support", CompareValue),
    (Country, "has_war_with", Scope(Country)),
    (Country, "has_wasted_construction", Boolean),
    (None, "hidden_trigger", Control),
    (Country, "highest_secession_progress", CompareValue),
    (InterestGroup, "ig_approval", CompareValue),
    (InterestGroup, "ig_clout", CompareValue),
    (InterestGroup, "ig_government_power_share", CompareValue),
    (
        State,
        "ig_state_pol_strength_share",
        Block(&[("target", Scope(InterestGroup)), ("value", CompareValue)]),
    ),
    (Country, "in_default", Boolean),
    (Country, "in_election_campaign", Boolean),
    (State, "incorporation_progress", CompareValue),
    (Country, "influence", CompareValue),
    (State | StateRegion, "infrastructure", CompareValue),
    (State | StateRegion, "infrastructure_usage", CompareValue),
    (DiplomaticPlay, "initiator_is", Scope(Country)),
    (
        Country,
        "institution_investment_level",
        Block(&[("institution", UncheckedValue), ("value", CompareValue)]),
    ),
    (InterestGroup, "interest_group_population", CompareValue),
    (
        InterestGroup,
        "interest_group_population_percentage",
        CompareValue,
    ),
    (
        InterestGroup,
        "interest_group_supports_political_movement",
        Boolean,
    ),
    (Country, "investment_pool", CompareValue),
    (Country, "investment_pool_gross_income", CompareValue),
    (Country, "investment_pool_net_income", CompareValue),
    (Country, "is_adjacent", Scope(Country)),
    (Character, "is_advancing_on_front", Scope(Front)),
    (Country, "is_ai", Boolean),
    (Country, "is_at_war", Boolean),
    (Character, "is_attacker_in_battle", Boolean),
    (Country, "is_banning_goods", Scope(Goods)),
    (InterestGroup, "is_being_bolstered", Boolean),
    (InterestGroup, "is_being_suppressed", Boolean),
    (Building, "is_buildable", Boolean),
    (Building, "is_building_group", UncheckedValue),
    (Building, "is_building_type", Item(Item::BuildingType)),
    (
        None,
        "is_building_type_expanded",
        ScopeOrItem(BuildingType, Item::BuildingType),
    ),
    (Character, "is_busy", Boolean),
    (State, "is_capital", Boolean),
    (Character, "is_character_alive", Boolean),
    (
        CivilWar,
        "is_civil_war_type",
        Choice(&["revolution", "secession"]),
    ),
    (State, "is_coastal", Boolean),
    (Country, "is_construction_paused", Boolean),
    (MarketGoods, "is_consumed_by_government_buildings", Boolean),
    (MarketGoods, "is_consumed_by_military_buildings", Boolean),
    (Country, "is_country_alive", Boolean),
    (Country, "is_country_type", UncheckedValue),
    (Character, "is_defender_in_battle", Boolean),
    (DiplomaticPact, "is_diplomatic_action_type", UncheckedValue),
    (DiplomaticPact, "is_diplomatic_in_danger", Boolean),
    (Country, "is_diplomatic_play_committed_participant", Boolean),
    (Country, "is_diplomatic_play_enemy_of", Scope(Country)),
    (Country, "is_diplomatic_play_initiator", Boolean),
    (Country, "is_diplomatic_play_target", Boolean),
    (DiplomaticPlay, "is_diplomatic_play_type", UncheckedValue),
    (Country, "is_diplomatic_play_undecided_participant", Boolean),
    (Country, "is_direct_subject_of", Scope(Country)),
    (Pop, "is_employed", Boolean),
    (Country, "is_enacting_law", UncheckedValue),
    (Country, "is_expanding_institution", Boolean),
    (Character, "is_female", Boolean),
    (None, "is_game_paused", Boolean),
    (None, "is_gamestate_tutorial_active", Boolean),
    (Journalentry, "is_goal_complete", Boolean),
    (Building, "is_government_funded", Boolean),
    (Character, "is_heir", Boolean),
    (Character, "is_historical", Boolean),
    (Country, "is_home_country_for", Scope(Country)),
    (
        StateRegion,
        "is_homeland",
        ScopeOrItem(Culture, Item::Culture),
    ),
    (State, "is_homeland_of_country_cultures", Scope(Country)),
    (Character, "is_in_battle", Boolean),
    (Country, "is_in_customs_union", Boolean),
    (Character, "is_in_exile_pool", Boolean),
    (InterestGroup, "is_in_government", Boolean),
    (None, "is_in_list", Special),
    (State, "is_in_revolt", Boolean),
    (Character, "is_in_void", Boolean),
    (Country, "is_in_war_together", Scope(Country)),
    (State, "is_incorporated", Boolean),
    (InterestGroup, "is_insurrectionary", Boolean),
    (InterestMarker, "is_interest_active", Boolean),
    (
        Character | InterestGroup,
        "is_interest_group_type",
        UncheckedValue,
    ),
    (State, "is_isolated_from_market", Boolean),
    (Country, "is_junior_in_customs_union", Boolean),
    (Theater, "is_land_theater", Boolean),
    (State, "is_largest_state_in_region", Boolean),
    (
        None,
        "is_lens_open",
        Block(&[("lens", UncheckedValue), ("tab_name", UncheckedValue)]),
    ),
    (Country, "is_local_player", Boolean),
    (Country, "is_losing_power_rank", Boolean),
    (InterestGroup, "is_marginal", Boolean),
    (State, "is_mass_migration_target", Boolean),
    (InterestGroup, "is_member_of_party", Scope(Party)),
    (Character, "is_mobilized", Boolean),
    (Character, "is_monarch", Boolean),
    (None, "is_objective_completed", Boolean),
    (Character, "is_on_front", Boolean),
    (Country, "is_owed_obligation_by", Scope(Country)),
    (
        None,
        "is_panel_open",
        Block(&[
            ("?target", UncheckedValue),
            ("?panel_name", UncheckedValue),
            ("tab_name", UncheckedValue),
        ]),
    ),
    (Party, "is_party", Scope(Party)),
    (Party, "is_party_type", UncheckedValue),
    (Country, "is_player", Boolean),
    (
        PoliticalMovement,
        "is_political_movement_type",
        UncheckedValue,
    ),
    (Pop, "is_pop_type", UncheckedValue),
    (None, "is_popup_open", UncheckedValue),
    (InterestGroup, "is_powerful", Boolean),
    (Culture, "is_primary_culture_of", Scope(Country)),
    (
        State,
        "is_production_method_active",
        Block(&[
            ("building_type", Item(Item::BuildingType)),
            ("production_method", UncheckedValue),
        ]),
    ),
    (Journalentry, "is_progressing", Boolean),
    (Province, "is_province_land", Boolean),
    (Character, "is_repairing", Boolean),
    (Country, "is_researching_technology", UncheckedValue),
    (
        Country,
        "is_researching_technology_category",
        UncheckedValue,
    ),
    (Country | InterestGroup, "is_revolutionary", Boolean),
    (PoliticalMovement, "is_revolutionary_movement", Boolean),
    (None, "is_rightclick_menu_open", Boolean),
    (Character, "is_ruler", Boolean),
    (
        InterestGroup,
        "is_same_interest_group_type",
        Scope(InterestGroup),
    ),
    (LawType, "is_same_law_group_as", Scope(LawType)),
    (Party, "is_same_party_type", Scope(Party)),
    (State, "is_sea_adjacent", Boolean),
    (Country, "is_secessionist", Boolean),
    (None, "is_set", Special),
    (State, "is_slave_state", Boolean),
    (State, "is_split_state", Boolean),
    (StateRegion, "is_state_region_land", Boolean),
    (Religion, "is_state_religion", Scope(Country)),
    (State, "is_strategic_objective", Scope(Country)),
    (InterestGroup, "is_strongest_ig_in_government", Boolean),
    (Country, "is_subject", Boolean),
    (Country, "is_subject_of", Scope(Country)),
    (Country, "is_subject_type", UncheckedValue),
    (Building, "is_subsidized", Boolean),
    (Building, "is_subsistence_building", Boolean),
    (
        Country,
        "is_supporting_unification_candidate",
        Block(&[
            ("who", Scope(Country)),
            ("country_formation", UncheckedValue),
        ]),
    ),
    (
        None,
        "is_target_in_global_variable_list",
        Block(&[
            ("name", UncheckedValue),
            ("*target", ScopeOkThis(ALL_BUT_NONE)),
        ]),
    ),
    (
        None,
        "is_target_in_local_variable_list",
        Block(&[
            ("name", UncheckedValue),
            ("*target", ScopeOkThis(ALL_BUT_NONE)),
        ]),
    ),
    (
        None,
        "is_target_in_variable_list",
        Block(&[
            ("name", UncheckedValue),
            ("*target", ScopeOkThis(ALL_BUT_NONE)),
        ]),
    ),
    (State, "is_target_of_wargoal", Scope(Country)),
    (Country, "is_taxing_goods", UncheckedValue),
    (TradeRoute, "is_trade_route_active", Boolean),
    (TradeRoute, "is_trade_route_productive", Boolean),
    (Goods | MarketGoods, "is_tradeable", Boolean),
    (Character, "is_traveling", Boolean),
    (State, "is_treaty_port", Boolean),
    (None, "is_tutorial_active", Boolean),
    (None, "is_tutorial_lesson_active", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_chain_completed", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_completed", UncheckedValue), // TODO
    (None, "is_tutorial_lesson_step_completed", UncheckedValue), // TODO
    (State, "is_under_colonization", Boolean),
    (Building, "is_under_construction", Boolean),
    (Country, "is_unification_candidate", UncheckedValue),
    (Country, "is_violating_sovereignty_of", Scope(Country)),
    (Front, "is_vulnerable_front", Scope(Country)),
    (DiplomaticPlay, "is_war", Boolean),
    (War, "is_war_participant", Scope(Country)),
    (War, "is_warleader", Scope(Country)),
    (Country, "isolated_states", CompareValue),
    (Law, "law_approved_by", Scope(InterestGroup)),
    (
        Character | InterestGroup,
        "law_stance",
        Block(&[("law", Scope(LawType)), ("value", CompareStance)]),
    ),
    (Country, "leading_producer_of", Scope(Goods)),
    (Country, "leads_customs_union", Boolean),
    (
        None,
        "list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (Country | Pop | State, "literacy_rate", CompareValue),
    (
        None,
        "local_variable_list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (
        Country | State,
        "loyalist_fraction",
        Block(&[
            ("value", CompareValue),
            ("?pop_type", UncheckedValue),
            ("?strata", UncheckedValue),
            ("?culture", ScopeOrItem(Culture, Item::Culture)),
            ("?religion", ScopeOrItem(Religion, Item::Religion)),
        ]),
    ),
    (State, "loyalty", CompareValue),
    (State, "market_access", CompareValue),
    (MarketGoods, "market_goods_buy_orders", CompareValue),
    (MarketGoods, "market_goods_cheaper", CompareValue),
    (MarketGoods, "market_goods_consumption", CompareValue),
    (MarketGoods, "market_goods_delta", CompareValue),
    (MarketGoods, "market_goods_exports", CompareValue),
    (MarketGoods, "market_goods_has_goods_shortage", Boolean),
    (MarketGoods, "market_goods_imports", CompareValue),
    (MarketGoods, "market_goods_pricier", CompareValue),
    (MarketGoods, "market_goods_production", CompareValue),
    (MarketGoods, "market_goods_sell_orders", CompareValue),
    (MarketGoods, "market_goods_shortage_ratio", CompareValue),
    (Market, "market_has_goods_shortage", Boolean),
    (Country, "max_num_declared_interested", CompareValue),
    (Country, "military_wage_level", CompareLevel),
    (Country, "military_wage_level_value", CompareValue),
    (Character, "mobilization_cost", CompareValue),
    (InterestGroup, "most_powerful_strata", UncheckedValue),
    (
        State,
        "most_prominent_revolting_interest_group",
        UncheckedValue,
    ),
    (None, "nand", Control),
    (Country, "navy_reserves", CompareValue),
    (None, "nor", Control),
    (None, "not", Control),
    (War, "num_casualties", CompareValue),
    (
        War,
        "num_country_casualties",
        Block(&[("target", Scope(Country)), ("value", CompareValue)]),
    ),
    (
        War,
        "num_country_dead",
        Block(&[("target", Scope(Country)), ("value", CompareValue)]),
    ),
    (
        War,
        "num_country_wounded",
        Block(&[("target", Scope(Country)), ("value", CompareValue)]),
    ),
    (War, "num_dead", CompareValue),
    (Country, "num_declared_interests", CompareValue),
    (Country, "num_taxed_goods", CompareValue),
    (War, "num_wounded", CompareValue),
    (Country, "number_of_possible_decisions", CompareValue),
    (Building, "occupancy", CompareValue),
    (None, "or", Control),
    (Country, "owes_obligation_to", Scope(Country)),
    (Country, "owns_entire_state_region", UncheckedValue),
    (Country, "owns_treaty_port_in", UncheckedValue),
    (
        PoliticalMovement,
        "political_movement_radicalism",
        CompareValue,
    ),
    (
        PoliticalMovement,
        "political_movement_support",
        CompareValue,
    ),
    (StateRegion, "pollution_amount", CompareValue),
    (State, "pollution_generation", CompareValue),
    (Pop, "pop_employment_building", Scope(Building)),
    (Pop, "pop_employment_building_group", Scope(Building)),
    (Pop, "pop_has_primary_culture", Boolean),
    (Pop, "pop_is_discriminated", Boolean),
    (
        Pop,
        "pop_type_percent_country",
        Block(&[("pop_type", UncheckedValue), ("percent", CompareValue)]),
    ),
    (InterestGroup, "prefers_law", UncheckedValue),
    (Country, "prestige", CompareValue),
    (Country, "produced_authority", CompareValue),
    (Country, "produced_bureaucracy", CompareValue),
    (Country, "produced_influence", CompareValue),
    (Pop, "quality_of_life", CompareValue),
    (
        Country | State,
        "radical_fraction",
        Block(&[
            ("value", CompareValue),
            ("?pop_type", UncheckedValue),
            ("?strata", UncheckedValue),
            ("?culture", ScopeOrItem(Culture, Item::Culture)),
            ("?religion", ScopeOrItem(Religion, Item::Religion)),
        ]),
    ),
    (State, "relative_infrastructure", CompareValue),
    (Pop, "religion_accepted", Boolean),
    (
        Country,
        "pop_type_percent_country",
        Block(&[("target", Scope(Religion)), ("value", CompareValue)]),
    ),
    (
        State,
        "pop_type_percent_state",
        Block(&[("target", Scope(Religion)), ("value", CompareValue)]),
    ),
    (
        StateRegion,
        "remaining_undepleted",
        Block(&[("type", UncheckedValue), ("amount", CompareValue)]),
    ),
    (Country, "ruler_can_have_command", Boolean),
    (ALL_BUT_NONE, "save_temporary_scope_as", Special),
    (None, "save_temporary_scope_value_as", Special),
    (Country, "scaled_debt", CompareValue),
    (
        Culture,
        "shares_heritage_and_other_trait_with_any_primary_culture",
        Scope(Country),
    ),
    (
        Culture,
        "shares_heritage_trait_with_any_primary_culture",
        Scope(Country),
    ),
    (
        Religion,
        "shares_heritage_trait_with_state_religion",
        Scope(Country),
    ),
    (
        Culture,
        "shares_non_heritage_trait_with_any_primary_culture",
        Scope(Country),
    ),
    (
        Culture,
        "shares_trait_with_any_primary_culture",
        Scope(Country),
    ),
    (Religion, "shares_trait_with_state_religion", Scope(Country)),
    (Country, "should_set_wargoal", Boolean),
    (None, "should_show_nudity", Boolean),
    (Country, "shrinking_institution", UncheckedValue),
    (Pop, "standard_of_living", CompareValue),
    (State, "state_has_goods_shortage", Boolean),
    (State, "state_population", CompareValue),
    (State, "state_unemployment_rate", CompareValue),
    (Pop, "strata", CompareValue),
    (Country, "supply_network_strength", CompareValue),
    (None, "switch", Special),
    (Country, "taking_loans", Boolean),
    (DiplomaticPlay, "target_is", Scope(Country)),
    (State, "tax_capacity", CompareValue),
    (State, "tax_capacity_usage", CompareValue),
    (Country, "tax_level", CompareLevel),
    (Country, "tax_level_value", CompareValue),
    (Country, "total_population", CompareValue),
    (State, "total_urbanization", CompareValue),
    (TradeRoute, "trade_route_needs_convoys_to_grow", Boolean),
    (Character, "trait_value", CompareValue),
    (None, "trigger_else", Control),
    (None, "trigger_else_if", Control),
    (None, "trigger_if", Control),
    (State, "turmoil", CompareValue),
    // docs for all three say "target" instead of "value"
    (
        None,
        "variable_list_size",
        Block(&[("name", UncheckedValue), ("value", CompareValue)]),
    ),
    (War, "war_has_active_peace_deal", Boolean),
    (Character, "was_exiled", Boolean),
    (Country, "was_formed_from", UncheckedValue),
    (Pop, "wealth", CompareValue),
    (Country, "weekly_net_fixed_income", CompareValue),
    (Building, "weekly_profit", CompareValue),
    (None, "weighted_calc_true_if", Special),
    (None, "year", CompareValue),
];
