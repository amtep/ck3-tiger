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

/// LAST UPDATED CK3 VERSION 1.9.2
pub const ARTIFACT_RARITY: &[&str] = &["common", "masterwork", "famed", "illustrious"];
