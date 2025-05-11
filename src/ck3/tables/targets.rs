use std::sync::LazyLock;

use crate::helpers::TigerHashMap;
use crate::scopes::{ArgumentValue, Scopes};

#[inline]
pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_TO_SCOPE_MAP.get(name).copied()
}

static SCOPE_TO_SCOPE_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to) in SCOPE_TO_SCOPE.iter().copied() {
            hash.insert(s, (from, to));
        }
        hash
    });

/// LAST UPDATED CK3 VERSION 1.16.0
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Accolade, "acclaimed_knight", Scopes::Character),
    (Scopes::Character, "accolade", Scopes::Accolade),
    (Scopes::Accolade, "accolade_owner", Scopes::Character),
    (Scopes::Accolade, "accolade_successor", Scopes::Character),
    (Scopes::Activity, "activity_host", Scopes::Character),
    (Scopes::Activity, "activity_location", Scopes::Province),
    (Scopes::Activity, "activity_type", Scopes::ActivityType),
    (Scopes::Army, "army_commander", Scopes::Character),
    (Scopes::Army, "army_owner", Scopes::Character),
    (Scopes::Artifact, "artifact_age", Scopes::Value),
    (Scopes::Artifact, "artifact_owner", Scopes::Character),
    (Scopes::Character, "assigned_tax_slot", Scopes::TaxSlot),
    (Scopes::LandedTitle.union(Scopes::Province), "barony", Scopes::LandedTitle),
    (Scopes::LandedTitle.union(Scopes::Province), "barony_controller", Scopes::Character),
    (Scopes::Character, "betrothed", Scopes::Character),
    (Scopes::Culture, "calc_culture_dominant_faith", Scopes::Faith),
    (Scopes::Culture, "calc_culture_dominant_religion", Scopes::Religion),
    (Scopes::Character, "capital_barony", Scopes::LandedTitle),
    (Scopes::Character, "capital_county", Scopes::LandedTitle),
    (Scopes::Character, "capital_province", Scopes::Province),
    (Scopes::LandedTitle, "capital_vassal", Scopes::LandedTitle),
    (Scopes::War, "casus_belli", Scopes::CasusBelli),
    (Scopes::War.union(Scopes::CasusBelli), "claimant", Scopes::Character),
    (Scopes::CombatSide, "combat", Scopes::Combat),
    (Scopes::Combat, "combat_attacker", Scopes::CombatSide),
    (Scopes::Combat, "combat_defender", Scopes::CombatSide),
    (Scopes::Combat, "combat_war", Scopes::War),
    (Scopes::Character, "commanding_army", Scopes::Army),
    (Scopes::Value, "compare_value", Scopes::Value), // special
    (Scopes::Character, "concubinist", Scopes::Character),
    (Scopes::Character, "confederation", Scopes::Confederation),
    (Scopes::Character, "council_task", Scopes::CouncilTask), // also has a prefix form
    (Scopes::CouncilTask, "councillor", Scopes::Character),
    (Scopes::Character, "councillor_task_target", Scopes::all()), // output scope depends on task
    (Scopes::LandedTitle.union(Scopes::Province), "county", Scopes::LandedTitle),
    (Scopes::LandedTitle.union(Scopes::Province), "county_controller", Scopes::Character),
    (Scopes::Character, "court_owner", Scopes::Character),
    (Scopes::Artifact, "creator", Scopes::Character),
    (
        Scopes::Character.union(Scopes::LandedTitle).union(Scopes::Province),
        "culture",
        Scopes::Culture,
    ),
    (Scopes::Culture, "culture_head", Scopes::Character),
    (Scopes::LandedTitle, "current_heir", Scopes::Character),
    (Scopes::TravelPlan, "current_location", Scopes::Province),
    (Scopes::Legend, "current_or_last_legend_owner", Scopes::Character),
    (Scopes::Character, "current_travel_plan", Scopes::TravelPlan),
    (Scopes::LandedTitle, "de_facto_liege", Scopes::LandedTitle),
    (Scopes::LandedTitle, "de_jure_liege", Scopes::LandedTitle),
    (Scopes::Character, "default_location", Scopes::Province),
    (Scopes::TravelPlan, "departure_location", Scopes::Province),
    (Scopes::Character, "designated_diarch", Scopes::Character),
    (Scopes::Character, "designated_heir", Scopes::Character),
    (Scopes::Character, "diarch", Scopes::Character),
    (Scopes::Character, "diarchy_successor", Scopes::Character),
    (Scopes::Character, "domicile", Scopes::Domicile),
    (Scopes::Domicile, "domicile_culture", Scopes::Culture),
    (Scopes::Domicile, "domicile_faith", Scopes::Faith),
    (Scopes::Domicile, "domicile_location", Scopes::Province),
    (Scopes::LandedTitle.union(Scopes::Province), "duchy", Scopes::LandedTitle),
    (Scopes::Dynasty, "dynasty_founder", Scopes::Character),
    (Scopes::None, "dummy_female", Scopes::Character),
    (Scopes::None, "dummy_male", Scopes::Character),
    (Scopes::Dynasty, "dynast", Scopes::Character),
    (Scopes::Character, "dynasty", Scopes::Dynasty),
    (Scopes::LandedTitle.union(Scopes::Province), "empire", Scopes::LandedTitle),
    (Scopes::Character, "employer", Scopes::Character),
    (Scopes::CombatSide, "enemy_side", Scopes::CombatSide),
    (Scopes::EpidemicType, "epidemic_trait", Scopes::Trait),
    (Scopes::Epidemic, "epidemic_type", Scopes::EpidemicType),
    (Scopes::Faction, "faction_leader", Scopes::Character),
    (Scopes::Faction, "faction_target", Scopes::Character),
    (Scopes::Faction, "faction_war", Scopes::War),
    (
        Scopes::Character
            .union(Scopes::LandedTitle)
            .union(Scopes::Province)
            .union(Scopes::GreatHolyWar),
        "faith",
        Scopes::Faith,
    ),
    (Scopes::Character, "father", Scopes::Character),
    (Scopes::TravelPlan, "final_destination_province", Scopes::Province),
    (Scopes::Faith, "founder", Scopes::Character),
    (Scopes::Accolade, "founder_culture", Scopes::Culture),
    (Scopes::Accolade, "founder_dynasty", Scopes::Dynasty),
    (Scopes::Accolade, "founder_faith", Scopes::Faith),
    (Scopes::Accolade, "founder_house", Scopes::DynastyHouse),
    (Scopes::Character, "ghw_beneficiary", Scopes::Character),
    (Scopes::GreatHolyWar, "ghw_designated_winner", Scopes::Character),
    (Scopes::GreatHolyWar, "ghw_target_character", Scopes::Character),
    (Scopes::GreatHolyWar, "ghw_target_title", Scopes::LandedTitle),
    (Scopes::GreatHolyWar, "ghw_title_recipient", Scopes::Character),
    (Scopes::GreatHolyWar, "ghw_war", Scopes::War),
    (Scopes::GreatHolyWar, "ghw_war_declarer", Scopes::Character),
    (Scopes::Character, "government_type", Scopes::GovernmentType),
    (Scopes::Faith, "great_holy_war", Scopes::GreatHolyWar),
    (Scopes::LandedTitle, "holder", Scopes::Character),
    (Scopes::Character, "holding_type", Scopes::HoldingType),
    (Scopes::HolyOrder, "holy_order_patron", Scopes::Character),
    (Scopes::Character, "home_court", Scopes::Character),
    (Scopes::Character, "host", Scopes::Character),
    (Scopes::Character, "house", Scopes::DynastyHouse),
    (Scopes::DynastyHouse, "house_founder", Scopes::Character),
    (Scopes::DynastyHouse, "house_head", Scopes::Character),
    (Scopes::Character, "imprisoner", Scopes::Character),
    (Scopes::Character, "inspiration", Scopes::Inspiration),
    (Scopes::Inspiration, "inspiration_owner", Scopes::Character),
    (Scopes::Inspiration, "inspiration_sponsor", Scopes::Character),
    (Scopes::Character, "intent_target", Scopes::Character),
    (Scopes::Character, "involved_activity", Scopes::Activity),
    (Scopes::Army, "involved_combat_side", Scopes::CombatSide),
    (Scopes::Character, "joined_faction", Scopes::Faction),
    (Scopes::Character, "killer", Scopes::Character),
    (Scopes::LandedTitle.union(Scopes::Province), "kingdom", Scopes::LandedTitle),
    (Scopes::Character, "knight_army", Scopes::Army),
    (Scopes::DynastyHouse, "last_house_head", Scopes::Character),
    (Scopes::Character, "last_played_character", Scopes::Character),
    (Scopes::HolyOrder, "leader", Scopes::Character),
    (Scopes::Legend, "legend_owner", Scopes::Character),
    (Scopes::Legend, "legend_protagonist", Scopes::Character),
    (Scopes::Legend, "legend_type", Scopes::LegendType),
    (Scopes::LandedTitle, "lessee", Scopes::Character),
    (Scopes::LandedTitle, "lessee_title", Scopes::LandedTitle),
    (Scopes::Character, "liege", Scopes::Character),
    (Scopes::Character, "liege_or_court_owner", Scopes::Character),
    (Scopes::Character.union(Scopes::Combat).union(Scopes::Army), "location", Scopes::Province),
    (Scopes::Character, "matchmaker", Scopes::Character),
    (Scopes::MercenaryCompany, "mercenary_company_leader", Scopes::Character),
    (Scopes::CharacterMemory, "memory_owner", Scopes::Character),
    (Scopes::Character, "mother", Scopes::Character),
    // named_script_value special
    (Scopes::TravelPlan, "next_destination_province", Scopes::Province),
    (Scopes::TravelPlan, "next_location", Scopes::Province),
    (Scopes::None, "no", Scopes::Bool),
    (Scopes::Character, "obedience_target", Scopes::Character),
    (Scopes::Epidemic, "outbreak_province", Scopes::Province),
    (Scopes::Character, "overlord", Scopes::Character),
    (Scopes::Domicile, "owner", Scopes::Character),
    (Scopes::SituationParticipantGroup, "participant_group_situation", Scopes::Situation),
    (Scopes::SituationParticipantGroup, "participant_group_sub_region", Scopes::SituationSubRegion),
    (Scopes::Character, "player_heir", Scopes::Character),
    (Scopes::Character, "pregnancy_assumed_father", Scopes::Character),
    (Scopes::Character, "pregnancy_real_father", Scopes::Character),
    // "prev" special
    (Scopes::LandedTitle, "previous_holder", Scopes::Character),
    (Scopes::Artifact, "previous_owner", Scopes::Character),
    (Scopes::Artifact, "previous_owner_level_2", Scopes::Character),
    (Scopes::Artifact, "previous_owner_level_3", Scopes::Character),
    (Scopes::War.union(Scopes::CasusBelli), "primary_attacker", Scopes::Character),
    (Scopes::War.union(Scopes::CasusBelli), "primary_defender", Scopes::Character),
    (Scopes::Character, "primary_heir", Scopes::Character),
    (Scopes::Character, "primary_partner", Scopes::Character),
    (Scopes::Character, "primary_spouse", Scopes::Character),
    (Scopes::Character, "primary_title", Scopes::LandedTitle),
    (Scopes::Accolade, "primary_type", Scopes::AccoladeType),
    (Scopes::Character, "promoted_legend", Scopes::Legend),
    (Scopes::Province, "province_owner", Scopes::Character),
    (Scopes::Character, "real_father", Scopes::Character),
    (Scopes::Character, "real_mother", Scopes::Character),
    (Scopes::Character, "realm_priest", Scopes::Character),
    (
        Scopes::Character
            .union(Scopes::LandedTitle)
            .union(Scopes::Province)
            .union(Scopes::Faith)
            .union(Scopes::GreatHolyWar),
        "religion",
        Scopes::Religion,
    ),
    (Scopes::Faith, "religious_head", Scopes::Character),
    (Scopes::Faith, "religious_head_title", Scopes::LandedTitle),
    (Scopes::Regiment, "regiment_controller", Scopes::Character),
    (Scopes::Regiment, "regiment_controlling_title", Scopes::LandedTitle),
    (Scopes::Regiment, "regiment_owner", Scopes::Character),
    (Scopes::Regiment, "regiment_owning_title", Scopes::LandedTitle),
    (Scopes::Regiment, "regiment_station", Scopes::Province),
    // "root" special
    (Scopes::TaskContract, "scheme", Scopes::Scheme),
    (Scopes::Scheme, "scheme_artifact", Scopes::Artifact),
    (Scopes::Scheme, "scheme_defender", Scopes::Character),
    (Scopes::Scheme, "scheme_owner", Scopes::Character),
    (Scopes::Scheme, "scheme_target_character", Scopes::Character),
    (Scopes::Scheme, "scheme_target_culture", Scopes::Culture),
    (Scopes::Scheme, "scheme_target_faith", Scopes::Faith),
    (Scopes::Scheme, "scheme_target_title", Scopes::LandedTitle),
    (Scopes::Accolade, "secondary_type", Scopes::AccoladeType),
    (Scopes::Character, "secret_faith", Scopes::Faith),
    (Scopes::Secret, "secret_owner", Scopes::Character),
    (Scopes::Secret, "secret_target", Scopes::Character),
    (Scopes::CombatSide, "side_commander", Scopes::Character),
    (Scopes::CombatSide, "side_primary_participant", Scopes::Character),
    (Scopes::Situation, "situation_top_gold", Scopes::Character),
    (Scopes::Situation, "situation_top_herd", Scopes::Character),
    (Scopes::Situation, "situation_top_provisions", Scopes::Character),
    (Scopes::Situation, "situation_top_sub_region", Scopes::SituationSubRegion),
    (Scopes::AgentSlot, "slot_character", Scopes::Character),
    (Scopes::Faction, "special_character", Scopes::Character),
    (Scopes::Faction, "special_title", Scopes::LandedTitle),
    (Scopes::LandedTitle, "state_faith", Scopes::Faith),
    (Scopes::StoryCycle, "story_owner", Scopes::Character),
    (Scopes::VassalObligationLevel, "subject_contract_type", Scopes::VassalContract),
    (Scopes::Character, "suzerain", Scopes::Character),
    (Scopes::Scheme, "task_contract", Scopes::TaskContract),
    (Scopes::TaskContract, "task_contract_destination", Scopes::Province),
    (Scopes::TaskContract, "task_contract_employer", Scopes::Character),
    (Scopes::TaskContract, "task_contract_location", Scopes::Province),
    (Scopes::TaskContract, "task_contract_taker", Scopes::Character),
    (Scopes::TaskContract, "task_contract_target", Scopes::Character),
    // "this" special
    (Scopes::TaxSlot, "tax_collector", Scopes::Character),
    (Scopes::Character, "tax_slot", Scopes::TaxSlot),
    (Scopes::TaxSlot, "tax_slot_liege", Scopes::Character),
    (Scopes::HolyOrder, "title", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_capital_county", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_domicile", Scopes::Domicile),
    (Scopes::LandedTitle, "title_province", Scopes::Province),
    (Scopes::TravelPlan, "travel_leader", Scopes::Character),
    (Scopes::TravelPlan, "travel_plan_activity", Scopes::Activity),
    (Scopes::TravelPlan, "travel_plan_owner", Scopes::Character),
    (Scopes::Character, "top_liege", Scopes::Character),
    (Scopes::Character, "top_overlord", Scopes::Character),
    (Scopes::Character, "top_suzerain", Scopes::Character),
    // "value" special
    (Scopes::VassalObligationLevel, "vassal_contract_type", Scopes::VassalContract),
    (Scopes::Character, "vassal_tax_collector", Scopes::Character),
    (Scopes::CasusBelli, "war", Scopes::War),
    (Scopes::Character, "warden", Scopes::Character),
    (Scopes::None, "yes", Scopes::Bool),
];

