#![allow(non_upper_case_globals)]

pub type Scopes = u32;

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_scopes.log` from the game data dumps.
pub const None: Scopes = 0x0000_0001;
pub const Value: Scopes = 0x0000_0002;
pub const Bool: Scopes = 0x0000_0004;
pub const Flag: Scopes = 0x0000_0008;
pub const Character: Scopes = 0x0000_0010;
pub const LandedTitle: Scopes = 0x0000_0020;
pub const Activity: Scopes = 0x0000_0040;
pub const Secret: Scopes = 0x0000_0080;
pub const Province: Scopes = 0x0000_0100;
pub const Scheme: Scopes = 0x0000_0200;
pub const Combat: Scopes = 0x0000_0400;
pub const CombatSide: Scopes = 0x0000_0800;
pub const TitleAndVassalChange: Scopes = 0x0000_1000;
pub const Faith: Scopes = 0x0000_2000;
pub const GreatHolyWar: Scopes = 0x0000_4000;
pub const Religion: Scopes = 0x0000_8000;
pub const War: Scopes = 0x0001_0000;
pub const StoryCycle: Scopes = 0x0002_0000;
pub const CasusBelli: Scopes = 0x0004_0000;
pub const Dynasty: Scopes = 0x0008_0000;
pub const DynastyHouse: Scopes = 0x0010_0000;
pub const Faction: Scopes = 0x0020_0000;
pub const Culture: Scopes = 0x0040_0000;
pub const Army: Scopes = 0x0080_0000;
pub const HolyOrder: Scopes = 0x0100_0000;
pub const CouncilTask: Scopes = 0x0200_0000;
pub const MercenaryCompany: Scopes = 0x0400_0000;
pub const Artifact: Scopes = 0x0800_0000;
pub const Inspiration: Scopes = 0x1000_0000;
pub const Struggle: Scopes = 0x2000_0000;
pub const ALL: Scopes = 0x3fff_ffff;

pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if *s == name {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Faction, "faction_leader", Character),
    (Faction, "faction_target", Character),
    (Faction, "faction_war", War),
    (Faction, "special_character", Character),
    (Faction, "special_title", LandedTitle),
    (War, "casus_belli", CasusBelli),
    (Activity, "activity_owner", Character),
    (Activity, "activity_province", Province),
    (Army, "army_commander", Character),
    (Army, "army_owner", Character),
    (Artifact, "artifact_age", Value),
    (Artifact, "artifact_owner", Character),
    (Artifact, "creator", Character),
    (Artifact, "previous_owner", Character),
    (Artifact, "previous_owner_level_2", Character),
    (Artifact, "previous_owner_level_3", Character),
    (LandedTitle, "capital_vassal", LandedTitle),
    (LandedTitle, "current_heir", Character),
    (LandedTitle, "de_facto_liege", LandedTitle),
    (LandedTitle, "de_jure_liege", LandedTitle),
    (LandedTitle, "holder", Character),
    (LandedTitle, "lessee", Character),
    (LandedTitle, "lessee_title", LandedTitle),
    (LandedTitle, "previous_holder", Character),
    (LandedTitle, "title_capital_county", LandedTitle),
    (LandedTitle, "title_province", Province),
    (GreatHolyWar, "ghw_designated_winner", Character),
    (GreatHolyWar, "ghw_target_character", Character),
    (GreatHolyWar, "ghw_target_title", LandedTitle),
    (GreatHolyWar, "ghw_title_recipient", Character),
    (GreatHolyWar, "ghw_war", War),
    (GreatHolyWar, "ghw_war_declarer", Character),
    (Province, "province_owner", Character),
    (LandedTitle | Province, "barony", LandedTitle),
    (LandedTitle | Province, "barony_controller", Character),
    (LandedTitle | Province, "county", LandedTitle),
    (LandedTitle | Province, "county_controller", Character),
    (LandedTitle | Province, "duchy", LandedTitle),
    (LandedTitle | Province, "empire", LandedTitle),
    (LandedTitle | Province, "kingdom", LandedTitle),
    (Scheme, "scheme_artifact", Artifact),
    (Scheme, "scheme_defender", Character),
    (Scheme, "scheme_owner", Character),
    (Scheme, "scheme_target", Character),
    (Character | Combat | Army, "location", Province),
    (CouncilTask, "councillor", Character),
    (HolyOrder, "holy_order_patron", Character),
    (HolyOrder, "leader", Character),
    (HolyOrder, "title", LandedTitle),
    (War | CasusBelli, "claimant", Character),
    (War | CasusBelli, "primary_attacker", Character),
    (War | CasusBelli, "primary_defender", Character),
    (Character, "activity", Activity),
    (Character, "betrothed", Character),
    (Character, "capital_barony", LandedTitle),
    (Character, "capital_county", LandedTitle),
    (Character, "capital_province", Province),
    (Character, "commanding_army", Army),
    (Character, "concubinist", Character),
    (Character, "council_task", CouncilTask), // also has a prefix form
    (Character, "councillor_task_target", ALL), // output scope depends on task
    (Character, "court_owner", Character),
    (Character, "designated_heir", Character),
    (Character, "dynasty", Dynasty),
    (Character, "employer", Character),
    (Character, "father", Character),
    (Character, "ghw_beneficiary", Character),
    (Character, "host", Character),
    (Character, "house", DynastyHouse),
    (Character, "imprisoner", Character),
    (Character, "inspiration", Inspiration),
    (Character, "joined_faction", Faction),
    (Character, "killer", Character),
    (Character, "knight_army", Army),
    (Character, "last_played_character", Character),
    (Character, "liege", Character),
    (Character, "liege_or_court_owner", Character),
    (Character, "matchmaker", Character),
    (Character, "mother", Character),
    (Character, "player_heir", Character),
    (Character, "pregnancy_assumed_father", Character),
    (Character, "pregnancy_real_father", Character),
    (Character, "primary_heir", Character),
    (Character, "primary_partner", Character),
    (Character, "primary_spouse", Character),
    (Character, "primary_title", LandedTitle),
    (Character, "real_father", Character),
    (Character, "realm_priest", Character),
    (Character, "top_liege", Character),
    (DynastyHouse, "house_founder", Character),
    (DynastyHouse, "house_head", Character),
    (DynastyHouse, "last_house_head", Character),
    (Combat, "combat_attacker", CombatSide),
    (Combat, "combat_defender", CombatSide),
    (Combat, "combat_war", War),
    (CombatSide, "combat", Combat),
    (CombatSide, "enemy_side", CombatSide),
    (CombatSide, "side_commander", Character),
    (CombatSide, "side_primary_participant", Character),
    (Character, "faith", Faith),
    (LandedTitle | Province | GreatHolyWar, "faith", Faith),
    (Character | LandedTitle | Province, "culture", Culture),
    (
        Character | LandedTitle | Province | Faith | GreatHolyWar,
        "religion",
        Religion,
    ),
    (CasusBelli, "war", War),
    (Culture, "calc_culture_dominant_faith", Faith),
    (Culture, "calc_culture_dominant_religion", Religion),
    (Culture, "culture_head", Character),
    (StoryCycle, "story_owner", Character),
    (Faith, "founder", Character),
    (Faith, "great_holy_war", GreatHolyWar),
    (Faith, "religious_head", Character),
    (Faith, "religious_head_title", LandedTitle),
    (Inspiration, "inspiration_owner", Character),
    (Inspiration, "inspiration_sponsor", Character),
    (Secret, "secret_owner", Character),
    (Secret, "secret_target", Character),
    (Dynasty, "dynast", Character),
    (None, "dummy_female", Character),
    (None, "dummy_male", Character),
    // named_script_value special
    (None, "no", Bool),
    // "prev" special
    // "root" special
    // "this" special
    (None, "yes", Bool),
];

/// LAST UPDATED VERSION 1.6.2.2
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_FROM_PREFIX: &[(Scopes, &str, Scopes)] = &[
    (Character, "vassal_contract_obligation_level", Value),
    (Character, "aptitude", Value),
    (Character, "council_task", CouncilTask),
    (Character, "court_position", Character),
    (Character, "cp", Character), // councillor
    (None, "array_define", Value),
    (None, "character", Character),
    (Value, "compare_value", Value), // ?? needs more investigation
    (None, "culture", Culture),
    (None, "define", Value),
    (None, "dynasty", Dynasty),
    (None, "event_id", Flag),
    (None, "faith", Faith),
    (None, "flag", Flag),
    (None, "global_var", ALL),
    (None, "house", DynastyHouse),
    (None, "local_var", ALL),
    (None, "province", Province),
    (None, "religion", Religion),
    (None, "scope", ALL),
    (None, "struggle", Struggle),
    (None, "title", LandedTitle),
    (ALL, "var", ALL),
];
