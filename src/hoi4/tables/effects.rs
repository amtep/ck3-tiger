use std::sync::LazyLock;

use crate::effect::Effect;
use crate::effect_validation::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::hoi4::effect_validation::*;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;

use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    SCOPE_EFFECT_MAP.get(&*name_lc).copied()
}

/// A hashed version of [`SCOPE_EFFECT`], for quick lookup by effect name.
static SCOPE_EFFECT_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Effect)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, effect) in SCOPE_EFFECT.iter().copied() {
            hash.insert(s, (from, effect));
        }
        hash
    });

// LAST UPDATED HOI4 VERSION 1.16.4
// See `documentation/effects_documentation.md` from the game files.
// TODO HOI4
const SCOPE_EFFECT: &[(Scopes, &str, Effect)] = &[
    (Scopes::Country, "activate_advisor", Item(Item::Character)),
    (Scopes::Country, "activate_decision", Item(Item::Decision)),
    (Scopes::Country, "activate_mission", Item(Item::Mission)),
    (Scopes::Country, "activate_mission_tooltip", Item(Item::Mission)),
    (Scopes::Country, "activate_shine_on_focus", Item(Item::Focus)),
    (
        Scopes::Country.union(Scopes::State),
        "activate_targeted_decision",
        ItemTarget("decision", Item::Decision, "target", Scopes::Country.union(Scopes::State)),
    ),
    (Scopes::Country, "add_ace", Vb(validate_add_ace)),
    (Scopes::Country.union(Scopes::Character), "add_advisor_role", Vb(validate_add_advisor_role)),
    (Scopes::Country, "add_ai_strategy", UncheckedTodo),
    (Scopes::Character, "add_attack", UncheckedTodo),
    (Scopes::Country, "add_autonomy_ratio", UncheckedTodo),
    (Scopes::Country, "add_autonomy_score", UncheckedTodo),
    (Scopes::Country, "add_breakthrough_points", UncheckedTodo),
    (Scopes::Country, "add_breakthrough_progress", UncheckedTodo),
    (Scopes::State, "add_building_construction", UncheckedTodo),
    (Scopes::Country, "add_cic", UncheckedTodo),
    (Scopes::Country, "add_civil_war_target", UncheckedTodo),
    (Scopes::State, "add_claim_by", UncheckedTodo),
    (Scopes::Country, "add_collaboration", UncheckedTodo),
    (Scopes::Country, "add_command_power", UncheckedTodo),
    (Scopes::State, "add_compliance", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "add_contested_owner", UncheckedTodo),
    (Scopes::Character, "add_coordination", UncheckedTodo),
    (Scopes::State, "add_core_of", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_corps_commander_role", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_country_leader_role", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_country_leader_trait", UncheckedTodo),
    (Scopes::Country, "add_days_mission_timeout", UncheckedTodo),
    (Scopes::Country, "add_days_remove", UncheckedTodo),
    (Scopes::Country, "add_decryption", UncheckedTodo),
    (Scopes::Character, "add_defense", UncheckedTodo),
    (Scopes::Country, "add_design_template_bonus", UncheckedTodo),
    (Scopes::Division, "add_divisional_commander_xp", UncheckedTodo),
    (Scopes::Country, "add_doctrine_cost_reduction", UncheckedTodo),
    (
        Scopes::State.union(Scopes::Country).union(Scopes::Character).union(Scopes::SpecialProject),
        "add_dynamic_modifier",
        UncheckedTodo,
    ),
    (Scopes::Country, "add_equipment_bonus", UncheckedTodo),
    (Scopes::Country, "add_equipment_production", UncheckedTodo),
    (Scopes::Country, "add_equipment_subsidy", UncheckedTodo),
    (Scopes::Country, "add_equipment_to_stockpile", UncheckedTodo),
    (Scopes::State, "add_extra_state_shared_building_slots", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_field_marshal_role", UncheckedTodo),
    (Scopes::Country, "add_fuel", UncheckedTodo),
    (Scopes::Division, "add_history_entry", UncheckedTodo),
    (Scopes::Country, "add_ideas", UncheckedTodo),
    (Scopes::Country, "add_intel", UncheckedTodo),
    (Scopes::Country, "add_legitimacy", UncheckedTodo),
    (Scopes::Character, "add_logistics", UncheckedTodo),
    (Scopes::Character, "add_maneuver", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "add_manpower", UncheckedTodo),
    (Scopes::Character, "add_max_trait", UncheckedTodo),
    (Scopes::Country, "add_mines", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_design_team_assign_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_design_team_change_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_funds", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_funds_gain_factor", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_industrial_manufacturer_assign_cost", UncheckedTodo),
    (Scopes::Country, "add_mio_policy_cooldown", UncheckedTodo),
    (Scopes::Country, "add_mio_policy_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_research_bonus", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_size", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_size_up_requirement_factor", UncheckedTodo),
    (Scopes::IndustrialOrg, "add_mio_task_capacity", UncheckedTodo),
    (Scopes::Country, "add_named_threat", UncheckedTodo),
    (Scopes::Character, "add_nationality", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_naval_commander_role", UncheckedTodo),
    (Scopes::Country, "add_nuclear_bombs", UncheckedTodo),
    (Scopes::Country, "add_offsite_building", UncheckedTodo),
    (Scopes::Country, "add_operation_token", UncheckedTodo),
    (Scopes::Country, "add_opinion_modifier", UncheckedTodo),
    (Scopes::Character, "add_planning", UncheckedTodo),
    (Scopes::Country, "add_political_power", UncheckedTodo),
    (Scopes::Country, "add_popularity", UncheckedTodo),
    (Scopes::None, "add_power_balance_modifier", UncheckedTodo),
    (Scopes::None, "add_power_balance_value", UncheckedTodo),
    (Scopes::SpecialProject, "add_project_progress_ratio", UncheckedTodo),
    (Scopes::State, "add_province_modifier", UncheckedTodo),
    (Scopes::RaidInstance, "add_raid_history_entry", UncheckedTodo),
    (Scopes::Character, "add_random_trait", UncheckedTodo),
    (Scopes::Division, "add_random_valid_trait_from_unit", UncheckedTodo),
    (Scopes::StrategicRegion, "add_region_efficiency", UncheckedTodo),
    (Scopes::Country, "add_relation_modifier", UncheckedTodo),
    (Scopes::Country, "add_relation_rule_override", UncheckedTodo),
    (Scopes::Country, "add_research_slot", UncheckedTodo),
    (Scopes::State, "add_resistance", UncheckedTodo),
    (Scopes::State, "add_resistance_target", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "add_resource", UncheckedTodo),
    (Scopes::Country, "add_scaled_political_power", UncheckedTodo),
    (Scopes::Character, "add_scientist_level", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_scientist_role", UncheckedTodo),
    (Scopes::Character, "add_scientist_trait", UncheckedTodo),
    (Scopes::Character, "add_scientist_xp", UncheckedTodo),
    (Scopes::Character, "add_skill_level", UncheckedTodo),
    (Scopes::Country, "add_stability", UncheckedTodo),
    (Scopes::Country, "add_state_claim", UncheckedTodo),
    (Scopes::Country, "add_state_core", UncheckedTodo),
    (Scopes::State, "add_state_modifier", UncheckedTodo),
    (Scopes::Country, "add_tech_bonus", UncheckedTodo),
    (Scopes::Character, "add_temporary_buff_to_units", UncheckedTodo),
    (Scopes::Country, "add_threat", UncheckedTodo),
    (Scopes::Country, "add_timed_idea", UncheckedTodo),
    (Scopes::Character, "add_timed_unit_leader_trait", UncheckedTodo),
    (Scopes::None, "add_to_array", UncheckedTodo),
    (Scopes::Country, "add_to_faction", UncheckedTodo),
    (Scopes::Country, "add_to_tech_sharing_group", UncheckedTodo),
    (Scopes::None, "add_to_temp_array", UncheckedTodo),
    (Scopes::None, "add_to_temp_variable", UncheckedTodo),
    (Scopes::None, "add_to_variable", UncheckedTodo),
    (Scopes::Country, "add_to_war", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "add_trait", UncheckedTodo),
    (Scopes::Country, "add_unit_bonus", UncheckedTodo),
    (Scopes::Character, "add_unit_leader_trait", UncheckedTodo),
    (Scopes::Division, "add_unit_medal_to_latest_entry", UncheckedTodo),
    (Scopes::Country, "add_units_to_division_template", UncheckedTodo),
    (Scopes::None, "add_victory_points", UncheckedTodo),
    (Scopes::Country, "add_war_support", UncheckedTodo),
    (Scopes::Country, "ai_message", UncheckedTodo),
    (Scopes::Country, "air_experience", UncheckedTodo),
    (Scopes::Country, "annex_country", UncheckedTodo),
    (Scopes::Country, "army_experience", UncheckedTodo),
    (Scopes::Country, "become_exiled_in", UncheckedTodo),
    (Scopes::Character, "boost_planning", UncheckedTodo),
    (Scopes::Country, "break_embargo", UncheckedTodo),
    (Scopes::None, "build_railway", UncheckedTodo),
    (Scopes::None, "cancel_border_war", UncheckedTodo),
    (Scopes::PurchaseContract, "cancel_purchase_contract", UncheckedTodo),
    (Scopes::State, "cancel_resistance", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "capture_operative", UncheckedTodo),
    (Scopes::Country, "career_profile_step_missiolini", UncheckedTodo),
    (Scopes::Division, "change_division_template", UncheckedTodo),
    (Scopes::None, "change_tag_from", UncheckedTodo),
    (Scopes::Country, "character_list_tooltip", UncheckedTodo),
    (Scopes::None, "clamp_temp_variable", UncheckedTodo),
    (Scopes::None, "clamp_variable", UncheckedTodo),
    (Scopes::None, "clear_array", UncheckedTodo),
    (Scopes::Country, "clear_division_template_cap", UncheckedTodo),
    (Scopes::None, "clear_global_event_target", UncheckedTodo),
    (Scopes::None, "clear_global_event_targets", UncheckedTodo),
    (Scopes::Country, "clear_rule", UncheckedTodo),
    (Scopes::None, "clear_temp_array", UncheckedTodo),
    (Scopes::None, "clear_variable", UncheckedTodo),
    (Scopes::Character, "clr_character_flag", UncheckedTodo),
    (Scopes::Country, "clr_country_flag", UncheckedTodo),
    (Scopes::None, "clr_global_flag", UncheckedTodo),
    (Scopes::IndustrialOrg, "clr_mio_flag", UncheckedTodo),
    (Scopes::SpecialProject, "clr_project_flag", UncheckedTodo),
    (Scopes::State, "clr_state_flag", UncheckedTodo),
    (Scopes::Character, "clr_unit_leader_flag", UncheckedTodo),
    (Scopes::IndustrialOrg, "complete_mio_trait", UncheckedTodo),
    (Scopes::Country, "complete_national_focus", UncheckedTodo),
    (Scopes::SpecialProject, "complete_prototype_reward_option", UncheckedTodo),
    (Scopes::Country, "complete_special_project", UncheckedTodo),
    (Scopes::State, "construct_building_in_random_province", UncheckedTodo),
    (Scopes::Country, "country_event", UncheckedTodo),
    (Scopes::Country, "country_lock_all_division_template", UncheckedTodo),
    (Scopes::Country, "create_colonial_division_template", UncheckedTodo),
    (Scopes::Country, "create_corps_commander", UncheckedTodo),
    (Scopes::Country, "create_country_leader", UncheckedTodo),
    (Scopes::None, "create_dynamic_country", UncheckedTodo),
    (Scopes::None, "create_entity", UncheckedTodo),
    (Scopes::Country, "create_equipment_variant", UncheckedTodo),
    (Scopes::Country, "create_faction", UncheckedTodo),
    (Scopes::Country, "create_field_marshal", UncheckedTodo),
    (Scopes::Country, "create_import", UncheckedTodo),
    (Scopes::Country, "create_intelligence_agency", UncheckedTodo),
    (Scopes::Country, "create_navy_leader", UncheckedTodo),
    (Scopes::Country, "create_operative_leader", UncheckedTodo),
    (Scopes::Country, "create_production_license", UncheckedTodo),
    (Scopes::None, "create_purchase_contract", UncheckedTodo),
    (Scopes::None, "create_railway_gun", UncheckedTodo),
    (Scopes::Country, "create_ship", UncheckedTodo),
    (Scopes::None, "create_unit", UncheckedTodo),
    (Scopes::Country, "create_wargoal", UncheckedTodo),
    (Scopes::None, "custom_effect_tooltip", UncheckedTodo),
    (Scopes::None, "custom_override_tooltip", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "damage_building", UncheckedTodo),
    (Scopes::None, "damage_units", UncheckedTodo),
    (Scopes::Country, "deactivate_advisor", UncheckedTodo),
    (Scopes::Country, "deactivate_shine_on_focus", UncheckedTodo),
    (Scopes::Country, "declare_war_on", UncheckedTodo),
    (Scopes::Country, "delete_unit", UncheckedTodo),
    (Scopes::Country, "delete_unit_template_and_units", UncheckedTodo),
    (Scopes::Country, "delete_units", UncheckedTodo),
    (Scopes::Character, "demote_leader", UncheckedTodo),
    (Scopes::None, "destroy_entity", UncheckedTodo),
    (Scopes::Country, "destroy_ships", UncheckedTodo),
    (Scopes::Division, "destroy_unit", UncheckedTodo),
    (Scopes::Country, "diplomatic_relation", UncheckedTodo),
    (Scopes::Country, "dismantle_faction", UncheckedTodo),
    (Scopes::None, "divide_temp_variable", UncheckedTodo),
    (Scopes::None, "divide_variable", UncheckedTodo),
    (Scopes::Country, "division_template", UncheckedTodo),
    (Scopes::Country, "drop_cosmetic_tag", UncheckedTodo),
    (Scopes::None, "effect_tooltip", UncheckedTodo),
    (Scopes::Country, "end_exile", UncheckedTodo),
    (Scopes::Country, "end_puppet", UncheckedTodo),
    (Scopes::None, "event_option_tooltip", UncheckedTodo),
    (Scopes::Operation, "execute_operation_coordinated_strike", UncheckedTodo),
    (Scopes::None, "finalize_border_war", UncheckedTodo),
    (Scopes::None, "find_highest_in_array", UncheckedTodo),
    (Scopes::None, "find_lowest_in_array", UncheckedTodo),
    (Scopes::None, "for_each_loop", UncheckedTodo),
    (Scopes::None, "for_each_scope_loop", UncheckedTodo),
    (Scopes::None, "for_loop_effect", UncheckedTodo),
    (Scopes::State, "force_disable_resistance", UncheckedTodo),
    (Scopes::State, "force_enable_resistance", UncheckedTodo),
    (Scopes::Character, "force_operative_leader_into_hiding", UncheckedTodo),
    (
        Scopes::State.union(Scopes::Country).union(Scopes::Character).union(Scopes::SpecialProject),
        "force_update_dynamic_modifier",
        UncheckedTodo,
    ),
    (Scopes::None, "force_update_map_mode", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "free_operative", UncheckedTodo),
    (Scopes::Country, "free_random_operative", UncheckedTodo),
    (Scopes::Character, "gain_xp", UncheckedTodo),
    (Scopes::Country, "generate_character", UncheckedTodo),
    (Scopes::Country, "generate_scientist_character", UncheckedTodo),
    (Scopes::Country, "get_highest_scored_country", UncheckedTodo),
    (Scopes::Country, "get_highest_scored_country_temp", UncheckedTodo),
    (Scopes::Country, "get_sorted_scored_countries", UncheckedTodo),
    (Scopes::Country, "get_sorted_scored_countries_temp", UncheckedTodo),
    (Scopes::Country, "get_supply_vehicles", UncheckedTodo),
    (Scopes::Country, "get_supply_vehicles_temp", UncheckedTodo),
    (Scopes::Country, "give_guarantee", UncheckedTodo),
    (Scopes::Country, "give_market_access", UncheckedTodo),
    (Scopes::Country, "give_military_access", UncheckedTodo),
    (Scopes::Country, "give_resource_rights", UncheckedTodo),
    (Scopes::Country, "global_every_army_leader", UncheckedTodo),
    (Scopes::Country, "goto_province", UncheckedTodo),
    (Scopes::None, "goto_state", UncheckedTodo),
    (Scopes::Character, "harm_operative_leader", UncheckedTodo),
    (Scopes::None, "hidden_effect", UncheckedTodo),
    (Scopes::Country, "hold_election", UncheckedTodo),
    (Scopes::None, "if", UncheckedTodo),
    (Scopes::Country, "inherit_technology", UncheckedTodo),
    (Scopes::Character, "injure_scientist_for_days", UncheckedTodo),
    (Scopes::Country, "kill_country_leader", UncheckedTodo),
    (Scopes::Country, "kill_ideology_leader", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "kill_operative", UncheckedTodo),
    (Scopes::Country, "launch_nuke", UncheckedTodo),
    (Scopes::Country, "leave_faction", UncheckedTodo),
    (Scopes::Country, "load_focus_tree", UncheckedTodo),
    (Scopes::Country, "load_oob", UncheckedTodo),
    (Scopes::None, "log", UncheckedTodo),
    (Scopes::Country, "mark_focus_tree_layout_dirty", UncheckedTodo),
    (Scopes::Country, "mark_technology_tree_layout_dirty", UncheckedTodo),
    (Scopes::None, "meta_effect", UncheckedTodo),
    (Scopes::Country, "modify_building_resources", UncheckedTodo),
    (Scopes::Character, "modify_character_flag", UncheckedTodo),
    (Scopes::Country, "modify_country_flag", UncheckedTodo),
    (Scopes::None, "modify_global_flag", UncheckedTodo),
    (Scopes::IndustrialOrg, "modify_mio_flag", UncheckedTodo),
    (Scopes::SpecialProject, "modify_project_flag", UncheckedTodo),
    (Scopes::State, "modify_state_flag", UncheckedTodo),
    (Scopes::Country, "modify_tech_sharing_bonus", UncheckedTodo),
    (Scopes::Country, "modify_timed_idea", UncheckedTodo),
    (Scopes::Character, "modify_unit_leader_flag", UncheckedTodo),
    (Scopes::None, "modulo_temp_variable", UncheckedTodo),
    (Scopes::None, "modulo_variable", UncheckedTodo),
    (Scopes::None, "multiply_temp_variable", UncheckedTodo),
    (Scopes::None, "multiply_variable", UncheckedTodo),
    (Scopes::Country, "navy_experience", UncheckedTodo),
    (Scopes::Country, "news_event", UncheckedTodo),
    (Scopes::Character, "operative_leader_event", UncheckedTodo),
    (Scopes::Country, "party_leader", UncheckedTodo),
    (Scopes::None, "play_song", UncheckedTodo),
    (
        Scopes::State.union(Scopes::Country).union(Scopes::Character),
        "print_variables",
        UncheckedTodo,
    ),
    (Scopes::Country.union(Scopes::Character), "promote_character", UncheckedTodo),
    (Scopes::Character, "promote_leader", UncheckedTodo),
    (Scopes::Division, "promote_officer_to_general", UncheckedTodo),
    (Scopes::Country, "puppet", UncheckedTodo),
    (Scopes::RaidInstance, "raid_add_unit_experience", UncheckedTodo),
    (Scopes::RaidInstance, "raid_damage_units", UncheckedTodo),
    (Scopes::State, "raid_reduce_project_progress_ratio", UncheckedTodo),
    (Scopes::None, "random", UncheckedTodo),
    (Scopes::None, "randomize_temp_variable", UncheckedTodo),
    (Scopes::None, "randomize_variable", UncheckedTodo),
    (Scopes::None, "randomize_weather", UncheckedTodo),
    (Scopes::Country, "recall_attache", UncheckedTodo),
    (Scopes::Country, "recall_volunteers_from", UncheckedTodo),
    (Scopes::Country, "recruit_character", UncheckedTodo),
    (Scopes::Country, "release", UncheckedTodo),
    (Scopes::Country, "release_autonomy", UncheckedTodo),
    (Scopes::Country, "release_on_controlled", UncheckedTodo),
    (Scopes::Country, "release_puppet", UncheckedTodo),
    (Scopes::Country, "release_puppet_on_controlled", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_advisor_role", UncheckedTodo),
    (Scopes::None, "remove_all_power_balance_modifiers", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "remove_building", UncheckedTodo),
    (Scopes::Country, "remove_civil_war_target", UncheckedTodo),
    (Scopes::State, "remove_claim_by", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "remove_contested_owner", UncheckedTodo),
    (Scopes::State, "remove_core_of", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_country_leader_role", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_country_leader_trait", UncheckedTodo),
    (Scopes::Country, "remove_decision", UncheckedTodo),
    (Scopes::Country, "remove_decision_on_cooldown", UncheckedTodo),
    (
        Scopes::State.union(Scopes::Country).union(Scopes::Character).union(Scopes::SpecialProject),
        "remove_dynamic_modifier",
        UncheckedTodo,
    ),
    (Scopes::Character, "remove_exile_tag", UncheckedTodo),
    (Scopes::None, "remove_from_array", UncheckedTodo),
    (Scopes::Country, "remove_from_faction", UncheckedTodo),
    (Scopes::Country, "remove_from_tech_sharing_group", UncheckedTodo),
    (Scopes::None, "remove_from_temp_array", UncheckedTodo),
    (Scopes::Country, "remove_ideas", UncheckedTodo),
    (Scopes::Country, "remove_ideas_with_trait", UncheckedTodo),
    (Scopes::Country, "remove_mission", UncheckedTodo),
    (Scopes::Country, "remove_operation_token", UncheckedTodo),
    (Scopes::Country, "remove_opinion_modifier", UncheckedTodo),
    (Scopes::Country, "remove_power_balance", UncheckedTodo),
    (Scopes::None, "remove_power_balance_modifier", UncheckedTodo),
    (Scopes::State, "remove_province_modifier", UncheckedTodo),
    (Scopes::Country, "remove_relation_modifier", UncheckedTodo),
    (Scopes::Country, "remove_relation_rule_override", UncheckedTodo),
    (Scopes::State, "remove_resistance_target", UncheckedTodo),
    (Scopes::Country, "remove_resource_rights", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_scientist_role", UncheckedTodo),
    (Scopes::Country, "remove_state_claim", UncheckedTodo),
    (Scopes::Country, "remove_state_core", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "remove_targeted_decision", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_trait", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_unit_leader", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "remove_unit_leader_role", UncheckedTodo),
    (Scopes::Character, "remove_unit_leader_trait", UncheckedTodo),
    (Scopes::Country, "remove_wargoal", UncheckedTodo),
    (Scopes::Character, "replace_unit_leader_trait", UncheckedTodo),
    (Scopes::Division, "reseed_division_commander", UncheckedTodo),
    (Scopes::Country, "reserve_dynamic_country", UncheckedTodo),
    (Scopes::None, "reset_province_name", UncheckedTodo),
    (Scopes::State, "reset_state_name", UncheckedTodo),
    (Scopes::None, "resize_array", UncheckedTodo),
    (Scopes::None, "resize_temp_array", UncheckedTodo),
    (Scopes::Character, "retire", UncheckedTodo),
    (Scopes::Country, "retire_character", UncheckedTodo),
    (Scopes::Country, "retire_country_leader", UncheckedTodo),
    (Scopes::Country, "retire_ideology_leader", UncheckedTodo),
    (Scopes::Country, "reverse_add_opinion_modifier", UncheckedTodo),
    (Scopes::None, "round_temp_variable", UncheckedTodo),
    (Scopes::None, "round_variable", UncheckedTodo),
    (Scopes::None, "save_event_target_as", UncheckedTodo),
    (Scopes::None, "save_global_event_target_as", UncheckedTodo),
    (Scopes::Country, "scoped_play_song", UncheckedTodo),
    (Scopes::Country, "scoped_sound_effect", UncheckedTodo),
    (Scopes::Country, "send_embargo", UncheckedTodo),
    (Scopes::Country, "send_equipment", UncheckedTodo),
    (Scopes::Country, "send_equipment_fraction", UncheckedTodo),
    (Scopes::Country, "set_air_oob", UncheckedTodo),
    (Scopes::Country, "set_autonomy", UncheckedTodo),
    (Scopes::State, "set_border_war", UncheckedTodo),
    (Scopes::None, "set_border_war_data", UncheckedTodo),
    (Scopes::State, "set_building_level", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "set_can_be_fired_in_advisor_role", UncheckedTodo),
    (Scopes::Country, "set_capital", UncheckedTodo),
    (Scopes::Character, "set_character_flag", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "set_character_name", UncheckedTodo),
    (Scopes::Country, "set_collaboration", UncheckedTodo),
    (Scopes::State, "set_compliance", UncheckedTodo),
    (Scopes::Country, "set_cosmetic_tag", UncheckedTodo),
    (Scopes::Country, "set_country_flag", UncheckedTodo),
    (Scopes::Country, "set_country_leader_description", UncheckedTodo),
    (Scopes::Country, "set_country_leader_ideology", UncheckedTodo),
    (Scopes::Country, "set_country_leader_name", UncheckedTodo),
    (Scopes::Country, "set_country_leader_portrait", UncheckedTodo),
    (Scopes::State, "set_demilitarized_zone", UncheckedTodo),
    (Scopes::Country, "set_division_force_allow_recruiting", UncheckedTodo),
    (Scopes::Country, "set_division_template_cap", UncheckedTodo),
    (Scopes::Country, "set_division_template_lock", UncheckedTodo),
    (Scopes::None, "set_entity_animation", UncheckedTodo),
    (Scopes::None, "set_entity_movement", UncheckedTodo),
    (Scopes::None, "set_entity_position", UncheckedTodo),
    (Scopes::None, "set_entity_rotation", UncheckedTodo),
    (Scopes::None, "set_entity_scale", UncheckedTodo),
    (Scopes::Country, "set_equipment_fraction", UncheckedTodo),
    (Scopes::Country, "set_equipment_version_number", UncheckedTodo),
    (Scopes::Country, "set_faction_leader", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "set_faction_name", UncheckedTodo),
    (Scopes::Country, "set_faction_spymaster", UncheckedTodo),
    (Scopes::Country, "set_fuel", UncheckedTodo),
    (Scopes::Country, "set_fuel_ratio", UncheckedTodo),
    (Scopes::State, "set_garrison_strength", UncheckedTodo),
    (Scopes::None, "set_global_flag", UncheckedTodo),
    (Scopes::Country, "set_keyed_oob", UncheckedTodo),
    (Scopes::Character, "set_leader_description", UncheckedTodo),
    (Scopes::Character, "set_leader_name", UncheckedTodo),
    (Scopes::Character, "set_leader_portrait", UncheckedTodo),
    (Scopes::Country, "set_legitimacy", UncheckedTodo),
    (Scopes::Country, "set_major", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_design_team_assign_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_design_team_change_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_flag", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_funds", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_funds_gain_factor", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_icon", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_industrial_manufacturer_assign_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_name_key", UncheckedTodo),
    (Scopes::Country, "set_mio_policy_cooldown", UncheckedTodo),
    (Scopes::Country, "set_mio_policy_cost", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_research_bonus", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_size_up_requirement_factor", UncheckedTodo),
    (Scopes::IndustrialOrg, "set_mio_task_capacity", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "set_nationality", UncheckedTodo),
    (Scopes::Country, "set_naval_oob", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "set_occupation_law", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "set_occupation_law_where_available", UncheckedTodo),
    (Scopes::Country, "set_oob", UncheckedTodo),
    (Scopes::Country, "set_party_name", UncheckedTodo),
    (Scopes::Country, "set_party_rule", UncheckedTodo),
    (Scopes::Country, "set_political_party", UncheckedTodo),
    (Scopes::Country, "set_political_power", UncheckedTodo),
    (Scopes::Country, "set_politics", UncheckedTodo),
    (Scopes::Country, "set_popularities", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "set_portraits", UncheckedTodo),
    (Scopes::Country, "set_power_balance", UncheckedTodo),
    (Scopes::None, "set_power_balance_gfx", UncheckedTodo),
    (Scopes::SpecialProject, "set_project_flag", UncheckedTodo),
    (Scopes::Country, "set_province_controller", UncheckedTodo),
    (Scopes::None, "set_province_name", UncheckedTodo),
    (Scopes::Country, "set_relation_rule", UncheckedTodo),
    (Scopes::Country, "set_research_slots", UncheckedTodo),
    (Scopes::State, "set_resistance", UncheckedTodo),
    (Scopes::Country, "set_rule", UncheckedTodo),
    (Scopes::Country, "set_stability", UncheckedTodo),
    (Scopes::State, "set_state_category", UncheckedTodo),
    (Scopes::Country, "set_state_controller", UncheckedTodo),
    (Scopes::State, "set_state_controller_to", UncheckedTodo),
    (Scopes::State, "set_state_flag", UncheckedTodo),
    (Scopes::State, "set_state_name", UncheckedTodo),
    (Scopes::Country, "set_state_owner", UncheckedTodo),
    (Scopes::State, "set_state_owner_to", UncheckedTodo),
    (Scopes::State, "set_state_province_controller", UncheckedTodo),
    (Scopes::Country, "set_technology", UncheckedTodo),
    (Scopes::None, "set_temp_variable", UncheckedTodo),
    (Scopes::None, "set_temp_variable_to_random", UncheckedTodo),
    (Scopes::Country, "set_truce", UncheckedTodo),
    (Scopes::Character, "set_unit_leader_flag", UncheckedTodo),
    (Scopes::Division, "set_unit_organization", UncheckedTodo),
    (Scopes::None, "set_variable", UncheckedTodo),
    (Scopes::None, "set_variable_to_random", UncheckedTodo),
    (Scopes::None, "set_victory_points", UncheckedTodo),
    (Scopes::Country, "set_war_support", UncheckedTodo),
    (Scopes::Country, "show_ideas_tooltip", UncheckedTodo),
    (Scopes::Country, "show_mio_tooltip", UncheckedTodo),
    (Scopes::Country, "show_unit_leaders_tooltip", UncheckedTodo),
    (Scopes::None, "sound_effect", UncheckedTodo),
    (Scopes::None, "start_border_war", UncheckedTodo),
    (Scopes::Country, "start_civil_war", UncheckedTodo),
    (Scopes::Country, "start_peace_conference", UncheckedTodo),
    (Scopes::State, "start_resistance", UncheckedTodo),
    (Scopes::State.union(Scopes::Country), "state_event", UncheckedTodo),
    (Scopes::Country, "steal_random_tech_bonus", UncheckedTodo),
    (Scopes::None, "subtract_from_temp_variable", UncheckedTodo),
    (Scopes::None, "subtract_from_variable", UncheckedTodo),
    (Scopes::Character, "supply_units", UncheckedTodo),
    (Scopes::Character, "swap_country_leader_traits", UncheckedTodo),
    (Scopes::Country, "swap_ideas", UncheckedTodo),
    (Scopes::Country, "swap_ruler_traits", UncheckedTodo),
    (Scopes::State, "teleport_armies", UncheckedTodo),
    (Scopes::Country, "teleport_railway_guns_to_deploy_province", UncheckedTodo),
    (Scopes::Country, "transfer_navy", UncheckedTodo),
    (Scopes::Country, "transfer_ship", UncheckedTodo),
    (Scopes::Country, "transfer_state", UncheckedTodo),
    (Scopes::State, "transfer_state_to", UncheckedTodo),
    (Scopes::Country, "transfer_units_fraction", UncheckedTodo),
    (Scopes::Country.union(Scopes::Character), "turn_operative", UncheckedTodo),
    (Scopes::Country, "uncomplete_national_focus", UncheckedTodo),
    (Scopes::Character, "unit_leader_event", UncheckedTodo),
    (Scopes::Country, "unlock_decision_category_tooltip", UncheckedTodo),
    (Scopes::Country, "unlock_decision_tooltip", UncheckedTodo),
    (Scopes::Country, "unlock_military_industrial_organization_tooltip", UncheckedTodo),
    (Scopes::None, "unlock_mio_policy_tooltip", UncheckedTodo),
    (Scopes::IndustrialOrg, "unlock_mio_trait_tooltip", UncheckedTodo),
    (Scopes::Country, "unlock_national_focus", UncheckedTodo),
    (Scopes::Country, "upgrade_intelligence_agency", UncheckedTodo),
    (Scopes::None, "while_loop_effect", UncheckedTodo),
    (Scopes::Country, "white_peace", UncheckedTodo),
];
