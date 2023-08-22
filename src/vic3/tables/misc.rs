// LAST UPDATED VIC3 VERSION 1.3.6
pub const DLC_VIC3: &[&str] = &["dlc001", "dlc002", "dlc003", "dlc004", "dlc005", "dlc006"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const DLC_FEATURES_VIC3: &[&str] =
    &["voice_of_the_people_content", "voice_of_the_people_preorder", "agitators"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const APPROVALS: &[&str] = &["angry", "unhappy", "neutral", "happy", "loyal"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const ATTITUDES: &[&str] = &[
    "antagonistic",
    "belligerent",
    "cautious",
    "conciliatory",
    "cooperative",
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

// LAST UPDATED VIC3 VERSION 1.3.6
pub const LEVELS: &[&str] = &["very_low", "low", "medium", "high", "very_high"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const POLITICAL_MOVEMENTS: &[&str] =
    &["movement_to_enact", "movement_to_preserve", "movement_to_restore"];

// LAST UPDATED VIC3 VERSION 1.3.6
// TODO: verify if "neutral" really exists. It doesn't make much sense.
pub const RELATIONS: &[&str] =
    &["friendly", "amicable", "cordial", "neutral", "poor", "cold", "hostile"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const SECRET_GOALS: &[&str] =
    &["none", "befriend", "reconcile", "protect", "antagonize", "conquer", "dominate"];

// LAST UPDATED VIC3 VERSION 1.3.6
// Deduced from `common/government_types/`
pub const TRANSFER_OF_POWER: &[&str] =
    &["hereditary", "presidential_elective", "dictatorial", "parliamentary_elective"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const STRATA: &[&str] = &["poor", "middle", "rich"];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const WARGOALS: &[&str] = &[
    "annex_country",
    "ban_slavery",
    "colonization_rights",
    "conquer_state",
    "contain_threat",
    "force_recognition",
    "humiliation",
    "independence",
    "liberate_country",
    "liberate_subject",
    "make_dominion",
    "make_puppet",
    "make_vassal",
    "open_market",
    "regime_change",
    "return_state",
    "revoke_all_claims",
    "revoke_claim",
    "secession",
    "take_treaty_port",
    "transfer_subject",
    "unification",
    "unification_leadership",
    "war_reparations",
];

// LAST UPDATED VIC3 VERSION 1.3.6
// TODO: maybe ruler and heir too?
pub const CHARACTER_ROLES: &[&str] = &["admiral", "agitator", "general", "politician"];

// LAST UPDATED VIC3 VERSION 1.3.6
// Taken from the object browser
pub const SOUNDS_VIC3: &[&str] = &[
    "event:/MUSIC/Main/theme_01",
    "event:/MUSIC/Mood/V3/Base/01_A_Prospering_Country",
    "event:/MUSIC/Mood/V3/Base/02_Rule_The_World",
    "event:/MUSIC/Mood/V3/Base/03_Adagio_For_Four_Strings",
    "event:/MUSIC/Mood/V3/Base/04_At_The_Country_Manor",
    "event:/MUSIC/Mood/V3/Base/05_Benedicte",
    "event:/MUSIC/Mood/V3/Base/06_England_1851",
    "event:/MUSIC/Mood/V3/Base/07_Moonlight_Waltz",
    "event:/MUSIC/Mood/V3/Base/08_Our_New_Residence",
    "event:/MUSIC/Mood/V3/Base/09_Over_The_Calm_Ocean",
    "event:/MUSIC/Mood/V3/Base/10_Quite_Noble_Festivities",
    "event:/MUSIC/Mood/V3/Base/11_Remembering_Prince_Albert",
    "event:/MUSIC/Mood/V3/Base/12_Sunrise_Over_London",
    "event:/MUSIC/Mood/V3/Base/13_Sunset_Over_Windsor_Castle",
    "event:/MUSIC/Mood/V3/Base/14_Tea_Time",
    "event:/MUSIC/Mood/V3/Base/15_The_Queen_Is_Actually_Amused",
    "event:/MUSIC/Mood/V3/Base/16_To_Build_A_Factory",
    "event:/MUSIC/Mood/V3/Base/17_Asset_Gathering",
    "event:/MUSIC/Mood/V3/Base/18_British_Soil",
    "event:/MUSIC/Mood/V3/Base/19_Death_March",
    "event:/MUSIC/Mood/V3/Base/20_Glory_To_The_Queen",
    "event:/MUSIC/Stingers/diplomatic_play/begun",
    "event:/MUSIC/Stingers/events/civil",
    "event:/MUSIC/Stingers/events/dramatic",
    "event:/MUSIC/Stingers/events/enthusiastic",
    "event:/MUSIC/Stingers/events/political",
    "event:/MUSIC/Stingers/events/sadness",
    "event:/MUSIC/Stingers/events/spiritual",
    "event:/MUSIC/Stingers/events/tranquil",
    "event:/MUSIC/Stingers/game_over/negative",
    "event:/MUSIC/Stingers/game_over/positive",
    "event:/MUSIC/Stingers/toasts/acquired_technology",
    "event:/MUSIC/Stingers/toasts/country_revolution",
    "event:/MUSIC/Stingers/toasts/election_results_negative",
    "event:/MUSIC/Stingers/toasts/election_results_neutral",
    "event:/MUSIC/Stingers/toasts/election_results_positive",
    "event:/MUSIC/Stingers/toasts/heir_born",
    "event:/MUSIC/Stingers/toasts/journal_entry_completed",
    "event:/MUSIC/Stingers/toasts/law_changed",
    "event:/MUSIC/Stingers/toasts/migration_target_created_other",
    "event:/MUSIC/Stingers/toasts/native_uprising",
    "event:/MUSIC/Stingers/toasts/new_parties",
    "event:/MUSIC/Stingers/toasts/rank_changed",
    "event:/MUSIC/Stingers/toasts/used_favor",
    "event:/MUSIC/Stingers/unique_buildings/angkorwat",
    "event:/MUSIC/Stingers/unique_buildings/bigben",
    "event:/MUSIC/Stingers/unique_buildings/eiffeltower",
    "event:/MUSIC/Stingers/unique_buildings/forbiddencity",
    "event:/MUSIC/Stingers/unique_buildings/hagiasophia",
    "event:/MUSIC/Stingers/unique_buildings/mosqueofdjenna",
    "event:/MUSIC/Stingers/unique_buildings/saintbasilscathedral",
    "event:/MUSIC/Stingers/unique_buildings/statueofliberty",
    "event:/MUSIC/Stingers/unique_buildings/tajmahal",
    "event:/MUSIC/Stingers/unique_buildings/thevatican",
    "event:/MUSIC/Stingers/unique_buildings/thewhitehouse",
    "event:/MUSIC/Stingers/war/outcome_neutral",
    "event:/MUSIC/Stingers/war/start",
    "event:/SFX/Ambience/2D/master",
    "event:/SFX/Ambience/3D/Hub/city_african",
    "event:/SFX/Ambience/3D/Hub/city_arabic",
    "event:/SFX/Ambience/3D/Hub/city_asian",
    "event:/SFX/Ambience/3D/Hub/city_south_american",
    "event:/SFX/Ambience/3D/Hub/city_western",
    "event:/SFX/Ambience/3D/Hub/farm",
    "event:/SFX/Ambience/3D/Hub/forestry",
    "event:/SFX/Ambience/3D/Hub/industry",
    "event:/SFX/Ambience/3D/Hub/mining",
    "event:/SFX/Ambience/3D/Hub/oil_rig",
    "event:/SFX/Ambience/3D/Hub/plantation",
    "event:/SFX/Ambience/3D/Hub/port",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/agitator_speaking",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/barricade",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/conspiring",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/cops_march",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/french_algeria",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/garibaldi",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/gunboat_diplomacy",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/hostile_court",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/monarch_holding_court",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/people_sneaking",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/prison",
    "event:/SFX/DLC/1.3_ip1/Events/unspecific/realist_household",
    "event:/SFX/DLC/1.3_ip1/UI/agitator_promote",
    "event:/SFX/DLC/1.3_ip1/UI/character_interaction",
    "event:/SFX/DLC/1.3_ip1/UI/character_invite",
    "event:/SFX/DLC/1.3_ip1/UI/exile_character",
    "event:/SFX/DLC/1.3_ip1/UI/exile_pool_open",
    "event:/SFX/DLC/1.3_ip1/UI/generic_agitator_stinger",
    "event:/SFX/DLC/1.3_ip1/UI/historical_agitator_stinger",
    "event:/SFX/DLC/1.3_ip1/UI/item_revolutionary_movt",
    "event:/SFX/DLC/1.3_ip1/UI/main_menu_illustration",
    "event:/SFX/DLC/1.3_ip1/UI/new_country_start",
    "event:/SFX/DLC/1.3_ip1/UI/open_character_panel",
    "event:/SFX/DLC/1.3_ip1/UI/revolution_progress_tick",
    "event:/SFX/Events/africa/animism",
    "event:/SFX/Events/africa/city_center",
    "event:/SFX/Events/africa/construction_colony",
    "event:/SFX/Events/africa/desert_expedition",
    "event:/SFX/Events/africa/diplomats_negotiating",
    "event:/SFX/Events/africa/leader_arguing",
    "event:/SFX/Events/africa/prosperous_farm",
    "event:/SFX/Events/africa/public_protest",
    "event:/SFX/Events/africa/soldiers_breaking_protest",
    "event:/SFX/Events/asia/buddhism",
    "event:/SFX/Events/asia/confucianism_shinto",
    "event:/SFX/Events/asia/dead_cattle_poor_harvest",
    "event:/SFX/Events/asia/factory_accident",
    "event:/SFX/Events/asia/farmers_market",
    "event:/SFX/Events/asia/hinduism_sikhism",
    "event:/SFX/Events/asia/politician_parliament_motion",
    "event:/SFX/Events/asia/poor_people_moving",
    "event:/SFX/Events/asia/sepoy_mutiny",
    "event:/SFX/Events/asia/union_leader",
    "event:/SFX/Events/asia/westeners_arriving_in_east_asia",
    "event:/SFX/Events/europenorthamerica/american_civil_war",
    "event:/SFX/Events/europenorthamerica/before_the_battle",
    "event:/SFX/Events/europenorthamerica/capitalists_meeting",
    "event:/SFX/Events/europenorthamerica/gold_prospectors",
    "event:/SFX/Events/europenorthamerica/judaism",
    "event:/SFX/Events/europenorthamerica/london_center",
    "event:/SFX/Events/europenorthamerica/native_american",
    "event:/SFX/Events/europenorthamerica/opium_smoker",
    "event:/SFX/Events/europenorthamerica/political_extremism",
    "event:/SFX/Events/europenorthamerica/rich_and_poor",
    "event:/SFX/Events/europenorthamerica/russian_serfs",
    "event:/SFX/Events/europenorthamerica/slaves_breaking_their_chains",
    "event:/SFX/Events/europenorthamerica/springtime_of_nation",
    "event:/SFX/Events/europenorthamerica/sufferage",
    "event:/SFX/Events/generic/civil",
    "event:/SFX/Events/generic/clandestine",
    "event:/SFX/Events/generic/dramatic",
    "event:/SFX/Events/generic/enthusiastic",
    "event:/SFX/Events/generic/political",
    "event:/SFX/Events/generic/sadness",
    "event:/SFX/Events/generic/spiritual",
    "event:/SFX/Events/generic/tranquil",
    "event:/SFX/Events/middleeast/battlefield_trenches",
    "event:/SFX/Events/middleeast/courtroom_upheaval",
    "event:/SFX/Events/middleeast/engineer_blueprint",
    "event:/SFX/Events/middleeast/islam",
    "event:/SFX/Events/middleeast/jungle_expedition",
    "event:/SFX/Events/middleeast/middleclass_cafe",
    "event:/SFX/Events/middleeast/oil_derricks",
    "event:/SFX/Events/middleeast/police_breaking_door",
    "event:/SFX/Events/middleeast/upperclass_party",
    "event:/SFX/Events/misc/1Character_2Flags",
    "event:/SFX/Events/misc/1Character_4Flags",
    "event:/SFX/Events/misc/1Character_Banner",
    "event:/SFX/Events/misc/2Characters",
    "event:/SFX/Events/misc/Icons_Various",
    "event:/SFX/Events/southamerica/aristocrats",
    "event:/SFX/Events/southamerica/child_labor",
    "event:/SFX/Events/southamerica/christianity",
    "event:/SFX/Events/southamerica/election",
    "event:/SFX/Events/southamerica/factory_opening",
    "event:/SFX/Events/southamerica/public_figure_assassination",
    "event:/SFX/Events/southamerica/slaves_night",
    "event:/SFX/Events/southamerica/war_civilians",
    "event:/SFX/Events/unspecific/airplane",
    "event:/SFX/Events/unspecific/airship",
    "event:/SFX/Events/unspecific/arctic",
    "event:/SFX/Events/unspecific/armored_train",
    "event:/SFX/Events/unspecific/art_gallery",
    "event:/SFX/Events/unspecific/automobile",
    "event:/SFX/Events/unspecific/destruction",
    "event:/SFX/Events/unspecific/devastation",
    "event:/SFX/Events/unspecific/factory_closed",
    "event:/SFX/Events/unspecific/gears_pistons",
    "event:/SFX/Events/unspecific/iceberg_in_the_antartica",
    "event:/SFX/Events/unspecific/leader_speaking_to_a_group_of_people",
    "event:/SFX/Events/unspecific/military_parade",
    "event:/SFX/Events/unspecific/naval_battle",
    "event:/SFX/Events/unspecific/sick_people_in_a_field_hospital",
    "event:/SFX/Events/unspecific/signed_contract",
    "event:/SFX/Events/unspecific/steam_ship",
    "event:/SFX/Events/unspecific/temperance_movement",
    "event:/SFX/Events/unspecific/trains",
    "event:/SFX/Events/unspecific/vandalized_storefront",
    "event:/SFX/Events/unspecific/whaling",
    "event:/SFX/Events/unspecific/world_fair",
    "event:/SFX/UI/Alerts/current_situation",
    "event:/SFX/UI/Alerts/event_appear",
    "event:/SFX/UI/Alerts/high_attrition",
    "event:/SFX/UI/Alerts/letter_appear",
    "event:/SFX/UI/Alerts/notification",
    "event:/SFX/UI/Alerts/notification_collapse",
    "event:/SFX/UI/Alerts/notification_dismiss",
    "event:/SFX/UI/Alerts/notification_expand",
    "event:/SFX/UI/Alerts/Toasts/acquired_technology",
    "event:/SFX/UI/Alerts/Toasts/capitulated",
    "event:/SFX/UI/Alerts/Toasts/country_mobilization",
    "event:/SFX/UI/Alerts/Toasts/country_revolution",
    "event:/SFX/UI/Alerts/Toasts/election_results",
    "event:/SFX/UI/Alerts/Toasts/heir_born",
    "event:/SFX/UI/Alerts/Toasts/journal_entry_completed",
    "event:/SFX/UI/Alerts/Toasts/law_changed",
    "event:/SFX/UI/Alerts/Toasts/migration_target",
    "event:/SFX/UI/Alerts/Toasts/native_uprising",
    "event:/SFX/UI/Alerts/Toasts/new_parties",
    "event:/SFX/UI/Alerts/Toasts/peace_agreement",
    "event:/SFX/UI/Alerts/Toasts/rank_changed",
    "event:/SFX/UI/Alerts/Toasts/ranking_to_great_power",
    "event:/SFX/UI/Alerts/Toasts/_transient",
    "event:/SFX/UI/Alerts/Toasts/used_favor",
    "event:/SFX/UI/Alerts/warning_fist_appear",
    "event:/SFX/UI/Budget/coins_lvl_1",
    "event:/SFX/UI/Budget/coins_lvl_2",
    "event:/SFX/UI/Budget/coins_lvl_3",
    "event:/SFX/UI/Budget/coins_lvl_4",
    "event:/SFX/UI/Budget/coins_lvl_5",
    "event:/SFX/UI/Budget/pause_all",
    "event:/SFX/UI/Budget/resume_all",
    "event:/SFX/UI/Frontend/bookmark_bottom_show",
    "event:/SFX/UI/Frontend/start_game",
    "event:/SFX/UI/Frontend/start_panel_show",
    "event:/SFX/UI/Global/alert_remove",
    "event:/SFX/UI/Global/back",
    "event:/SFX/UI/Global/checkbox",
    "event:/SFX/UI/Global/close",
    "event:/SFX/UI/Global/confirm",
    "event:/SFX/UI/Global/decrement",
    "event:/SFX/UI/Global/exit_game",
    "event:/SFX/UI/Global/flag",
    "event:/SFX/UI/Global/game_pause",
    "event:/SFX/UI/Global/game_speed",
    "event:/SFX/UI/Global/game_unpause",
    "event:/SFX/UI/Global/increment",
    "event:/SFX/UI/Global/map_click",
    "event:/SFX/UI/Global/map_hover",
    "event:/SFX/UI/Global/map_hover_interact",
    "event:/SFX/UI/Global/panel_hide",
    "event:/SFX/UI/Global/panel_show",
    "event:/SFX/UI/Global/pause_logo",
    "event:/SFX/UI/Global/play_continue",
    "event:/SFX/UI/Global/play_pause",
    "event:/SFX/UI/Global/pointer_over",
    "event:/SFX/UI/Global/popup_hide",
    "event:/SFX/UI/Global/popup_show",
    "event:/SFX/UI/Global/promote",
    "event:/SFX/UI/Global/select",
    "event:/SFX/UI/Global/shimmer",
    "event:/SFX/UI/Global/situation",
    "event:/SFX/UI/Global/suppress",
    "event:/SFX/UI/Global/tab",
    "event:/SFX/UI/Global/tooltip_lock",
    "event:/SFX/UI/Global/victoria_logo",
    "event:/SFX/UI/Global/zoom",
    "event:/SFX/UI/MapInteraction/build_building",
    "event:/SFX/UI/MapInteraction/build_building_epic",
    "event:/SFX/UI/MapInteraction/civil",
    "event:/SFX/UI/MapInteraction/diplomatic_action_benign",
    "event:/SFX/UI/MapInteraction/diplomatic_action_hostile",
    "event:/SFX/UI/MapInteraction/diplomatic_action_interest",
    "event:/SFX/UI/MapInteraction/diplomatic_action_request",
    "event:/SFX/UI/MapInteraction/diplomatic_play",
    "event:/SFX/UI/MapInteraction/diplomatic_play_epic",
    "event:/SFX/UI/MapInteraction/establish_colony",
    "event:/SFX/UI/MapInteraction/map_interact_transient",
    "event:/SFX/UI/MapInteraction/trade_route",
    "event:/SFX/UI/MapLenses/diplomatic",
    "event:/SFX/UI/MapLenses/diplomatic_stop",
    "event:/SFX/UI/MapLenses/generic",
    "event:/SFX/UI/MapLenses/generic_open",
    "event:/SFX/UI/MapLenses/location_finder",
    "event:/SFX/UI/MapLenses/military",
    "event:/SFX/UI/MapLenses/military_stop",
    "event:/SFX/UI/MapLenses/mobilize_general",
    "event:/SFX/UI/MapLenses/political",
    "event:/SFX/UI/MapLenses/political_stop",
    "event:/SFX/UI/MapLenses/production",
    "event:/SFX/UI/MapLenses/production_stop",
    "event:/SFX/UI/MapLenses/trade",
    "event:/SFX/UI/MapLenses/trade_stop",
    "event:/SFX/UI/Market/filter/industrial",
    "event:/SFX/UI/Market/filter/luxury",
    "event:/SFX/UI/Market/filter/military",
    "event:/SFX/UI/Market/filter/staple",
    "event:/SFX/UI/MaxiMap/activate",
    "event:/SFX/UI/MaxiMap/deactivate",
    "event:/SFX/UI/Military/add_war_goal",
    "event:/SFX/UI/Military/commander_mobilize",
    "event:/SFX/UI/Military/commander_promote",
    "event:/SFX/UI/Military/commander_recruit",
    "event:/SFX/UI/Military/commander_retire",
    "event:/SFX/UI/Military/command_grant",
    "event:/SFX/UI/Military/command_remove",
    "event:/SFX/UI/Military/conscription_center_activate",
    "event:/SFX/UI/Military/order_admiral_convoy_raiding",
    "event:/SFX/UI/Military/order_admiral_intercept",
    "event:/SFX/UI/Military/order_admiral_naval_invasion",
    "event:/SFX/UI/Military/order_admiral_patrol",
    "event:/SFX/UI/Military/order_general_activate",
    "event:/SFX/UI/Military/order_general_front_advance",
    "event:/SFX/UI/Military/order_general_front_defend",
    "event:/SFX/UI/Military/order_general_standby",
    "event:/SFX/UI/Military/strategic_objective_confirm",
    "event:/SFX/UI/MusicPlayer/music_density_slider",
    "event:/SFX/UI/Popups/diplomatic_play_demobilize",
    "event:/SFX/UI/Popups/diplomatic_play_mobilize",
    "event:/SFX/UI/Popups/diplomatic_play_oppose",
    "event:/SFX/UI/Popups/diplomatic_play_support",
    "event:/SFX/UI/Popups/war_breaking_out",
    "event:/SFX/UI/Popups/war_to_arms",
    "event:/SFX/UI/SideBar/budget",
    "event:/SFX/UI/SideBar/budget_stop",
    "event:/SFX/UI/SideBar/buildings",
    "event:/SFX/UI/SideBar/buildings_stop",
    "event:/SFX/UI/SideBar/country",
    "event:/SFX/UI/SideBar/country_stop",
    "event:/SFX/UI/SideBar/culture",
    "event:/SFX/UI/SideBar/culture_stop",
    "event:/SFX/UI/SideBar/diplomacy",
    "event:/SFX/UI/SideBar/diplomacy_stop",
    "event:/SFX/UI/SideBar/journal",
    "event:/SFX/UI/SideBar/journal_stop",
    "event:/SFX/UI/SideBar/list_hide",
    "event:/SFX/UI/SideBar/list_show",
    "event:/SFX/UI/SideBar/markets",
    "event:/SFX/UI/SideBar/markets_stop",
    "event:/SFX/UI/SideBar/military",
    "event:/SFX/UI/SideBar/military_stop",
    "event:/SFX/UI/SideBar/outliner",
    "event:/SFX/UI/SideBar/outliner_stop",
    "event:/SFX/UI/SideBar/politics",
    "event:/SFX/UI/SideBar/politics_stop",
    "event:/SFX/UI/SideBar/population",
    "event:/SFX/UI/SideBar/population_stop",
    "event:/SFX/UI/SideBar/technology",
    "event:/SFX/UI/SideBar/technology_stop",
    "event:/SFX/UI/SideBar/vickypedia",
    "event:/SFX/UI/SideBar/vickypedia_stop",
    "event:/SFX/UI/Technology/confirm",
    "event:/SFX/Vehicles/bleriotxi",
    "event:/SFX/Vehicles/car",
    "event:/SFX/Vehicles/flatbed_truck",
    "event:/SFX/Vehicles/horse_cart",
    "event:/SFX/Vehicles/ships/ship_cargo",
    "event:/SFX/Vehicles/ships/ship_transport",
    "event:/SFX/Vehicles/ships/steamboat",
    "event:/SFX/Vehicles/tractor",
    "event:/SFX/Vehicles/train/cargo/logs",
    "event:/SFX/Vehicles/train/cargo/ore",
    "event:/SFX/Vehicles/train/diesel",
    "event:/SFX/Vehicles/train/electric",
    "event:/SFX/Vehicles/train/european_locomotive",
    "event:/SFX/Vehicles/zeppelin",
    "event:/SFX/Vehicles/zeppelin_2",
    "event:/SFX/VFX/building_demolish",
    "event:/SFX/VFX/building_demote",
    "event:/SFX/VFX/building_promote",
    "event:/SFX/VFX/conscription_center",
    "event:/SFX/VFX/devastation_stage_1",
    "event:/SFX/VFX/devastation_stage_2",
    "event:/SFX/VFX/devastation_stage_3",
    "event:/SFX/VFX/fireworks",
    "event:/SFX/VFX/geyser",
    "event:/SFX/VFX/pollution",
    "event:/SFX/VFX/rain",
    "event:/SFX/VFX/revolution_ongoing",
    "event:/SFX/VFX/sandstorm",
    "event:/SFX/VFX/scaffolding/big_start",
    "event:/SFX/VFX/scaffolding/big_stop",
    "event:/SFX/VFX/scaffolding/sml_start",
    "event:/SFX/VFX/scaffolding/sml_stop",
    "event:/SFX/VFX/scaffolding/special_start",
    "event:/SFX/VFX/scaffolding/special_stop",
    "event:/SFX/VFX/snow",
    "event:/SFX/VFX/unrest_2",
    "event:/SFX/VFX/unrest_3",
    "event:/SFX/VFX/unrest_4",
    "event:/SFX/VFX/volcano",
    "event:/SFX/VFX/war/armored_division/aerial_recon",
    "event:/SFX/VFX/war/armored_division/mechanized_infantry",
    "event:/SFX/VFX/war/artillery_breech",
    "event:/SFX/VFX/war/artillery_chemical",
    "event:/SFX/VFX/war/artillery_mobile",
    "event:/SFX/VFX/war/artillery/mobile/generic",
    "event:/SFX/VFX/war/artillery/siege/aerial_recon",
    "event:/SFX/VFX/war/artillery/siege/chemical",
    "event:/SFX/VFX/war/artillery/siege/generic",
    "event:/SFX/VFX/war/artillery/siege/machine_gunners",
    "event:/SFX/VFX/war/artillery/siege/shrapnel",
    "event:/SFX/VFX/war/artillery/siege/siege_artillery",
    "event:/SFX/VFX/war/campfire",
    "event:/SFX/VFX/war/infantry/irregular/generic",
    "event:/SFX/VFX/war/infantry/irregular/machine_gunners",
    "event:/SFX/VFX/war/infantry/line/cannon_artillery",
    "event:/SFX/VFX/war/infantry/line/flamethrower_company",
    "event:/SFX/VFX/war/infantry/line/generic",
    "event:/SFX/VFX/war/infantry/line/machine_gunners",
    "event:/SFX/VFX/war/infantry/line/skirmish",
    "event:/SFX/VFX/war/infantry/trench/aerial_recon",
    "event:/SFX/VFX/war/infantry/trench/generic",
    "event:/SFX/VFX/war/infantry/trench/machine_gunners",
    "event:/SFX/VFX/war/infantry/trench/squad",
    "event:/SFX/VFX/war/musket",
    "event:/SFX/VFX/war/rifle",
    "event:/SFX/VFX/war/rifle_bolt",
    "event:/SFX/VFX/war/rifle_repeating",
    "event:/SFX/VFX/war/ships/battleship",
    "event:/SFX/VFX/war/ships/ironclad",
    "event:/SFX/VFX/war/ships/ship_of_the_line",
    "event:/SFX/VFX/war/zone_center",
    "event:/SFX/VFX/war/zone_side",
    "event:/SFX/VFX/war/zone_snapshot_mute_2Damb",
    "event:/SFX/VFX/waterfall",
    "event:/SFX/VFX/whale_exhale",
    "snapshot:/Dynamic/duck_mx_map_height",
    "snapshot:/Dynamic/low_pass_world",
    "snapshot:/Dynamic/mute_2Damb_war",
    "snapshot:/Dynamic/mute_mx_100",
    "snapshot:/Dynamic/mute_mx_80",
    "snapshot:/Dynamic/mute_mx_80_music_player",
    "snapshot:/Dynamic/mute_veh_100",
    "snapshot:/Dynamic/mute_vfx_war_100",
    "snapshot:/Dynamic/mute_world_100",
    "snapshot:/Dynamic/mute_world_3D_60",
    "snapshot:/Dynamic/mute_world_80",
    "snapshot:/Gameplay/EventPopup",
    "snapshot:/Gameplay/GamePaused",
    "snapshot:/Gameplay/GameSpeedChange",
    "snapshot:/Gameplay/mute_music_mood",
    "snapshot:/Gameplay/mute_music_mood_via_toast_stinger",
    "snapshot:/Output/Headphones",
    "snapshot:/Output/Night_Mode",
    "snapshot:/Output/TV",
    "snapshot:/Settings/equal_loudness_contour",
    "snapshot:/States/FloatMapTransition",
    "snapshot:/States/MainMenu",
];

// LAST UPDATED VIC3 VERSION 1.3.6
pub const COMMON_DIRS: &[&str] = &[
    "common/achievement_groups.txt",
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
    "common/combat_unit_types",
    "common/commander_orders",
    "common/commander_ranks",
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
    "common/diplomatic_plays",
    "common/discrimination_traits",
    "common/dna_data",
    "common/dynamic_country_map_colors",
    "common/dynamic_country_names",
    "common/effect_localization",
    "common/ethnicities",
    "common/flag_definitions",
    "common/game_concepts",
    "common/game_rules",
    "common/genes",
    "common/goods",
    "common/government_types",
    "common/history/ai",
    "common/history/buildings",
    "common/history/characters",
    "common/history/conscription",
    "common/history/countries",
    "common/history/diplomacy",
    "common/history/diplomatic_plays",
    "common/history/global",
    "common/history/governments",
    "common/history/interests",
    "common/history/pops",
    "common/history/population",
    "common/history/production_methods",
    "common/history/states",
    "common/history/trade_routes",
    "common/ideologies",
    "common/institutions",
    "common/interest_group_traits",
    "common/interest_groups",
    "common/journal_entries",
    "common/labels",
    "common/law_groups",
    "common/laws",
    "common/legitimacy_levels",
    "common/map_interaction_types",
    "common/messages",
    "common/modifier_types",
    "common/modifiers",
    "common/named_colors",
    "common/objective_subgoal_categories",
    "common/objective_subgoals",
    "common/objectives",
    "common/on_actions",
    "common/opinion_modifiers",
    "common/parties",
    "common/pop_needs",
    "common/pop_types",
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
    "common/scripted_rules",
    "common/scripted_triggers",
    "common/state_traits",
    "common/strategic_regions",
    "common/subject_types",
    "common/technology",
    "common/technology/eras",
    "common/technology/technologies",
    "common/terrain",
    "common/terrain_manipulators",
    "common/terrain_manipulators/provinces",
    "common/trigger_localization",
    "common/tutorial_lesson_chains",
    "common/tutorial_lessons",
];
