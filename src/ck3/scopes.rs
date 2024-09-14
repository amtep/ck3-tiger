#![allow(non_upper_case_globals)]

use std::fmt::Formatter;

use once_cell::sync::Lazy;

use crate::everything::Everything;
use crate::helpers::{display_choices, TigerHashMap};
use crate::scopes::{ArgumentValue, Scopes};

// LAST UPDATED CK3 VERSION 1.12.1
pub fn scope_from_snake_case(s: &str) -> Option<Scopes> {
    Some(match s {
        "none" => Scopes::None,
        "value" => Scopes::Value,
        "bool" => Scopes::Bool,
        "flag" => Scopes::Flag,
        "character" => Scopes::Character,
        "landed_title" => Scopes::LandedTitle,
        "activity" => Scopes::Activity,
        "secret" => Scopes::Secret,
        "province" => Scopes::Province,
        "scheme" => Scopes::Scheme,
        "combat" => Scopes::Combat,
        "combat_side" => Scopes::CombatSide,
        "title_and_vassal_change" => Scopes::TitleAndVassalChange,
        "faith" => Scopes::Faith,
        "ghw" => Scopes::GreatHolyWar, // Warning, this is an exception to the general rule
        "religion" => Scopes::Religion,
        "war" => Scopes::War,
        "story" => Scopes::StoryCycle, // Another exception
        "casus_belli" => Scopes::CasusBelli,
        "dynasty" => Scopes::Dynasty,
        "dynasty_house" => Scopes::DynastyHouse,
        "faction" => Scopes::Faction,
        "culture" => Scopes::Culture,
        "army" => Scopes::Army,
        "holy_order" => Scopes::HolyOrder,
        "council_task" => Scopes::CouncilTask,
        "mercenary_company" => Scopes::MercenaryCompany,
        "artifact" => Scopes::Artifact,
        "inspiration" => Scopes::Inspiration,
        "struggle" => Scopes::Struggle,
        "character_memory" => Scopes::CharacterMemory,
        "travel_plan" => Scopes::TravelPlan,
        "accolade" => Scopes::Accolade,
        "accolade_type" => Scopes::AccoladeType,
        "decision" => Scopes::Decision,
        "doctrine" => Scopes::Doctrine,
        "activity_type" => Scopes::ActivityType,
        "culture_tradition" => Scopes::CultureTradition,
        "culture_pillar" => Scopes::CulturePillar,
        "government_type" => Scopes::GovernmentType,
        "holding_type" => Scopes::HoldingType,
        "trait" => Scopes::Trait,
        "tax_slot" => Scopes::TaxSlot,
        "vassal_contract" => Scopes::VassalContract,
        "vassal_contract_obligation_level" => Scopes::VassalObligationLevel,
        "epidemic_type" => Scopes::EpidemicType,
        "epidemic" => Scopes::Epidemic,
        "legend_type" => Scopes::LegendType,
        "legend" => Scopes::Legend,
        "geographical_region" => Scopes::GeographicalRegion,
        "domicile" => Scopes::Domicile,
        "agent_slot" => Scopes::AgentSlot,
        "task_contract" => Scopes::TaskContract,
        "task_contract_type" => Scopes::TaskContractType,
        "regiment" => Scopes::Regiment,
        "casus_belli_type" => Scopes::CasusBelliType,
        _ => return std::option::Option::None,
    })
}

