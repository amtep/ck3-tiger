//! Miscellaneous tables used to back `Item` variants.

// LAST UPDATED CK3 VERSION 1.11.3
pub const ACTIVITY_STATES: &[&str] = &["passive", "travel", "active"];

// LAST UPDATED CK3 VERSION 1.13.0.1
// Taken from the agent_slot_has_contribution_type trigger doc
pub const AGENT_SLOT_CONTRIBUTION_TYPE: &[&str] =
    &["secrecy", "success_chance", "success_chance_growth", "success_chance_max", "speed"];

// LAST UPDATED CK3 VERSION 1.12.1
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

/// LAST UPDATED CK3 VERSION 1.13.0.1
/// Taken from `has_dlc_feature` in triggers.log
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
    "legacy_of_persia",
    "elegance_of_the_empire",
    "wards_and_wardens",
    "legends_of_the_dead",
    "legends",
    "north_african_attire",
    "couture_of_the_capets",
    "landless_playable",
    "admin_gov",
    "roads_to_power",
    "court_room_view",
    "wandering_nobles",
    "west_slavic_attire",
];

/// A list of music provided by DLCs, for people who don't have them
/// LAST UPDATED CK3 VERSION 1.12.1
pub const DLC_MUSIC: &[&str] = &[
    // FP1
    "mx_raid",
    "mx_drakkar",
    "mx_scandinavia",
    "mx_thefeast",
    // EP1
    "middleeasterncourt_cue",
    "europeancourt_cue",
    "indiancourt_cue",
    "mediterraneancourt_cue",
    "mep1_mood_01",
    "mep1_mood_02",
    "mep1_mood_03",
    "mep1_mood_04",
    "group_roco",
    // FP2
    "mx_IberiaWar",
    "mx_Struggle_ending_compromise",
    "mx_Struggle_ending_conciliation",
    "mx_Struggle_ending_hostility",
    "mx_Struggle_Opening",
    "mx_iberian_moodTrack1",
    "mx_iberian_moodTrack2",
    "mx_iberian_moodTrack3",
    "group_foi",
    // BP1
    "mx_BP1Mood_Generic",
    "mx_BP1Mood_Western",
    "mx_BP1Mood_MiddleEastern",
    "group_bp1",
    // EP2
    "tournamentwest_cue",
    "tournamentmena_cue",
    "tournamentindia_cue",
    "tournamentend_cue",
    "tourwest_cue",
    "tourmena_cue",
    "tourindia_cue",
    "tourend_cue",
    "weddingwest_cue",
    "weddingmena_cue",
    "weddingindia_cue",
    "weddingend_cue",
    "grandfeast_cue",
    "murderfeast_event_cue",
    "murderfest_cue",
    "india_arrival_neutral_cue",
    "india_arrival_suspicious_cue",
    "india_arrival_welcome_cue",
    "mena_arrival_neutral_cue",
    "mena_arrival_suspicious_cue",
    "mena_arrival_welcome_cue",
    "west_arrival_neutral_cue",
    "west_arrival_suspicious_cue",
    "west_arrival_welcome_cue",
    "mep2_mood_01",
    "mep2_mood_02",
    "mep2_mood_03",
    "mep2_mood_04",
    "group_ep2_cuetrack",
    "group_ep2_moodtrack",
    "mx_cue_tournament_win",
    "mx_cue_tournament_lose",
    "mx_cue_tournament_brawl",
    "mx_cue_tournament_horse",
    "mx_cue_tournament_mind",
    "mx_cue_armorer",
    "mx_cue_visitor_camp",
    "mx_cue_farrier",
    "mx_cue_fletcher",
    "mx_cue_tourney_grounds",
    "mx_cue_settlement",
    "mx_cue_tailor",
    "mx_cue_tavern",
    "mx_cue_temple",
    "mx_cue_weaponsmith",
    // BP2
    "mbp2_mood_01",
    "mbp2_mood_02",
    "mbp2_mood_03",
    "mbp2_mood_04",
    "group_bp2_moodtrack",
    // FP3
    "strugglestart_cue",
    "struggleend_cue",
    "strugglewar_cue",
    "mfp3_mood_01",
    "mfp3_mood_02",
    "mfp3_mood_03",
    "mfp3_mood_04",
    "mfp3_mood_05",
    "group_fp3_cuetrack",
    "group_fp3_moodtrack",
    // CE1 (documented as fp4)
    "apocalyptic_plague",
    "black_death",
    "legend_begins",
    "mfp4_mood_epidemics_01",
    "mfp4_mood_epidemics_02",
    "mfp4_mood_legends_01",
    "mfp4_mood_legends_02",
    "group_fp4_cuetrack",
    "group_fp4_moodtrack",
];

