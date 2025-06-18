// LAST UPDATED VIC3 VERSION 1.9.0
pub const DLC_FEATURES_VIC3: &[&str] = &[
    "voice_of_the_people_content",
    "voice_of_the_people_preorder",
    "agitators",
    "rp1_content",
    "power_bloc_features",
    "ep1_content",
    "foreign_investment",
    "lobbies",
    "subject_and_bloc_actions",
    "ep1_cosmetics",
    "ip2_content",
    "ip2_cosmetics",
    "mp1_content",
];

/// A list of music provided by DLCs, for people who don't have them
pub const DLC_MUSIC: &[&str] = &[
    // TODO
];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const EVENT_CATEGORIES: &[&str] = &["enactment", "revolution"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const APPROVALS: &[&str] = &["angry", "unhappy", "neutral", "happy", "loyal"];

// LAST UPDATED VIC3 VERSION 1.7.1
pub const ATTITUDES: &[&str] = &[
    "antagonistic",
    "belligerent",
    "cautious",
    "conciliatory",
    "cooperative",
    "defiant",
    "disinterested",
    "domineering",
    "genial",
    "human",
    "loyal",
    "protective",
    "rebellious",
    "wary",
];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const COUNTRY_TIERS: &[&str] =
    &["city_state", "principality", "grand_principality", "kingdom", "empire", "hegemony"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const INFAMY_THRESHOLDS: &[&str] = &["notorious", "infamous", "pariah"];

// LAST UPDATED VIC3 VERSION 1.7.0
pub const LOBBY_FORMATION_REASON: &[&str] = &[
    "diplomacy",
    "defense",
    "ideology",
    "funded",
    "aggression",
    "religion",
    "technology",
    "economy",
    "none",
];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const LEVELS: &[&str] = &["very_low", "low", "medium", "high", "very_high"];

// LAST UPDATED VIC3 VERSION 1.3.6
// TODO: verify if "neutral" really exists. It doesn't make much sense.
pub const RELATIONS: &[&str] =
    &["friendly", "amicable", "cordial", "neutral", "poor", "cold", "hostile"];

// LAST UPDATED VIC3 VERSION 1.7.1
pub const SECRET_GOALS: &[&str] = &[
    "none",
    "befriend",
    "reconcile",
    "protect",
    "antagonize",
    "conquer",
    "dominate",
    "comply",
    "defy",
];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const STANCES: &[&str] =
    &["strongly_disapprove", "disapprove", "neutral", "approve", "strongly_approve"];

// LAST UPDATED VIC3 VERSION 1.7.6
pub const STATE_TYPES: &[&str] = &["incorporated", "unincorporated", "treaty_port"];

// LAST UPDATED VIC3 VERSION 1.3.6
// Deduced from `common/government_types/`
pub const TRANSFER_OF_POWER: &[&str] =
    &["hereditary", "presidential_elective", "dictatorial", "parliamentary_elective"];

// LAST UPDATED VIC3 VERSION 1.8.1
pub const STRATA: &[&str] = &["lower", "middle", "upper"];

// LAST UPDATED VIC3 VERSION 1.9.0
// Gathered from usage in vanilla
pub const TARIFF_LEVELS: &[&str] = &[
    "max_subventions",
    "high_subventions",
    "low_subventions",
    "no_tariffs_or_subventions",
    "low_tariffs",
    "high_tariffs",
    "max_tariffs",
];

// LAST UPDATED VIC3 VERSION 1.9.0
// From common/treaty_articles/treaty_articles.md
pub const TREATY_ARTICLE_CATEGORIES: &[&str] = &[
    "economy",
    "trade",
    "military",
    "military_defense",
    "ideology",
    "expansion",
    "power_bloc",
    "other",
];

// LAST UPDATED VIC3 VERSION 1.8.1
// Taken from `localization/english/diplomatic_plays_l_english.yml` entries
// that start with `war_goal_`.
pub const WARGOALS: &[&str] = &[
    "annex_country",
    "ban_slavery",
    "colonization_rights",
    "conquer_state",
    "contain_threat",
    "force_nationalization",
    "force_recognition",
    "foreign_investment_rights",
    "humiliation",
    "increase_autonomy",
    "independence",
    "join_power_bloc",
    "leave_power_bloc",
    "liberate_country",
    "liberate_subject",
    "make_dominion",
    "make_protectorate",
    "make_tributary",
    "open_market",
    "reduce_autonomy",
    "regime_change",
    "return_state",
    "revoke_all_claims",
    "revoke_claim",
    "revolution",
    "secession",
    "take_treaty_port",
    "transfer_subject",
    "unification",
    "unification_leadership",
    "war_reparations",
];

// LAST UPDATED VIC3 VERSION 1.3.6
// TODO: maybe ruler and heir too?
pub const CHARACTER_ROLES: &[&str] = &["admiral", "agitator", "general", "politician", "executive"];

// LAST UPDATED VIC3 VERSION 1.7.1
// Taken from common/labels/00_terrain_labels.txt and the unit_offense_ and unit_defense_ modifs.
pub const TERRAIN_KEYS: &[&str] = &[
    "flat",
    "elevated",
    "forested",
    "hazardous",
    "developed",
    "water",
    "travel_harsh_environment",
];

// LAST UPDATED VIC3 VERSION 1.7.0
pub const COMMON_DIRS: &[&str] = &[
    "common/acceptance_statuses",
    "common/achievements",
    "common/ai_strategies",
    "common/alert_groups",
    "common/alert_types",
    "common/battle_conditions",
    "common/building_groups",
    "common/buildings",
    "common/buy_packages",
    "common/canals",
    "common/character_interactions",
    "common/character_templates",
    "common/character_traits",
    "common/coat_of_arms/coat_of_arms",
    "common/coat_of_arms/options",
    "common/coat_of_arms/template_lists",
    "common/cohesion_levels",
    "common/combat_unit_experience_levels",
    "common/combat_unit_groups",
    "common/combat_unit_types",
    "common/commander_orders",
    "common/commander_ranks",
    "common/company_charter_types",
    "common/company_types",
    "common/country_creation",
    "common/country_definitions",
    "common/country_formation",
    "common/country_ranks",
    "common/country_types",
    "common/culture_graphics",
    "common/cultures",
    "common/customizable_localization",
    "common/decisions",
    "common/decrees",
    "common/defines",
    "common/diplomatic_actions",
    "common/diplomatic_catalyst_categories",
    "common/diplomatic_catalysts",
    "common/diplomatic_plays",
    "common/discrimination_traits",
    "common/dna_data",
    "common/dynamic_company_names",
    "common/dynamic_country_map_colors",
    "common/dynamic_country_names",
    "common/dynamic_treaty_names",
    "common/effect_localization",
    "common/ethnicities",
    "common/flag_definitions",
    "common/game_concepts",
    "common/game_rules",
    "common/genes",
    "common/goods",
    "common/government_types",
    "common/harvest_condition_types",
    "common/history",
    "common/ideologies",
    "common/institutions",
    "common/interest_group_traits",
    "common/interest_groups",
    "common/journal_entries",
    "common/journal_entry_groups",
    "common/labels",
    "common/law_groups",
    "common/laws",
    "common/legitimacy_levels",
    "common/liberty_desire_levels",
    "common/map_interaction_types",
    "common/map_notification_types",
    "common/messages",
    "common/military_formation_flags",
    "common/mobilization_option_groups",
    "common/mobilization_options",
    "common/modifier_type_definitions",
    "common/named_colors",
    "common/objective_subgoal_categories",
    "common/objective_subgoals",
    "common/objectives",
    "common/on_actions",
    "common/opinion_modifiers",
    "common/parties",
    "common/political_lobbies",
    "common/political_lobby_appeasement",
    "common/political_movement_categories",
    "common/political_movement_pop_support",
    "common/political_movements",
    "common/pop_needs",
    "common/pop_types",
    "common/power_bloc_coa_pieces",
    "common/power_bloc_identities",
    "common/power_bloc_map_textures",
    "common/power_bloc_names",
    "common/power_bloc_principle_groups",
    "common/power_bloc_principles",
    "common/prestige_goods",
    "common/production_method_groups",
    "common/production_methods",
    "common/proposal_types",
    "common/religions",
    "common/script_values",
    "common/scripted_buttons",
    "common/scripted_effects",
    "common/scripted_guis",
    "common/scripted_lists",
    "common/scripted_modifiers",
    "common/scripted_progress_bars",
    "common/scripted_rules",
    "common/scripted_triggers",
    "common/social_classes",
    "common/social_hierarchies",
    "common/state_traits",
    "common/static_modifiers",
    "common/strategic_regions",
    "common/subject_types",
    "common/technology/eras",
    "common/technology/technologies",
    "common/terrain",
    "common/terrain_manipulators",
    "common/terrain_manipulators/provinces",
    "common/themes",
    "common/treaty_articles",
    "common/trigger_localization",
    "common/tutorial_lesson_chains",
    "common/tutorial_lessons",
];

// LAST UPDATED VIC3 VERSION 1.7.6
pub const COMMON_SUBDIRS_OK: &[&str] = &["common/defines", "common/history"];