// LAST UPDATED CK3 VERSION 1.12.1
pub fn display_fmt(s: Scopes, f: &mut Formatter) -> Result<(), std::fmt::Error> {
    let mut vec = Vec::new();
    if s.contains(Scopes::None) {
        vec.push("none");
    }
    if s.contains(Scopes::Value) {
        vec.push("value");
    }
    if s.contains(Scopes::Bool) {
        vec.push("bool");
    }
    if s.contains(Scopes::Flag) {
        vec.push("flag");
    }
    if s.contains(Scopes::Character) {
        vec.push("character");
    }
    if s.contains(Scopes::LandedTitle) {
        vec.push("landed title");
    }
    if s.contains(Scopes::Activity) {
        vec.push("activity");
    }
    if s.contains(Scopes::Secret) {
        vec.push("secret");
    }
    if s.contains(Scopes::Province) {
        vec.push("province");
    }
    if s.contains(Scopes::Scheme) {
        vec.push("scheme");
    }
    if s.contains(Scopes::Combat) {
        vec.push("combat");
    }
    if s.contains(Scopes::CombatSide) {
        vec.push("combat side");
    }
    if s.contains(Scopes::TitleAndVassalChange) {
        vec.push("title and vassal change");
    }
    if s.contains(Scopes::Faith) {
        vec.push("faith");
    }
    if s.contains(Scopes::GreatHolyWar) {
        vec.push("great holy war");
    }
    if s.contains(Scopes::Religion) {
        vec.push("religion");
    }
    if s.contains(Scopes::War) {
        vec.push("war");
    }
    if s.contains(Scopes::StoryCycle) {
        vec.push("story cycle");
    }
    if s.contains(Scopes::CasusBelli) {
        vec.push("casus belli");
    }
    if s.contains(Scopes::Dynasty) {
        vec.push("dynasty");
    }
    if s.contains(Scopes::DynastyHouse) {
        vec.push("dynasty house");
    }
    if s.contains(Scopes::Faction) {
        vec.push("faction");
    }
    if s.contains(Scopes::Culture) {
        vec.push("culture");
    }
    if s.contains(Scopes::Army) {
        vec.push("army");
    }
    if s.contains(Scopes::HolyOrder) {
        vec.push("holy order");
    }
    if s.contains(Scopes::CouncilTask) {
        vec.push("council task");
    }
    if s.contains(Scopes::MercenaryCompany) {
        vec.push("mercenary company");
    }
    if s.contains(Scopes::Artifact) {
        vec.push("artifact");
    }
    if s.contains(Scopes::Inspiration) {
        vec.push("inspiration");
    }
    if s.contains(Scopes::Struggle) {
        vec.push("struggle");
    }
    if s.contains(Scopes::CharacterMemory) {
        vec.push("character memory");
    }
    if s.contains(Scopes::TravelPlan) {
        vec.push("travel plan");
    }
    if s.contains(Scopes::Accolade) {
        vec.push("accolade");
    }
    if s.contains(Scopes::AccoladeType) {
        vec.push("accolade type");
    }
    if s.contains(Scopes::Decision) {
        vec.push("decision");
    }
    if s.contains(Scopes::Doctrine) {
        vec.push("doctrine");
    }
    if s.contains(Scopes::ActivityType) {
        vec.push("activity type");
    }
    if s.contains(Scopes::CultureTradition) {
        vec.push("culture tradition");
    }
    if s.contains(Scopes::CulturePillar) {
        vec.push("culture pillar");
    }
    if s.contains(Scopes::GovernmentType) {
        vec.push("government type");
    }
    if s.contains(Scopes::HoldingType) {
        vec.push("holding type");
    }
    if s.contains(Scopes::Trait) {
        vec.push("trait");
    }
    if s.contains(Scopes::TaxSlot) {
        vec.push("tax slot");
    }
    if s.contains(Scopes::VassalContract) {
        vec.push("vassal contract");
    }
    if s.contains(Scopes::VassalObligationLevel) {
        vec.push("vassal obligation level");
    }
    if s.contains(Scopes::EpidemicType) {
        vec.push("epidemic type");
    }
    if s.contains(Scopes::Epidemic) {
        vec.push("epidemic");
    }
    if s.contains(Scopes::LegendType) {
        vec.push("legend type");
    }
    if s.contains(Scopes::Legend) {
        vec.push("legend");
    }
    if s.contains(Scopes::GeographicalRegion) {
        vec.push("geographical region");
    }
    if s.contains(Scopes::Domicile) {
        vec.push("domicile");
    }
    if s.contains(Scopes::AgentSlot) {
        vec.push("agent slot");
    }
    if s.contains(Scopes::TaskContract) {
        vec.push("task contract");
    }
    if s.contains(Scopes::TaskContractType) {
        vec.push("task contract type");
    }
    if s.contains(Scopes::Regiment) {
        vec.push("regiment");
    }
    if s.contains(Scopes::CasusBelliType) {
        vec.push("casus belli type");
    }
    display_choices(f, &vec, "or")
}