#[inline]
pub fn scope_prefix(name: &str) -> Option<(Scopes, Scopes, ArgumentValue)> {
    SCOPE_PREFIX_MAP.get(name).copied()
}

static SCOPE_PREFIX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to, argument) in SCOPE_PREFIX.iter().copied() {
            hash.insert(s, (from, to, argument));
        }
        hash
    });

/// LAST UPDATED CK3 VERSION 1.16.0
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_PREFIX: &[(Scopes, &str, Scopes, ArgumentValue)] = {
    use crate::item::Item;
    use crate::scopes::ArgumentValue::*;
    &[
        (Scopes::None, "accolade_type", Scopes::AccoladeType, Item(Item::AccoladeType)),
        (Scopes::None, "activity_type", Scopes::ActivityType, Item(Item::ActivityType)),
        (Scopes::Character, "aptitude", Scopes::Value, Item(Item::CourtPosition)),
        (Scopes::Character, "aptitude_score", Scopes::Value, Item(Item::CourtPosition)),
        (Scopes::None, "array_define", Scopes::Value, UncheckedValue),
        (Scopes::None, "casus_belli_type", Scopes::CasusBelliType, Item(Item::CasusBelli)),
        (Scopes::None, "character", Scopes::Character, Item(Item::Character)),
        (
            Scopes::SituationSubRegion,
            "character_participant_group",
            Scopes::SituationParticipantGroup,
            Scope(Scopes::Character),
        ),
        (
            Scopes::Situation,
            "character_top_participant_group",
            Scopes::SituationParticipantGroup,
            Scope(Scopes::Character),
        ),
        (Scopes::None, "contract_type", Scopes::VassalContract, Item(Item::SubjectContract)),
        (Scopes::Character, "council_task", Scopes::CouncilTask, Item(Item::CouncilPosition)),
        (Scopes::Character, "court_position", Scopes::Character, Item(Item::CourtPosition)),
        (Scopes::None, "court_position_type", Scopes::CourtPositionType, Item(Item::CourtPosition)),
        (Scopes::Character, "cp", Scopes::Character, Item(Item::CouncilPosition)), // councillor
        (Scopes::None, "culture", Scopes::Culture, Item(Item::Culture)),
        (Scopes::None, "culture_pillar", Scopes::CulturePillar, Item(Item::CulturePillar)),
        (Scopes::None, "culture_tradition", Scopes::CultureTradition, Item(Item::CultureTradition)),
        (Scopes::Character, "dead_var", Scopes::all(), UncheckedValue),
        (Scopes::None, "decision", Scopes::Decision, Item(Item::Decision)),
        (Scopes::None, "define", Scopes::Value, UncheckedValue),
        (Scopes::None, "doctrine", Scopes::Doctrine, Item(Item::Doctrine)),
        (Scopes::None, "dynasty", Scopes::Dynasty, Item(Item::Dynasty)),
        (Scopes::None, "epidemic_type", Scopes::EpidemicType, Item(Item::EpidemicType)),
        (Scopes::None, "event_id", Scopes::Flag, Item(Item::Event)),
        (Scopes::None, "faith", Scopes::Faith, Item(Item::Faith)),
        (Scopes::None, "flag", Scopes::Flag, UncheckedValue),
        (Scopes::None, "geographical_region", Scopes::GeographicalRegion, Item(Item::Region)),
        (Scopes::None, "global_var", Scopes::all(), UncheckedValue),
        (Scopes::None, "government_type", Scopes::GovernmentType, Item(Item::GovernmentType)),
        (Scopes::None, "holding_type", Scopes::HoldingType, Item(Item::GovernmentType)),
        (Scopes::None, "house", Scopes::DynastyHouse, Item(Item::House)),
        (Scopes::Legend, "legend_property", Scopes::all(), Item(Item::LegendProperty)),
        (Scopes::None, "legend_type", Scopes::LegendType, Item(Item::LegendType)),
        (Scopes::None, "list_size", Scopes::Value, UncheckedValue),
        (Scopes::None, "local_var", Scopes::all(), UncheckedValue),
        (
            Scopes::Character,
            "mandate_type_qualification",
            Scopes::Value,
            Item(Item::DiarchyMandate),
        ),
        (
            Scopes::Character,
            "max_number_maa_soldiers_of_base_type",
            Scopes::Value,
            Item(Item::MenAtArmsBase),
        ),
        (
            Scopes::Character,
            "max_number_maa_soldiers_of_type",
            Scopes::Value,
            Item(Item::MenAtArms),
        ),
        (Scopes::CharacterMemory, "memory_participant", Scopes::Character, UncheckedValue),
        (
            Scopes::Culture,
            "num_discovered_innovations_in_era",
            Scopes::Value,
            Item(Item::CultureEra),
        ),
        (
            Scopes::Character,
            "number_maa_regiments_of_base_type",
            Scopes::Value,
            Item(Item::MenAtArmsBase),
        ),
        (Scopes::Character, "number_maa_regiments_of_type", Scopes::Value, Item(Item::MenAtArms)),
        (
            Scopes::Character,
            "number_maa_soldiers_of_base_type",
            Scopes::Value,
            Item(Item::MenAtArmsBase),
        ),
        (Scopes::Character, "number_maa_soldiers_of_type", Scopes::Value, Item(Item::MenAtArms)),
        (Scopes::None, "province", Scopes::Province, Item(Item::Province)),
        (Scopes::None, "religion", Scopes::Religion, Item(Item::Religion)),
        (Scopes::None, "scope", Scopes::all(), UncheckedValue),
        // TODO: "only available if the situation has is_unique = yes"
        (Scopes::None, "situation", Scopes::Situation, Item(Item::Situation)),
        (
            Scopes::Situation,
            "situation_participant_group",
            Scopes::SituationParticipantGroup,
            Item(Item::SituationParticipantGroup),
        ),
        (
            Scopes::Situation,
            "situation_sub_region",
            Scopes::SituationSubRegion,
            Item(Item::SituationSubRegion),
        ),
        (Scopes::Activity, "special_guest", Scopes::Character, Item(Item::SpecialGuest)),
        (Scopes::None, "struggle", Scopes::Struggle, Item(Item::Struggle)),
        (
            Scopes::SituationSubRegion,
            "sub_region_participant_group",
            Scopes::SituationParticipantGroup,
            Item(Item::SituationParticipantGroup),
        ),
        (
            Scopes::None,
            "task_contract_type",
            Scopes::TaskContractType,
            Item(Item::TaskContractType),
        ),
        (Scopes::Character, "tax_collector_aptitude", Scopes::Value, Item(Item::TaxSlotType)),
        (Scopes::None, "title", Scopes::LandedTitle, Item(Item::Title)),
        (Scopes::None, "trait", Scopes::Trait, Item(Item::Trait)),
        (Scopes::all(), "var", Scopes::all(), UncheckedValue),
        (Scopes::None, "vassal_contract", Scopes::VassalContract, Item(Item::SubjectContract)),
        (
            Scopes::Character,
            "vassal_contract_obligation_level",
            Scopes::Value,
            Item(Item::SubjectContract),
        ),
    ]
};

pub fn scope_to_scope_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_TO_SCOPE_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[
    ("activity", "1.9", ""),
    ("activity_owner", "1.9", "replaced by `activity_host`"),
    ("activity_province", "1.9", "replaced by `activity_location`"),
    ("scheme_target", "1.13", "replaced by `scheme_target_character`"),
];
