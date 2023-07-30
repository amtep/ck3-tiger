//! Miscellaneous tables used to back `Item` variants.

// LAST UPDATED CK3 VERSION 1.9.2
pub const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

// LAST UPDATED CK3 VERSION 1.9.2
// Taken from the create_artifact description in effects.log
pub const ARTIFACT_HISTORY: &[&str] = &[
    "created_before_history",
    "created",
    "prize_created",
    "discovered",
    "creator_discovered",
    "claimed_by_house",
    "given",
    "stolen",
    "inherited",
    "conquest",
    "taken_in_siege",
    "taken_in_battle",
    "won_in_duel",
    "purchased",
    "prize_awarded",
    "ransomed",
    "reforged",
];

// LAST UPDATED CK3 VERSION 1.9.2
// TODO: parse it from dlc_metadata/ ? Unfortunately Tours and Tournaments
// is an exception.
pub const DLC_CK3: &[&str] = &[
    "Fashion of the Abbasid Court",
    "The Northern Lords",
    "Garments of the Holy Roman Empire",
    "The Fate of Iberia",
    "The Royal Court",
    "Friends and Foes",
    "tours_and_tournaments",
    "Elegance of the Empire",
];

/// LAST UPDATED CK3 VERSION 1.9.2
/// Entries verified in-game by seeing if datafunction `HasDlcFeature` logs an error.
pub const DLC_FEATURES_CK3: &[&str] = &[
    "garments_of_the_hre",
    "fashion_of_the_abbasid_court",
    "the_northern_lords",
    "hybridize_culture",
    "diverge_culture",
    "royal_court",
    "reform_culture",
    "court_artifacts",
    "the_fate_of_iberia",
    "friends_and_foes",
    "tours_and_tournaments",
    "advanced_activities",
    "accolades",
    "elegance_of_the_empire",
];

// LAST UPDATED CK3 VERSION 1.9.2
// Gathered from vanilla game files
pub const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

// LAST UPDATED CK3 VERSION 1.9.2
// Gathered from vanilla game files
pub const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

// LAST UPDATED CK3 VERSION 1.9.2
pub const SKILLS: &[&str] =
    &["diplomacy", "intrigue", "learning", "martial", "prowess", "stewardship"];

// LAST UPDATED CK3 VERSION 1.9.2
pub const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

// LAST UPDATED CK3 VERSION 1.9.2
// Taken from recent_history description in triggers.log
pub const TITLE_HISTORY_TYPES: &[&str] = &[
    "conquest",
    "conquest_holy_war",
    "conquest_claim",
    "conquest_populist",
    "election",
    "inheritance",
    "abdication",
    "created",
    "destroyed",
    "usurped",
    "granted",
    "revoked",
    "independency",
    "leased_out",
    "lease_revoked",
    "returned",
    "faction_demand",
    "swear_fealty",
];

// LAST UPDATED CK3 VERSION 1.9.2
// Gathered from vanilla common/traits/
pub const TRAIT_CATEGORIES: &[&str] = &[
    "personality",
    "education",
    "childhood",
    "commander",
    "winter_commander",
    "lifestyle",
    "court_type",
    "fame",
    "health",
];

// LAST UPDATED CK3 VERSION 1.9.2
// Gathered from vanilla game files
pub const DANGER_TYPES: &[&str] = &[
    "default",
    "battle",
    "raid",
    "siege",
    "army",
    "occupation",
    "county_control",
    "county_opinion",
    "owner_opinion",
];

// LAST UPDATED CK3 VERSION 1.9.2
pub const ARTIFACT_RARITY: &[&str] = &["common", "masterwork", "famed", "illustrious"];