// LAST UPDATED CK3 VERSION 1.12.1
pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    use crate::item::Item;
    if scopes == Scopes::AccoladeType && data.item_exists(Item::AccoladeType, arg) {
        return Some("accolade_type");
    }
    if scopes == Scopes::ActivityType && data.item_exists(Item::ActivityType, arg) {
        return Some("activity_type");
    }
    if scopes == Scopes::Character && data.item_exists(Item::Character, arg) {
        return Some("character");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("culture");
    }
    if scopes == Scopes::CulturePillar && data.item_exists(Item::CulturePillar, arg) {
        return Some("culture_pillar");
    }
    if scopes == Scopes::CultureTradition && data.item_exists(Item::CultureTradition, arg) {
        return Some("culture_tradition");
    }
    if scopes == Scopes::Decision && data.item_exists(Item::Decision, arg) {
        return Some("decision");
    }
    if scopes == Scopes::Doctrine && data.item_exists(Item::Doctrine, arg) {
        return Some("doctrine");
    }
    if scopes == Scopes::Dynasty && data.item_exists(Item::Dynasty, arg) {
        return Some("dynasty");
    }
    if scopes == Scopes::EpidemicType && data.item_exists(Item::EpidemicType, arg) {
        return Some("epidemic_type");
    }
    if scopes == Scopes::Faith && data.item_exists(Item::Faith, arg) {
        return Some("faith");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::GeographicalRegion && data.item_exists(Item::Region, arg) {
        return Some("geographical_region");
    }
    if scopes == Scopes::GovernmentType && data.item_exists(Item::GovernmentType, arg) {
        return Some("government_type");
    }
    if scopes == Scopes::HoldingType && data.item_exists(Item::HoldingType, arg) {
        return Some("holding_type");
    }
    if scopes == Scopes::DynastyHouse && data.item_exists(Item::House, arg) {
        return Some("house");
    }
    if scopes == Scopes::LegendType && data.item_exists(Item::LegendType, arg) {
        return Some("legend_type");
    }
    if scopes == Scopes::Province && data.item_exists(Item::Province, arg) {
        return Some("province");
    }
    if scopes == Scopes::Religion && data.item_exists(Item::Religion, arg) {
        return Some("religion");
    }
    if scopes == Scopes::Struggle && data.item_exists(Item::Struggle, arg) {
        return Some("struggle");
    }
    if scopes == Scopes::LandedTitle && data.item_exists(Item::Title, arg) {
        return Some("title");
    }
    if scopes == Scopes::VassalContract && data.item_exists(Item::VassalContract, arg) {
        return Some("vassal_contract");
    }
    None
}

#[inline]
pub fn scope_to_scope(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_TO_SCOPE_MAP.get(name).copied()
}

static SCOPE_TO_SCOPE_MAP: Lazy<TigerHashMap<&'static str, (Scopes, Scopes)>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in SCOPE_TO_SCOPE.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED CK3 VERSION 1.12.1
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
    (Scopes::Faith, "great_holy_war", Scopes::GreatHolyWar),
    (Scopes::LandedTitle, "holder", Scopes::Character),
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
    (Scopes::Epidemic, "outbreak_province", Scopes::Province),
    (Scopes::Domicile, "owner", Scopes::Character),
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
    (Scopes::AgentSlot, "slot_character", Scopes::Character),
    (Scopes::Faction, "special_character", Scopes::Character),
    (Scopes::Faction, "special_title", Scopes::LandedTitle),
    (Scopes::LandedTitle, "state_faith", Scopes::Faith),
    (Scopes::StoryCycle, "story_owner", Scopes::Character),
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
    (Scopes::LandedTitle, "title_province", Scopes::Province),
    (Scopes::TravelPlan, "travel_leader", Scopes::Character),
    (Scopes::TravelPlan, "travel_plan_activity", Scopes::Activity),
    (Scopes::TravelPlan, "travel_plan_owner", Scopes::Character),
    (Scopes::Character, "top_liege", Scopes::Character),
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

static SCOPE_PREFIX_MAP: Lazy<TigerHashMap<&'static str, (Scopes, Scopes, ArgumentValue)>> =
    Lazy::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to, argument) in SCOPE_PREFIX.iter().copied() {
            hash.insert(s, (from, to, argument));
        }
        hash
    });

