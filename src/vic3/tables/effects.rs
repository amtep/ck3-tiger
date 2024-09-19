use once_cell::sync::Lazy;

use crate::effect::Effect;
use crate::effect_validation::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::vic3::effect_validation::*;
use crate::vic3::tables::misc::STATE_TYPES;

use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    SCOPE_EFFECT_MAP.get(&*name_lc).copied()
}

/// A hashed version of [`SCOPE_EFFECT`], for quick lookup by effect name.
static SCOPE_EFFECT_MAP: Lazy<TigerHashMap<&'static str, (Scopes, Effect)>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, effect) in SCOPE_EFFECT.iter().copied() {
        hash.insert(s, (from, effect));
    }
    hash
});

// LAST UPDATED VIC3 VERSION 1.7.0
// See `effects.log` from the game data dumps
const SCOPE_EFFECT: &[(Scopes, &str, Effect)] = &[
    (Scopes::InterestGroup, "abandon_revolution", Boolean),
    (Scopes::State, "activate_building", Item(Item::BuildingType)),
    (Scopes::Country, "activate_law", Scope(Scopes::LawType)),
    (
        Scopes::Country.union(Scopes::State),
        "activate_production_method",
        Vb(validate_activate_production_method),
    ),
    (Scopes::StateRegion, "add_arable_land", ScriptValue),
    (Scopes::Country, "add_banned_goods", Scope(Scopes::Goods)),
    (Scopes::Country, "add_change_relations_progress", Vb(validate_country_value)), // Not used in vanilla
    (Scopes::Character, "add_character_role", Item(Item::CharacterRole)),
    (Scopes::CivilWar, "add_civil_war_progress", ScriptValue),
    (Scopes::StateRegion, "add_claim", Scope(Scopes::Country)),
    (Scopes::PowerBloc, "add_cohesion_number", ScriptValue),
    (Scopes::PowerBloc, "add_cohesion_percent", ScriptValue),
    (Scopes::Character, "add_commander_rank", Integer),
    (Scopes::Country, "add_company", Scope(Scopes::CompanyType)),
    (Scopes::State, "add_cultural_community", Scope(Scopes::Culture)),
    (Scopes::Culture, "add_cultural_community_in_state", Scope(Scopes::State)),
    (Scopes::Culture, "add_cultural_obsession", Item(Item::Goods)),
    (
        Scopes::State,
        "add_culture_standard_of_living_modifier",
        Vb(validate_add_culture_sol_modifier),
    ),
    (Scopes::Country, "add_declared_interest", Item(Item::StrategicRegion)),
    (Scopes::StateRegion, "add_devastation", ScriptValue),
    (
        Scopes::DiplomaticPlay,
        "add_diplomatic_play_war_support",
        TargetValue("target", Scopes::Country, "value"),
    ),
    (Scopes::Country, "add_enactment_modifier", Vb(validate_add_enactment_modifier)),
    (Scopes::Country, "add_enactment_phase", ScriptValue),
    (Scopes::Country, "add_enactment_setback", ScriptValue),
    (Scopes::Country, "add_era_researched", Item(Item::TechnologyEra)),
    (Scopes::DiplomaticPlay, "add_escalation", Integer),
    (Scopes::Character, "add_experience", ScriptValue),
    (Scopes::StateRegion, "add_homeland", ScopeOrItem(Scopes::Culture, Item::Culture)),
    (Scopes::InterestGroup, "add_ideology", Item(Item::Ideology)),
    (Scopes::Party, "add_ig_to_party", Scope(Scopes::InterestGroup)),
    (Scopes::DiplomaticPlay, "add_initiator_backers", Vb(validate_addremove_backers)),
    (Scopes::Country, "add_investment_pool", ScriptValue),
    (Scopes::None, "add_journal_entry", Vb(validate_add_journalentry)),
    (Scopes::Country, "add_law_progress", ScriptValue),
    (Scopes::PowerBloc, "add_leverage", TargetValue("target", Scopes::Country, "value")),
    (Scopes::Country, "add_liberty_desire", ScriptValue),
    (Scopes::PoliticalLobby, "add_lobby_member", Scope(Scopes::InterestGroup)),
    (Scopes::Country, "add_loyalists", Vb(validate_add_loyalists)),
    (Scopes::State, "add_loyalists_in_state", Vb(validate_add_loyalists)),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::Institution)
            .union(Scopes::InterestGroup)
            .union(Scopes::JournalEntry)
            .union(Scopes::PoliticalMovement)
            .union(Scopes::PowerBloc)
            .union(Scopes::State),
        "add_modifier",
        Vbv(validate_add_modifier),
    ),
    (Scopes::Party, "add_momentum", ScriptValue),
    (Scopes::NewCombatUnit, "add_morale", ScriptValue),
    (Scopes::MilitaryFormation, "add_organization", ScriptValue), // not used in vanilla
    (Scopes::StateRegion, "add_pollution", ScriptValue),
    (Scopes::Pop, "add_pop_wealth", Vb(validate_pop_wealth)),
    (Scopes::Country, "add_primary_culture", Scope(Scopes::Culture)),
    (Scopes::PowerBloc, "add_principle", Item(Item::Principle)),
    (Scopes::JournalEntry, "add_progress", Vb(validate_progress)),
    (Scopes::Country, "add_radicals", Vb(validate_add_loyalists)),
    (Scopes::State, "add_radicals_in_state", Vb(validate_add_loyalists)),
    (Scopes::Character, "add_random_trait", Choice(&["personality", "skill", "condition"])),
    (
        Scopes::State,
        "add_religion_standard_of_living_modifier",
        Vb(validate_add_religion_sol_modifier),
    ),
    (Scopes::InterestGroup, "add_ruling_interest_group", Boolean),
    (Scopes::StateRegion, "add_state_trait", Item(Item::StateTrait)),
    (Scopes::DiplomaticPlay, "add_target_backers", Vb(validate_addremove_backers)),
    (Scopes::Country, "add_taxed_goods", Scope(Scopes::Goods)),
    (Scopes::Country, "add_technology_progress", Vb(validate_add_technology_progress)),
    (Scopes::Country, "add_technology_researched", Item(Item::Technology)),
    (Scopes::None, "add_to_global_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::all_but_none(), "add_to_list", Vv(validate_add_to_list)),
    (Scopes::None, "add_to_local_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::all_but_none(), "add_to_temporary_list", Vv(validate_add_to_list)),
    (Scopes::None, "add_to_variable_list", Vb(validate_add_to_variable_list)),
    (Scopes::Character, "add_trait", Item(Item::CharacterTrait)),
    (Scopes::Country, "add_treasury", ScriptValue),
    (Scopes::War, "add_war_exhaustion", TargetValue("target", Scopes::Country, "value")),
    (Scopes::DiplomaticPlay, "add_war_goal", Vb(validate_add_war_goal)),
    (Scopes::War, "add_war_war_support", TargetValue("target", Scopes::Country, "value")),
    (Scopes::Country, "annex", Scope(Scopes::Country)),
    (Scopes::Country, "annex_as_civil_war", Scope(Scopes::Country)),
    (Scopes::Country, "annex_with_incorporation", Scope(Scopes::Country)),
    (Scopes::None, "assert_if", Unchecked),
    (Scopes::None, "assert_read", Unchecked),
    (Scopes::Country, "call_election", Vb(validate_call_election)),
    (Scopes::Country, "cancel_enactment", Yes),
    // Documentation says scope is None but describes scope as Law
    (Scopes::Law, "cancel_imposition", Yes),
    (Scopes::PoliticalLobby, "change_appeasement", Vb(validate_change_appeasement)),
    (Scopes::Character, "change_character_culture", Scope(Scopes::Culture)),
    (Scopes::Character, "change_character_religion", Scope(Scopes::Religion)),
    (Scopes::None, "change_global_variable", Vb(validate_change_variable)),
    (Scopes::None, "change_infamy", ScriptValue),
    (
        Scopes::Country,
        "change_institution_investment_level",
        Vb(validate_change_institution_investment_level),
    ),
    (Scopes::None, "change_local_variable", Vb(validate_change_variable)),
    (Scopes::Pop, "change_pop_culture", TargetValue("target", Scopes::Culture, "value")),
    (Scopes::Pop, "change_pop_religion", TargetValue("target", Scopes::Religion, "value")),
    (Scopes::Pop, "change_poptype", Scope(Scopes::PopType)),
    (Scopes::Country, "change_relations", Vb(validate_country_value)),
    (Scopes::Country, "change_subject_type", Item(Item::SubjectType)),
    (Scopes::Country, "change_tag", Item(Item::Country)),
    (Scopes::Country, "change_tension", Vb(validate_country_value)),
    (Scopes::None, "change_variable", Vb(validate_change_variable)),
    (Scopes::None, "clamp_global_variable", Vb(validate_clamp_variable)),
    (Scopes::None, "clamp_local_variable", Vb(validate_clamp_variable)),
    (Scopes::None, "clamp_variable", Vb(validate_clamp_variable)),
    (Scopes::Country, "clear_debt", Boolean),
    (Scopes::Country, "clear_enactment_modifier", Yes),
    (Scopes::None, "clear_global_variable_list", Unchecked),
    (Scopes::None, "clear_local_variable_list", Unchecked),
    (Scopes::None, "clear_saved_scope", Unchecked),
    (Scopes::Country, "clear_scaled_debt", ScriptValue),
    (Scopes::None, "clear_variable_list", Unchecked),
    (Scopes::Country, "complete_objective_subgoal", Item(Item::ObjectiveSubgoal)),
    (Scopes::State, "convert_population", TargetValue("target", Scopes::Religion, "value")),
    (Scopes::Country, "copy_laws", Scope(Scopes::Country)),
    (Scopes::Country, "create_bidirectional_truce", Vb(validate_create_truce)),
    (Scopes::State, "create_building", Vb(validate_create_building)),
    (Scopes::Country, "create_character", Vb(validate_create_character)),
    (Scopes::None, "create_country", Vb(validate_create_country)),
    (Scopes::Country, "create_diplomatic_catalyst", Vb(validate_create_catalyst)),
    (Scopes::Country, "create_diplomatic_pact", Vb(validate_diplomatic_pact)),
    (Scopes::Country, "create_diplomatic_play", Vb(validate_create_diplomatic_play)),
    (Scopes::None, "create_dynamic_country", Vb(validate_create_dynamic_country)),
    (Scopes::Country, "create_incident", Vb(validate_country_value)),
    (Scopes::State, "create_mass_migration", Vb(validate_create_mass_migration)),
    (Scopes::Country, "create_military_formation", Vb(validate_create_military_formation)),
    (Scopes::Country, "create_political_lobby", Vb(validate_create_lobby)),
    (Scopes::State, "create_pop", Vb(validate_create_pop)),
    (Scopes::Country, "create_power_bloc", Vb(validate_create_power_bloc)),
    (Scopes::StateRegion, "create_state", Vb(validate_create_state)),
    (Scopes::Country, "create_trade_route", Vb(validate_create_trade_route)),
    (
        Scopes::Country,
        "create_truce",
        Removed("1.7", "replaced with create_bidirectional_truce and create_unidirectional_truce"),
    ),
    (Scopes::Country, "create_unidirectional_truce", Vb(validate_create_truce)),
    (Scopes::None, "custom_description", Control),
    (Scopes::None, "custom_description_no_bullet", Control),
    (Scopes::None, "custom_label", ControlOrLabel),
    (Scopes::None, "custom_tooltip", ControlOrLabel),
    (Scopes::State, "deactivate_building", Item(Item::BuildingType)),
    (Scopes::Country, "deactivate_law", Scope(Scopes::LawType)),
    (Scopes::Country, "deactivate_parties", Yes),
    (Scopes::None, "debug_log", Unchecked),
    (Scopes::None, "debug_log_scopes", Boolean),
    (Scopes::Country, "decrease_autonomy", Yes),
    (Scopes::Character, "demobilize", Removed("1.6", "")),
    (Scopes::MilitaryFormation, "deploy_to_front", Scope(Scopes::Front)),
    (Scopes::Party, "disband_party", Yes),
    (Scopes::PoliticalLobby, "disband_political_lobby", Yes),
    (Scopes::Character, "disinherit_character", Yes),
    (Scopes::None, "else", Control),
    (Scopes::None, "else_if", Control),
    (Scopes::DiplomaticPlay, "end_play", Boolean),
    (Scopes::Country, "end_truce", Scope(Scopes::Country)),
    (Scopes::Character, "exile_character", Yes),
    (Scopes::State, "force_resource_depletion", Item(Item::BuildingGroup)),
    (Scopes::State, "force_resource_discovery", Item(Item::BuildingGroup)),
    (Scopes::Character, "free_character_from_void", Yes),
    (Scopes::MilitaryFormation, "fully_mobilize_army", Yes),
    (Scopes::None, "hidden_effect", Control),
    (Scopes::None, "if", Control),
    (Scopes::Country, "increase_autonomy", Yes),
    (Scopes::Country, "join_power_bloc", Scope(Scopes::Country)),
    (Scopes::InterestGroup, "join_revolution", Boolean),
    (Scopes::War, "join_war", Vb(validate_join_war)),
    (Scopes::Character, "kill_character", Vbv(validate_kill_character)),
    (Scopes::Country, "kill_population", UncheckedTodo),
    (Scopes::Country, "kill_population_in_state", UncheckedTodo),
    (Scopes::Country, "kill_population_percent", UncheckedTodo),
    (Scopes::State, "kill_population_percent_in_state", UncheckedTodo),
    (Scopes::TradeRoute, "lock_trade_route", Timespan),
    (Scopes::Country, "make_independent", Boolean),
    (Scopes::MilitaryFormation, "mobilize_army", Yes),
    (Scopes::Pop, "move_pop", Scope(Scopes::State)),
    (Scopes::Character, "place_character_in_void", ScriptValue),
    (Scopes::Country, "play_as", Scope(Scopes::Country)),
    (Scopes::None, "post_notification", Vv(validate_post_notification)),
    (Scopes::None, "post_proposal", UncheckedTodo),
    (Scopes::None, "random", Control),
    (Scopes::None, "random_list", Vb(validate_random_list)),
    (Scopes::None, "random_log_scopes", Boolean),
    (Scopes::Country, "recalculate_pop_ig_support", Boolean),
    (Scopes::Country, "regime_change", Scope(Scopes::Country)),
    (Scopes::Country, "remove_active_objective_subgoal", Item(Item::ObjectiveSubgoal)),
    (Scopes::Character, "remove_as_interest_group_leader", Yes),
    (Scopes::Country, "remove_banned_goods", Scope(Scopes::Goods)),
    (Scopes::State, "remove_building", Item(Item::BuildingType)),
    (Scopes::Character, "remove_character_role", Item(Item::CharacterRole)),
    (Scopes::StateRegion, "remove_claim", Scope(Scopes::Country)),
    (Scopes::Country, "remove_company", Scope(Scopes::CompanyType)),
    (Scopes::Culture, "remove_cultural_obsession", Item(Item::Goods)),
    (Scopes::Country, "remove_diplomatic_pact", Vb(validate_diplomatic_pact)),
    (Scopes::Country, "remove_enactment_modifier", UncheckedTodo), // No examples in vanilla
    (Scopes::all_but_none(), "remove_from_list", Vv(validate_remove_from_list)),
    (Scopes::None, "remove_global_variable", Unchecked),
    (Scopes::StateRegion, "remove_homeland", Scope(Scopes::Culture)),
    (Scopes::InterestGroup, "remove_ideology", Item(Item::Ideology)),
    (Scopes::Party, "remove_ig_from_party", Scope(Scopes::InterestGroup)),
    (Scopes::DiplomaticPlay, "remove_initiator_backers", Vb(validate_addremove_backers)),
    (Scopes::None, "remove_list_global_variable", Vb(validate_add_to_variable_list)),
    (Scopes::None, "remove_list_local_variable", Vb(validate_add_to_variable_list)),
    (Scopes::None, "remove_list_variable", Vb(validate_add_to_variable_list)),
    (Scopes::PoliticalLobby, "remove_lobby_member", Scope(Scopes::InterestGroup)),
    (Scopes::None, "remove_local_variable", Unchecked),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::Institution)
            .union(Scopes::InterestGroup)
            .union(Scopes::JournalEntry)
            .union(Scopes::PoliticalMovement)
            .union(Scopes::PowerBloc)
            .union(Scopes::State),
        "remove_modifier",
        Item(Item::Modifier),
    ),
    (Scopes::Country, "remove_primary_culture", Scope(Scopes::Culture)),
    (Scopes::PowerBloc, "remove_principle", Item(Item::Principle)),
    (Scopes::InterestGroup, "remove_ruling_interest_group", Boolean),
    (Scopes::StateRegion, "remove_state_trait", Item(Item::StateTrait)),
    (Scopes::DiplomaticPlay, "remove_target_backers", Vb(validate_addremove_backers)),
    (Scopes::Country, "remove_taxed_goods", Scope(Scopes::Goods)),
    (Scopes::Character, "remove_trait", Item(Item::CharacterTrait)),
    (Scopes::None, "remove_variable", Unchecked),
    (Scopes::DiplomaticPlay, "remove_war_goal", Vb(validate_remove_war_goal)),
    (Scopes::DiplomaticPlay, "resolve_play_for", Scope(Scopes::Country)),
    (Scopes::None, "round_global_variable", Vb(validate_round_variable)),
    (Scopes::None, "round_local_variable", Vb(validate_round_variable)),
    (Scopes::None, "round_variable", Vb(validate_round_variable)),
    (Scopes::all_but_none(), "save_scope_as", Vv(validate_save_scope)),
    (Scopes::None, "save_scope_value_as", Vb(validate_save_scope_value)),
    (Scopes::all_but_none(), "save_temporary_scope_as", Vv(validate_save_scope)),
    (Scopes::None, "save_temporary_scope_value_as", Vb(validate_save_scope_value)),
    (Scopes::Country, "seize_investment_pool", Boolean),
    (Scopes::Character, "set_as_interest_group_leader", Boolean),
    (Scopes::State, "set_available_for_autonomous_investment", Scope(Scopes::BuildingType)),
    (Scopes::JournalEntry, "set_bar_progress", Vb(validate_progress)),
    (Scopes::Country, "set_capital", Item(Item::StateRegion)),
    (Scopes::Character, "set_character_as_ruler", Yes),
    (
        Scopes::Character,
        "set_character_busy",
        Removed("1.11", "replaced with set_character_busy_and_immortal"),
    ),
    (Scopes::Character, "set_character_busy_and_immortal", Boolean),
    (Scopes::Character, "set_character_immortal", Boolean),
    (Scopes::Character, "set_commander_rank", Integer),
    (Scopes::Company, "set_company_establishment_date", Date),
    (Scopes::Country, "set_country_type", Item(Item::CountryType)),
    (Scopes::StateRegion, "set_devastation", ScriptValue),
    (Scopes::Country, "set_diplomats_expelled", Scope(Scopes::Country)),
    (Scopes::None, "set_global_variable", Vbv(validate_set_variable)),
    (Scopes::Country, "set_government_wage_level", Item(Item::Level)),
    (Scopes::Character, "set_home_country", Scope(Scopes::Country)),
    (Scopes::Character, "set_home_country_definition", Scope(Scopes::CountryDefinition)),
    (Scopes::Character, "set_ideology", Scope(Scopes::Ideology)),
    (Scopes::InterestGroup, "set_ig_bolstering", Boolean),
    (Scopes::InterestGroup, "set_ig_suppression", Boolean),
    (Scopes::InterestGroup, "set_ig_trait", Scope(Scopes::InterestGroupTrait)),
    (Scopes::Country, "set_immune_to_revolutions", Boolean),
    (Scopes::Country, "set_institution_investment_level", UncheckedTodo),
    (Scopes::InterestGroup, "set_interest_group_name", Item(Item::Localization)),
    (Scopes::DiplomaticPlay, "set_key", Item(Item::Localization)),
    (Scopes::None, "set_local_variable", Vbv(validate_set_variable)),
    (Scopes::Country, "set_market_capital", Item(Item::StateRegion)),
    (Scopes::Country, "set_military_wage_level", Item(Item::Level)),
    (Scopes::Country, "set_mutual_secret_goal", Vb(validate_set_secret_goal)),
    (Scopes::Country, "set_next_election_date", Date),
    (Scopes::Country, "set_owes_obligation_to", UncheckedTodo),
    (Scopes::StateRegion, "set_owner_of_provinces", UncheckedTodo),
    (Scopes::Pop, "set_pop_literacy", UncheckedTodo),
    (Scopes::Pop, "set_pop_qualifications", UncheckedTodo),
    (Scopes::Pop, "set_pop_wealth", Vb(validate_pop_wealth)),
    (Scopes::Country, "set_relations", Vb(validate_country_value)),
    (Scopes::Country, "set_ruling_interest_groups", UncheckedTodo),
    (Scopes::Party, "set_ruling_party", Yes),
    (Scopes::Country, "set_secret_goal", Vb(validate_set_secret_goal)),
    (Scopes::State, "set_state_owner", Scope(Scopes::Country)),
    (Scopes::Country, "set_state_religion", Scope(Scopes::Religion)),
    (Scopes::State, "set_state_type", Choice(STATE_TYPES)),
    (Scopes::Country, "set_strategy", Item(Item::AiStrategy)),
    (Scopes::Building, "set_subsidized", Boolean),
    (Scopes::JournalEntry, "set_target_technology", Scope(Scopes::Technology)),
    (Scopes::Country, "set_tariffs_export_priority", Scope(Scopes::Goods)),
    (Scopes::Country, "set_tariffs_import_priority", Scope(Scopes::Goods)),
    (Scopes::Country, "set_tariffs_no_priority", Scope(Scopes::Goods)),
    (Scopes::Country, "set_tax_level", Item(Item::Level)),
    (Scopes::Country, "set_tension", Vb(validate_country_value)),
    (Scopes::None, "set_variable", Vbv(validate_set_variable)),
    (Scopes::DiplomaticPlay, "set_war", Boolean),
    (Scopes::None, "show_as_tooltip", Control),
    (Scopes::State, "start_building_construction", Item(Item::BuildingType)),
    (Scopes::Country, "start_enactment", Scope(Scopes::LawType)),
    (Scopes::State, "start_privately_funded_building_construction", Item(Item::BuildingType)),
    (Scopes::Country, "start_research_random_technology", Yes),
    (Scopes::None, "start_tutorial_lesson", UncheckedTodo),
    (Scopes::None, "switch", Vb(validate_switch)),
    (Scopes::Country, "take_on_scaled_debt", TargetValue("who", Scopes::Country, "value")),
    (Scopes::MilitaryFormation, "teleport_to_front", Scope(Scopes::Front)),
    (Scopes::Character, "transfer_character", Scope(Scopes::Country)),
    (Scopes::Character, "transfer_to_formation", Scope(Scopes::MilitaryFormation)),
    (Scopes::None, "trigger_event", Vbv(validate_trigger_event)),
    (Scopes::Country, "try_form_government_with", Vb(validate_form_government)),
    (Scopes::State, "unset_available_for_autonomous_investment", Scope(Scopes::BuildingType)),
    (Scopes::Country, "update_party_support", Yes),
    (Scopes::Country, "validate_subsidies", Boolean),
    (Scopes::Country, "violate_sovereignty_join", UncheckedTodo),
    (Scopes::None, "while", Control),
];