// LAST UPDATED CK3 VERSION 1.9.1
pub const COMMON_DIRS: &[&str] = &[
    "common/accolade_icons",
    "common/accolade_names",
    "common/accolade_types",
    "common/achievement_groups.txt", // exception for this file
    "common/achievements",
    "common/activities/activity_locales",
    "common/activities/activity_types",
    "common/activities/guest_invite_rules",
    "common/activities/intents",
    "common/activities/pulse_actions",
    "common/ai_goaltypes",
    "common/ai_war_stances",
    "common/artifacts/blueprints",
    "common/artifacts/feature_groups",
    "common/artifacts/features",
    "common/artifacts/slots",
    "common/artifacts/templates",
    "common/artifacts/types",
    "common/artifacts/visuals",
    "common/bookmark_portraits",
    "common/bookmarks/bookmarks",
    "common/bookmarks/groups",
    "common/buildings",
    "common/casus_belli_groups",
    "common/casus_belli_types",
    "common/character_backgrounds",
    "common/character_interaction_categories",
    "common/character_interactions",
    "common/character_memory_types",
    "common/coat_of_arms/coat_of_arms",
    "common/coat_of_arms/dynamic_definitions",
    "common/coat_of_arms/options",
    "common/coat_of_arms/template_lists",
    "common/combat_effects",
    "common/combat_phase_events",
    "common/console_groups",
    "common/council_positions",
    "common/council_tasks",
    "common/court_amenities",
    "common/courtier_guest_management",
    "common/court_positions/categories",
    "common/court_positions/types",
    "common/court_types",
    "common/culture/aesthetics_bundles",
    "common/culture/creation_names",
    "common/culture/cultures",
    "common/culture/eras",
    "common/culture/innovations",
    "common/culture/name_equivalency",
    "common/culture/name_lists",
    "common/culture/pillars",
    "common/culture/traditions",
    "common/customizable_localization",
    "common/deathreasons",
    "common/decisions",
    "common/defines",
    "common/diarchies/diarchy_mandates",
    "common/diarchies/diarchy_types",
    "common/dna_data",
    "common/dynasties",
    "common/dynasty_house_motto_inserts",
    "common/dynasty_house_mottos",
    "common/dynasty_houses",
    "common/dynasty_legacies",
    "common/dynasty_perks",
    "common/effect_localization",
    "common/ethnicities",
    "common/event_backgrounds",
    "common/event_themes",
    "common/event_transitions",
    "common/factions",
    "common/flavorization",
    "common/focuses",
    "common/game_concepts",
    "common/game_rules",
    "common/genes",
    "common/governments",
    "common/guest_system",
    "common/holdings",
    "common/hook_types",
    "common/important_actions",
    "common/inspirations",
    "common/landed_titles",
    "common/laws",
    "common/lease_contracts",
    "common/lifestyle_perks",
    "common/lifestyles",
    "common/men_at_arms_types",
    "common/messages",
    "common/modifier_definition_formats",
    "common/modifier_icons",
    "common/modifiers",
    "common/named_colors",
    "common/nicknames",
    "common/on_action",
    "common/opinion_modifiers",
    "common/playable_difficulty_infos",
    "common/pool_character_selectors",
    "common/province_terrain",
    "common/religion/doctrines",
    "common/religion/fervor_modifiers",
    "common/religion/holy_sites",
    "common/religion/religion_families",
    "common/religion/religions",
    "common/schemes",
    "common/scripted_animations",
    "common/scripted_character_templates",
    "common/scripted_costs",
    "common/scripted_effects",
    "common/scripted_guis",
    "common/scripted_lists",
    "common/scripted_modifiers",
    "common/scripted_relations",
    "common/scripted_rules",
    "common/scripted_triggers",
    "common/script_values",
    "common/secret_types",
    "common/story_cycles",
    "common/struggle/catalysts",
    "common/struggle/struggles",
    "common/succession_election",
    "common/suggestions",
    "common/terrain_types",
    "common/traits",
    "common/travel/point_of_interest_types",
    "common/travel/travel_options",
    "common/trigger_localization",
    "common/tutorial_lesson_chains",
    "common/tutorial_lessons",
    "common/vassal_contracts",
    "common/vassal_stances",
];