/// LAST UPDATED CK3 VERSION 1.12.1
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
        (Scopes::Character, "council_task", Scopes::CouncilTask, Item(Item::CouncilPosition)),
        (Scopes::Character, "court_position", Scopes::Character, Item(Item::CourtPosition)),
        (Scopes::Character, "cp", Scopes::Character, Item(Item::CouncilPosition)), // councillor
        (Scopes::None, "culture", Scopes::Culture, Item(Item::Culture)),
        (Scopes::None, "culture_pillar", Scopes::CulturePillar, Item(Item::CulturePillar)),
        (Scopes::None, "culture_tradition", Scopes::CultureTradition, Item(Item::CultureTradition)),
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
        (Scopes::CharacterMemory, "memory_participant", Scopes::Character, UncheckedValue),
        (
            Scopes::Culture,
            "num_discovered_innovations_in_era",
            Scopes::Value,
            Item(Item::CultureEra),
        ),
        (Scopes::None, "province", Scopes::Province, Item(Item::Province)),
        (Scopes::None, "religion", Scopes::Religion, Item(Item::Religion)),
        (Scopes::None, "scope", Scopes::all(), UncheckedValue),
        (Scopes::Activity, "special_guest", Scopes::Character, Item(Item::SpecialGuest)),
        (Scopes::None, "struggle", Scopes::Struggle, Item(Item::Struggle)),
        (
            Scopes::None,
            "task_contract_type",
            Scopes::TaskContractType,
            Item(Item::TaskContractType),
        ),
        (Scopes::None, "title", Scopes::LandedTitle, Item(Item::Title)),
        (Scopes::None, "trait", Scopes::Trait, Item(Item::Trait)),
        (Scopes::all(), "var", Scopes::all(), UncheckedValue),
        (Scopes::None, "vassal_contract", Scopes::VassalContract, Item(Item::VassalContract)),
        (
            Scopes::Character,
            "vassal_contract_obligation_level",
            Scopes::Value,
            Item(Item::VassalContract),
        ),
    ]
};

#[inline]
pub fn scope_iterator(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_ITERATOR_MAP.get(name).copied()
}