// LAST UPDATED CK3 VERSION 1.13.0.1
// taken from the governments .info file
pub const GOVERNMENT_RULES: &[&str] = &[
    "create_cadet_branches",
    "religious",
    "court_generate_spouses",
    "council",
    "rulers_should_have_dynasty",
    "regiments_prestige_as_gold",
    "dynasty_named_realms",
    "royal_court",
    "legitimacy",
    "administrative",
    "landless_playable",
    "use_as_base_on_landed",
    "use_as_base_on_rank_up",
    "conditional_maa_refill",
    "mercenary",
    "use_title_tier_modifiers",
    "inherit_from_dynastic_government",
    "state_faith",                    // undocumented
    "allow_out_of_realm_inheritance", // undocumented
];

// LAST UPDATED CK3 VERSION 1.12.1
pub const LEGEND_QUALITY: &[&str] = &["famed", "illustrious", "mythical"];

// LAST UPDATED CK3 VERSION 1.12.1
// Gathered from vanilla game files
pub const REWARD_ITEMS: &[&str] = &["newsletter_crown"];

// LAST UPDATED CK3 VERSION 1.12.1
// Gathered from vanilla game files
pub const PRISON_TYPES: &[&str] = &["dungeon", "house_arrest"];

// LAST UPDATED CK3 VERSION 1.12.1
pub const SKILLS: &[&str] =
    &["diplomacy", "intrigue", "learning", "martial", "prowess", "stewardship"];

// LAST UPDATED CK3 VERSION 1.12.1
pub const SEXUALITIES: &[&str] = &["heterosexual", "homosexual", "bisexual", "asexual", "none"];

// LAST UPDATED CK3 VERSION 1.13.0.1
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
    "stepped_down",
    "appointment",
    "appointment_succession",
];

// LAST UPDATED CK3 VERSION 1.12.1
// Gathered from vanilla common/traits/
pub const TRAIT_CATEGORIES: &[&str] = &[
    "childhood",
    "commander",
    "court_type",
    "education",
    "fame",
    "health",
    "lifestyle",
    "personality",
    "winter_commander",
];

// LAST UPDATED CK3 VERSION 1.12.1
// Gathered from vanilla game files, both the use of the `travel_danger_type` trigger and what's in
// the localization files for `travel_danger_type_*`
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
    "epidemic",
];

// LAST UPDATED CK3 VERSION 1.12.1
pub const ARTIFACT_RARITIES: &[&str] = &["common", "masterwork", "famed", "illustrious"];

// LAST UPDATED CK3 VERSION 1.12.1
pub const OUTBREAK_INTENSITIES: &[&str] = &["minor", "major", "apocalyptic"];

// LAST UPDATED CK3 VERSION 1.14.0.2
pub const COMMON_DIRS: &[&str] = &[
    "common/accolade_icons",
    "common/accolade_names",
    "common/accolade_types",
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
    "common/bookmarks/challenge_characters",
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
    "common/court_positions/categories",
    "common/court_positions/tasks",
    "common/court_positions/types",
    "common/court_types",
    "common/courtier_guest_management",
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
    "common/decision_group_types",
    "common/decisions",
    "common/defines",
    "common/diarchies/diarchy_mandates",
    "common/diarchies/diarchy_types",
    "common/dna_data",
    "common/domiciles/buildings",
    "common/domiciles/types",
    "common/dynasties",
    "common/dynasty_house_motto_inserts",
    "common/dynasty_house_mottos",
    "common/dynasty_houses",
    "common/dynasty_legacies",
    "common/dynasty_perks",
    "common/effect_localization",
    "common/epidemics",
    "common/ethnicities",
    "common/event_2d_effects",
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
    "common/house_power_bonus",
    "common/house_unities",
    "common/important_actions",
    "common/inspirations",
    "common/landed_titles",
    "common/laws",
    "common/lease_contracts",
    "common/legends/chronicles",
    "common/legends/legend_seeds",
    "common/legends/legend_types",
    "common/legitimacy",
    "common/lifestyle_perks",
    "common/lifestyles",
    "common/men_at_arms_types",
    "common/message_filter_types",
    "common/message_group_types",
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
    "common/portrait_types",
    "common/province_terrain",
    "common/religion/doctrines",
    "common/religion/fervor_modifiers",
    "common/religion/holy_sites",
    "common/religion/religion_families",
    "common/religion/religions",
    "common/schemes/agent_types",
    "common/schemes/pulse_actions",
    "common/schemes/scheme_countermeasures",
    "common/schemes/scheme_types",
    "common/script_values",
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
    "common/secret_types",
    "common/story_cycles",
    "common/struggle/catalysts",
    "common/struggle/struggles",
    "common/succession_appointment",
    "common/succession_election",
    "common/suggestions",
    "common/task_contracts",
    "common/tax_slots/obligations",
    "common/tax_slots/types",
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

// LAST UPDATED CK3 VERSION 1.13
// As of 1.13, all common dirs can have subdirectories.
pub const COMMON_SUBDIRS_OK: &[&str] = COMMON_DIRS;
