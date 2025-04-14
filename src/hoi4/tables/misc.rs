// LAST UPDATED HOI4 VERSION 1.16
// Taken from common/ai_strategy/_documentation.md
pub const AI_STRATEGY_TYPES: &[&str] = &[
    "alliance",
    "antagonize",
    "avoid_starting_wars",
    "asking_foreign_garrison",
    "befriend",
    "conquer",
    "consider_weak",
    "contain",
    "declare_war",
    "diplo_action_acceptance",
    "diplo_action_desire",
    "dont_join_wars_with",
    "ignore",
    "ignore_claim",
    "influence",
    "prepare_for_war",
    "protect",
    "send_lend_lease_desire",
    "send_volunteers_desire",
    "support",
    "area_priority",
    "dont_defend_ally_borders",
    "force_defend_ally_borders",
    "force_concentration_front_factor",
    "force_concentration_factor",
    "force_concentration_target_weight",
    "front_armor_score",
    "front_control",
    "front_unit_request",
    "garrison",
    "garrison_reinforcement_priority",
    "ignore_army_incompetence",
    "invasion_unit_request",
    "invade",
    "occupation_policy",
    "put_unit_buffers",
    "scorched_earth_prio",
    "spare_unit_factor",
    "theatre_distribution_demand_increase",
    "naval_avoid_region",
    "naval_convoy_raid_region",
    "naval_invasion_focus",
    "naval_invasion_supremacy_weight",
    "naval_mission_threshold",
    "strike_force_home_base",
    "activate_crypto",
    "agency_ai_base_num_factories_factor",
    "agency_ai_per_upgrade_factories_factor",
    "decrypt_target",
    "intelligence_agency_branch_desire_factor",
    "intelligence_agency_usable_factories",
    "operation_equipment_priority",
    "operative_mission",
    "operative_operation",
    "become_spymaster",
    "added_military_to_civilian_factory_ratio",
    "air_factory_balance",
    "build_airplane",
    "build_army",
    "build_building",
    "build_ship",
    "building_target",
    "convoy_efficiency_to_cancel_trades",
    "dockyard_to_military_factory_ratio",
    "equipment_production_factor",
    "equipment_variant_production_factor",
    "equipment_production_surplus_management",
    "equipment_production_min_factories",
    "equipment_production_min_factories_archetype",
    "equipment_stockpile_surplus_ratio",
    "equipment_market_spend_factories",
    "equipment_market_for_sale_threshold",
    "equipment_market_for_sale_factor",
    "equipment_market_max_for_sale",
    "equipment_market_min_for_sale",
    "equipment_market_buying_threshold",
    "equipment_market_buy",
    "equipment_market_trade_desire",
    "factory_build_score_factor",
    "force_build_armies",
    "fuel_buffer",
    "min_convoy_efficiency_factor_for_war_support_hit",
    "production_upgrade_desire_offset",
    "railway_gun_divisions_ratio",
    "research_tech",
    "research_weight_factor",
    "role_ratio",
    "save_equipment",
    "template_prio",
    "unit_ratio",
    "land_xp_spend_priority",
    "air_xp_spend_priority",
    "navy_xp_spend_priority",
    "pp_spend_amount",
    "pp_spend_priority",
    "min_wanted_supply_trucks",
    "wanted_supply_trucks",
    "min_wanted_supply_trains",
    "wanted_supply_trains",
    "ai_wanted_divisions_factor",
    "strategic_air_importance",
    "raid_target_country",
];

/// A list of music provided by DLCs, for people who don't have them
// LAST UPDATED HOI4 VERSION 1.16
pub const DLC_MUSIC: &[&str] = &[
    // TODO
];

// LAST UPDATED HOI4 VERSION 1.16
pub const COMMON_DIRS: &[&str] = &[
    "common/abilities",
    "common/aces",
    "common/ai_areas",
    "common/ai_equipment",
    "common/ai_focuses",
    "common/ai_strategy",
    "common/ai_strategy_plans",
    "common/ai_templates",
    "common/autonomous_states",
    "common/bookmarks",
    "common/bop",
    "common/buildings",
    "common/characters",
    "common/continuous_focus",
    "common/countries",
    "common/country_leader",
    "common/country_tag_aliases",
    "common/country_tags",
    "common/decisions",
    "common/decisions/categories",
    "common/defines",
    "common/difficulty_settings",
    "common/dynamic_modifiers",
    "common/equipment_groups",
    "common/focus_inlay_windows",
    "common/game_rules",
    "common/generation",
    "common/ideas",
    "common/idea_tags",
    "common/ideologies",
    "common/intelligence_agencies",
    "common/intelligence_agency_upgrades",
    "common/map_modes",
    "common/medals",
    "common/military_industrial_organization/ai_bonus_weights",
    "common/military_industrial_organization/organizations",
    "common/military_industrial_organization/policies",
    "common/modifier_definitions",
    "common/modifiers",
    "common/mtth",
    "common/names",
    "common/national_focus",
    "common/occupation_laws",
    "common/on_actions",
    "common/operation_phases",
    "common/operations",
    "common/operation_tokens",
    "common/opinion_modifiers",
    "common/peace_conference/ai_peace",
    "common/peace_conference/categories",
    "common/peace_conference/cost_modifiers",
    "common/profile_backgrounds",
    "common/profile_pictures",
    "common/raids",
    "common/raids/categories",
    "common/resistance_activity",
    "common/resistance_compliance_modifiers",
    "common/resources",
    "common/ribbons",
    "common/scientist_traits",
    "common/scorers",
    "common/scorers/country",
    "common/script_constants",
    "common/scripted_diplomatic_actions",
    "common/scripted_effects",
    "common/scripted_guis",
    "common/scripted_localisation",
    "common/scripted_triggers",
    "common/special_projects/projects",
    "common/special_projects/project_tags",
    "common/special_projects/prototype_rewards",
    "common/special_projects/specialization",
    "common/state_category",
    "common/technologies",
    "common/technology_sharing",
    "common/technology_tags",
    "common/terrain",
    "common/timed_activities",
    "common/unit_leader",
    "common/unit_medals",
    "common/units",
    "common/units/codenames_operatives",
    "common/units/critical_parts",
    "common/units/equipment",
    "common/units/equipment/modules",
    "common/units/equipment/upgrades",
    "common/units/names",
    "common/units/names_divisions",
    "common/units/names_railway_guns",
    "common/units/names_ships",
    "common/units/unit_modifiers",
    "common/unit_tags",
    "common/wargoals",
];

// LAST UPDATED HOI4 VERSION 1.16
pub const COMMON_SUBDIRS_OK: &[&str] = &[];

// LAST UPDATED HOI4 VERSION 1.16
pub const COMMON_FILES: &[&str] = &[
    "common/acclimatation.txt",
    "common/achievements.txt",
    "common/ai_attitudes.txt",
    "common/ai_personalities.txt",
    "common/alerts.txt",
    "common/combat_tactics.txt",
    "common/event_modifiers.txt",
    "common/graphicalculturetype.txt",
    "common/region_colors.txt",
    "common/script_enums.txt",
    "common/triggered_modifiers.txt",
    "common/weather.txt",
];