static SCOPE_ITERATOR_MAP: Lazy<TigerHashMap<&'static str, (Scopes, Scopes)>> = Lazy::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in SCOPE_ITERATOR.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED CK3 VERSION 1.12.1
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Character, "acclaimed_knight", Scopes::Character),
    (Scopes::Character, "accolade", Scopes::Accolade),
    (Scopes::None, "accolade_type", Scopes::AccoladeType),
    (Scopes::Character, "active_accolade", Scopes::Accolade),
    (Scopes::None, "activity", Scopes::Activity),
    (Scopes::Activity, "activity_phase_location", Scopes::Province),
    (Scopes::Activity, "activity_phase_location_future", Scopes::Province),
    (Scopes::Activity, "activity_phase_location_past", Scopes::Province),
    (Scopes::None, "activity_type", Scopes::ActivityType),
    (Scopes::Character, "alert_creatable_title", Scopes::LandedTitle),
    (Scopes::Character, "alert_usurpable_title", Scopes::LandedTitle),
    (Scopes::Character, "ally", Scopes::Character),
    (Scopes::Character, "ancestor", Scopes::Character),
    (Scopes::Character, "army", Scopes::Army),
    (Scopes::Province, "army_in_location", Scopes::Army),
    (Scopes::None, "artifact", Scopes::Artifact),
    (Scopes::Artifact, "artifact_claimant", Scopes::Character),
    (Scopes::Artifact, "artifact_house_claimant", Scopes::DynastyHouse),
    (Scopes::Activity, "attending_character", Scopes::Character),
    (Scopes::None, "barony", Scopes::LandedTitle),
    (Scopes::Character, "character_artifact", Scopes::Artifact),
    (Scopes::Character, "character_epidemic", Scopes::Epidemic),
    (Scopes::Province, "character_in_location", Scopes::Character),
    (Scopes::Character, "character_struggle", Scopes::Struggle),
    (
        Scopes::Character,
        "character_to_title_neighboring_and_across_water_county",
        Scopes::LandedTitle,
    ),
    (
        Scopes::Character,
        "character_to_title_neighboring_and_across_water_duchy",
        Scopes::LandedTitle,
    ),
    (
        Scopes::Character,
        "character_to_title_neighboring_and_across_water_empire",
        Scopes::LandedTitle,
    ),
    (
        Scopes::Character,
        "character_to_title_neighboring_and_across_water_kingdom",
        Scopes::LandedTitle,
    ),
    (Scopes::Character, "character_to_title_neighboring_county", Scopes::LandedTitle),
    (Scopes::Character, "character_to_title_neighboring_duchy", Scopes::LandedTitle),
    (Scopes::Character, "character_to_title_neighboring_empire", Scopes::LandedTitle),
    (Scopes::Character, "character_to_title_neighboring_kingdom", Scopes::LandedTitle),
    (Scopes::Character, "character_trait", Scopes::Trait),
    (Scopes::Character, "character_war", Scopes::War),
    (Scopes::None, "character_with_royal_court", Scopes::Character),
    (Scopes::Character, "child", Scopes::Character),
    (Scopes::Character, "claim", Scopes::LandedTitle),
    (Scopes::LandedTitle, "claimant", Scopes::Character),
    (Scopes::Character, "claimed_artifact", Scopes::Artifact),
    (Scopes::Character, "close_family_member", Scopes::Character),
    (Scopes::Character, "close_or_extended_family_member", Scopes::Character),
    (Scopes::Combat, "combat_side", Scopes::CombatSide),
    (Scopes::None, "completed_legend", Scopes::Legend),
    (Scopes::Character, "concubine", Scopes::Character),
    (Scopes::LandedTitle, "connected_county", Scopes::LandedTitle),
    (Scopes::Character, "consort", Scopes::Character),
    (Scopes::LandedTitle, "controlled_faith", Scopes::Faith),
    (Scopes::Character, "councillor", Scopes::Character),
    (Scopes::None, "county", Scopes::LandedTitle),
    (Scopes::None, "county_in_region", Scopes::LandedTitle),
    (Scopes::LandedTitle, "county_province", Scopes::Province),
    (Scopes::LandedTitle, "county_struggle", Scopes::Struggle),
    (Scopes::Character, "court_position_employer", Scopes::Character),
    (Scopes::Character, "court_position_holder", Scopes::Character), // TODO find out how court position is supplied
    (Scopes::Character, "courtier", Scopes::Character),
    (Scopes::Character, "courtier_away", Scopes::Character),
    (Scopes::Character, "courtier_or_guest", Scopes::Character),
    (Scopes::Culture, "culture_county", Scopes::LandedTitle),
    (Scopes::Culture, "culture_duchy", Scopes::LandedTitle),
    (Scopes::Culture, "culture_empire", Scopes::LandedTitle),
    (Scopes::None, "culture_global", Scopes::Culture),
    (Scopes::Culture, "culture_kingdom", Scopes::LandedTitle),
    (Scopes::None, "culture_pillar", Scopes::CulturePillar),
    (Scopes::None, "culture_tradition", Scopes::CultureTradition),
    (Scopes::Character, "de_jure_claim", Scopes::LandedTitle),
    (Scopes::LandedTitle, "de_jure_county", Scopes::LandedTitle),
    (Scopes::LandedTitle, "de_jure_county_holder", Scopes::Character),
    (Scopes::LandedTitle, "de_jure_top_liege", Scopes::Character),
    (Scopes::None, "decision", Scopes::Decision),
    (Scopes::Faith, "defensive_great_holy_wars", Scopes::GreatHolyWar),
    (Scopes::LandedTitle, "dejure_vassal_title_holder", Scopes::Character),
    (Scopes::Character, "diarchy_succession_character", Scopes::Character),
    (Scopes::Character, "diplomacy_councillor", Scopes::Character),
    (Scopes::LandedTitle, "direct_de_facto_vassal_title", Scopes::LandedTitle),
    (Scopes::LandedTitle, "direct_de_jure_vassal_title", Scopes::LandedTitle),
    (Scopes::Character, "directly_owned_province", Scopes::Province),
    (Scopes::None, "doctrine", Scopes::Doctrine),
    (Scopes::None, "duchy", Scopes::LandedTitle),
    (Scopes::Dynasty, "dynasty_member", Scopes::Character),
    (Scopes::LandedTitle, "election_candidate", Scopes::Character),
    (Scopes::Character, "election_title", Scopes::LandedTitle),
    (Scopes::LandedTitle, "elector", Scopes::Character),
    (Scopes::None, "empire", Scopes::LandedTitle),
    (Scopes::TravelPlan, "entourage_character", Scopes::Character),
    (Scopes::None, "epidemic", Scopes::Epidemic),
    (Scopes::None, "epidemic_type", Scopes::EpidemicType),
    (Scopes::Character, "equipped_character_artifact", Scopes::Artifact),
    (Scopes::Character, "extended_family_member", Scopes::Character),
    (Scopes::Faction, "faction_county_member", Scopes::LandedTitle),
    (Scopes::Faction, "faction_member", Scopes::Character),
    (Scopes::Religion, "faith", Scopes::Faith),
    (Scopes::Faith, "faith_character", Scopes::Character),
    (Scopes::Faith, "faith_holy_order", Scopes::HolyOrder),
    (Scopes::Faith, "faith_playable_ruler", Scopes::Character),
    (Scopes::Faith, "faith_ruler", Scopes::Character),
    (Scopes::Character, "foreign_court_guest", Scopes::Character),
    (Scopes::Character, "former_concubine", Scopes::Character),
    (Scopes::Character, "former_concubinist", Scopes::Character),
    (Scopes::Character, "former_spouse", Scopes::Character),
    (Scopes::TravelPlan, "future_path_location", Scopes::Province),
    (Scopes::Character, "general_councillor", Scopes::Character),
    (Scopes::None, "geographical_region", Scopes::GeographicalRegion),
    (Scopes::Character, "government_type", Scopes::GovernmentType),
    (Scopes::Activity, "guest_subset", Scopes::Character),
    (Scopes::Activity, "guest_subset_current_phase", Scopes::Character),
    (Scopes::Character, "heir", Scopes::Character),
    // TODO one of these might be reversed
    (Scopes::Character, "heir_title", Scopes::LandedTitle),
    (Scopes::Character, "heir_to_title", Scopes::LandedTitle),
    (Scopes::Character, "held_title", Scopes::LandedTitle),
    (Scopes::Character, "hired_mercenary", Scopes::MercenaryCompany),
    (Scopes::None, "holding_type", Scopes::HoldingType),
    (Scopes::Faith, "holy_site", Scopes::LandedTitle),
    (Scopes::Character, "home_court_hostage", Scopes::Character),
    (Scopes::Character, "hooked_character", Scopes::Character),
    (Scopes::Character, "hostile_raider", Scopes::Character),
    (Scopes::DynastyHouse, "house_claimed_artifact", Scopes::Artifact),
    (Scopes::DynastyHouse, "house_member", Scopes::Character),
    (Scopes::DynastyHouse, "house_unity_member", Scopes::Character),
    (Scopes::LandedTitle, "in_de_facto_hierarchy", Scopes::LandedTitle),
    (Scopes::LandedTitle, "in_de_jure_hierarchy", Scopes::LandedTitle),
    (Scopes::None, "in_global_list", Scopes::all()),
    (Scopes::None, "in_list", Scopes::all()),
    (Scopes::None, "in_local_list", Scopes::all()),
    (Scopes::None, "independent_ruler", Scopes::Character),
    (Scopes::Epidemic, "infected_province", Scopes::Province),
    (Scopes::None, "inspiration", Scopes::Inspiration),
    (Scopes::None, "inspired_character", Scopes::Character),
    (Scopes::Struggle, "interloper_ruler", Scopes::Character),
    (Scopes::Character, "intrigue_councillor", Scopes::Character),
    (Scopes::Character, "invited_activity", Scopes::Activity),
    (Scopes::Activity, "invited_character", Scopes::Character),
    (Scopes::Struggle, "involved_county", Scopes::LandedTitle),
    (Scopes::Struggle, "involved_ruler", Scopes::Character),
    (Scopes::Character.union(Scopes::Artifact), "killed_character", Scopes::Character),
    (Scopes::None, "kingdom", Scopes::LandedTitle),
    (Scopes::Character, "knight", Scopes::Character),
    (Scopes::Character, "known_secret", Scopes::Secret),
    (Scopes::Character, "learning_councillor", Scopes::Character),
    (Scopes::HolyOrder, "leased_title", Scopes::LandedTitle),
    (Scopes::None, "legend", Scopes::Legend),
    (Scopes::Legend, "legend_promoter", Scopes::Character),
    (Scopes::None, "legend_type", Scopes::LegendType),
    (Scopes::Character, "liege_or_above", Scopes::Character),
    (Scopes::None, "living_character", Scopes::Character),
    (Scopes::Character, "martial_councillor", Scopes::Character),
    (Scopes::None, "mercenary_company", Scopes::MercenaryCompany),
    (Scopes::Character, "memory", Scopes::CharacterMemory),
    (Scopes::CharacterMemory, "memory_participant", Scopes::Character),
    (Scopes::Character, "neighboring_and_across_water_realm_same_rank_owner", Scopes::Character),
    (Scopes::Character, "neighboring_and_across_water_top_liege_realm", Scopes::LandedTitle),
    (Scopes::Character, "neighboring_and_across_water_top_liege_realm_owner", Scopes::Character),
    (Scopes::LandedTitle, "neighboring_county", Scopes::LandedTitle),
    (Scopes::Province, "neighboring_province", Scopes::Province),
    (Scopes::Character, "neighboring_realm_same_rank_owner", Scopes::Character),
    (Scopes::Character, "neighboring_top_liege_realm", Scopes::LandedTitle),
    (Scopes::Character, "neighboring_top_liege_realm_owner", Scopes::Character),
    (Scopes::None, "open_invite_activity", Scopes::Activity),
    (Scopes::Character, "opposite_sex_spouse_candidate", Scopes::Character),
    (Scopes::Trait, "opposite_trait", Scopes::Trait),
    (Scopes::Character, "owned_story", Scopes::StoryCycle),
    (Scopes::Character, "parent", Scopes::Character),
    (Scopes::Culture, "parent_culture", Scopes::Culture),
    (Scopes::Culture, "parent_culture_or_above", Scopes::Culture),
    (Scopes::LandedTitle, "past_holder", Scopes::Character),
    (Scopes::LandedTitle, "past_holder_reversed", Scopes::Character),
    (Scopes::Character, "patroned_holy_order", Scopes::HolyOrder),
    (Scopes::Character, "personal_claimed_artifact", Scopes::Artifact),
    (Scopes::Character, "pinned_character", Scopes::Character),
    (Scopes::Character, "pinning_character", Scopes::Character),
    (Scopes::Character, "played_character", Scopes::Character),
    (Scopes::None, "player", Scopes::Character),
    (Scopes::Character, "player_heir", Scopes::Character),
    (Scopes::GreatHolyWar, "pledged_attacker", Scopes::Character),
    (Scopes::GreatHolyWar, "pledged_defender", Scopes::Character),
    (Scopes::None, "pool_character", Scopes::Character),
    (Scopes::Character, "pool_guest", Scopes::Character),
    (Scopes::Character, "potential_marriage_option", Scopes::Character),
    (Scopes::Character, "powerful_vassal", Scopes::Character),
    (Scopes::Character, "pretender_title", Scopes::LandedTitle),
    (Scopes::Character, "primary_war_enemy", Scopes::Character),
    (Scopes::Character, "prisoner", Scopes::Character),
    (Scopes::None, "province", Scopes::Province),
    (Scopes::Province, "province_epidemic", Scopes::Epidemic),
    (Scopes::Province, "province_legend", Scopes::Legend),
    (Scopes::Character, "prowess_councillor", Scopes::Character),
    (Scopes::Character, "raid_target", Scopes::Character),
    (Scopes::Character, "realm_border_county", Scopes::LandedTitle),
    (Scopes::Character, "realm_county", Scopes::LandedTitle),
    (Scopes::Character, "realm_de_jure_duchy", Scopes::LandedTitle),
    (Scopes::Character, "realm_de_jure_empire", Scopes::LandedTitle),
    (Scopes::Character, "realm_de_jure_kingdom", Scopes::LandedTitle),
    (Scopes::Character, "realm_province", Scopes::Province),
    (Scopes::Character, "relation", Scopes::Character), // TODO takes a type
    (Scopes::None, "religion_global", Scopes::Religion),
    (Scopes::None, "ruler", Scopes::Character),
    (Scopes::Character, "same_sex_spouse_candidate", Scopes::Character),
    (Scopes::Character, "scheme", Scopes::Scheme),
    (Scopes::Scheme, "scheme_agent", Scopes::Character),
    (Scopes::Character, "secret", Scopes::Secret),
    (Scopes::Secret, "secret_knower", Scopes::Character),
    (Scopes::Secret, "secret_participant", Scopes::Character),
    (Scopes::Character, "sibling", Scopes::Character),
    (Scopes::CombatSide, "side_commander", Scopes::Character),
    (Scopes::CombatSide, "side_knight", Scopes::Character),
    (Scopes::None, "special_building_province", Scopes::Province),
    (Scopes::Activity, "special_guest", Scopes::Character),
    (Scopes::Character, "sponsored_inspiration", Scopes::Inspiration),
    (Scopes::Character, "spouse", Scopes::Character),
    (Scopes::Character, "spouse_candidate", Scopes::Character),
    (Scopes::Legend, "spread_province", Scopes::Province),
    (Scopes::Character, "stewardship_councillor", Scopes::Character),
    (Scopes::Character, "sub_realm_barony", Scopes::LandedTitle),
    (Scopes::Character, "sub_realm_county", Scopes::LandedTitle),
    (Scopes::Character, "sub_realm_duchy", Scopes::LandedTitle),
    (Scopes::Character, "sub_realm_empire", Scopes::LandedTitle),
    (Scopes::Character, "sub_realm_kingdom", Scopes::LandedTitle),
    (Scopes::Character, "sub_realm_title", Scopes::LandedTitle),
    (Scopes::CasusBelli, "target_title", Scopes::LandedTitle),
    (Scopes::Character, "targeting_faction", Scopes::Faction),
    (Scopes::Character, "targeting_scheme", Scopes::Scheme),
    (Scopes::Character, "targeting_secret", Scopes::Secret),
    (Scopes::Character, "tax_collector", Scopes::Character),
    (Scopes::Character, "tax_collector_vassal", Scopes::Character),
    (Scopes::Character, "tax_slot", Scopes::TaxSlot),
    (Scopes::TaxSlot, "tax_slot_vassal", Scopes::Character),
    (Scopes::LandedTitle, "this_title_or_de_jure_above", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_heir", Scopes::Character),
    (Scopes::LandedTitle, "title_joined_faction", Scopes::Faction),
    (
        Scopes::LandedTitle,
        "title_to_title_neighboring_and_across_water_county",
        Scopes::LandedTitle,
    ),
    (Scopes::LandedTitle, "title_to_title_neighboring_and_across_water_duchy", Scopes::LandedTitle),
    (
        Scopes::LandedTitle,
        "title_to_title_neighboring_and_across_water_empire",
        Scopes::LandedTitle,
    ),
    (
        Scopes::LandedTitle,
        "title_to_title_neighboring_and_across_water_kingdom",
        Scopes::LandedTitle,
    ),
    (Scopes::LandedTitle, "title_to_title_neighboring_county", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_to_title_neighboring_duchy", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_to_title_neighboring_empire", Scopes::LandedTitle),
    (Scopes::LandedTitle, "title_to_title_neighboring_kingdom", Scopes::LandedTitle),
    (Scopes::Character, "top_realm_border_county", Scopes::LandedTitle),
    (Scopes::Culture, "tradition", Scopes::CultureTradition),
    (Scopes::None, "trait", Scopes::Trait),
    (Scopes::None, "trait_in_category", Scopes::Trait),
    (Scopes::Character, "traveling_family_member", Scopes::Character),
    (Scopes::Character, "truce_holder", Scopes::Character),
    (Scopes::Character, "truce_target", Scopes::Character),
    (Scopes::Character, "unassigned_taxpayers", Scopes::Character),
    (Scopes::Character, "unspent_known_secret", Scopes::Secret),
    (Scopes::Character, "vassal", Scopes::Character),
    (Scopes::None, "vassal_contract", Scopes::VassalContract),
    (Scopes::Character, "vassal_or_below", Scopes::Character),
    (Scopes::TravelPlan, "visited_location", Scopes::Province),
    (Scopes::Character, "war_ally", Scopes::Character),
    (Scopes::War, "war_attacker", Scopes::Character),
    (Scopes::War, "war_defender", Scopes::Character),
    (Scopes::Character, "war_enemy", Scopes::Character),
    (Scopes::War, "war_participant", Scopes::Character),
    (Scopes::Character, "warden_hostage", Scopes::Character),
];

pub fn scope_iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

/// Every entry represents a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR_REMOVED: &[(&str, &str, &str)] = &[
    ("activity_declined", "1.9", ""),
    ("activity_invited", "1.9", ""),
    ("participant", "1.9", ""),
];

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
