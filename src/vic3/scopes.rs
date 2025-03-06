#![allow(non_upper_case_globals)]

use std::fmt::Formatter;
use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::{display_choices, TigerHashMap};
use crate::scopes::{ArgumentValue, Scopes};

pub fn scope_from_snake_case(s: &str) -> Option<Scopes> {
    Some(match s {
        "none" => Scopes::None,
        "value" => Scopes::Value,
        "bool" => Scopes::Bool,
        "flag" => Scopes::Flag,
        "color" => Scopes::Color,
        "country" => Scopes::Country,
        "battle" => Scopes::Battle,
        "battle_side" => Scopes::BattleSide,
        "building" => Scopes::Building,
        "building_type" => Scopes::BuildingType,
        "canal_type" => Scopes::CanalType,
        "character" => Scopes::Character,
        "civil_war" => Scopes::CivilWar,
        "combat_unit_type" => Scopes::CombatUnitType,
        "company" => Scopes::Company,
        "company_type" => Scopes::CompanyType,
        "commander_order_type" => Scopes::CommanderOrderType,
        "country_creation" => Scopes::CountryCreation,
        "country_definition" => Scopes::CountryDefinition,
        "country_formation" => Scopes::CountryFormation,
        "cultural_community" => Scopes::CulturalCommunity,
        "culture" => Scopes::Culture,
        "decree" => Scopes::Decree,
        "diplomatic_action" => Scopes::DiplomaticAction,
        "diplomatic_demand" => Scopes::DiplomaticDemand,
        "diplomatic_pact" => Scopes::DiplomaticPact,
        "diplomatic_play" => Scopes::DiplomaticPlay,
        "diplomatic_play_type" => Scopes::DiplomaticPlayType,
        "diplomatic_relations" => Scopes::DiplomaticRelations,
        "diplomatic_catalyst" => Scopes::DiplomaticCatalyst,
        "diplomatic_catalyst_type" => Scopes::DiplomaticCatalystType,
        "diplomatic_catalyst_category" => Scopes::DiplomaticCatalystCategory,
        "front" => Scopes::Front,
        "goods" => Scopes::Goods,
        "harvest_condition" => Scopes::HarvestCondition,
        "harvest_condition_type" => Scopes::HarvestConditionType,
        "hq" => Scopes::Hq,
        "ideology" => Scopes::Ideology,
        "institution" => Scopes::Institution,
        "institution_type" => Scopes::InstitutionType,
        "interest_marker" => Scopes::InterestMarker,
        "interest_group" => Scopes::InterestGroup,
        "interest_group_trait" => Scopes::InterestGroupTrait,
        "interest_group_type" => Scopes::InterestGroupType,
        "journalentry" => Scopes::JournalEntry,
        "law" => Scopes::Law,
        "law_type" => Scopes::LawType,
        "market" => Scopes::Market,
        "market_goods" => Scopes::MarketGoods,
        "military_formation" => Scopes::MilitaryFormation,
        "mobilization_option" => Scopes::MobilizationOption,
        "naval_invasion" => Scopes::NavalInvasion,
        "new_combat_unit" => Scopes::NewCombatUnit,
        "objective" => Scopes::Objective,
        "party" => Scopes::Party,
        "political_lobby" => Scopes::PoliticalLobby,
        "political_lobby_type" => Scopes::PoliticalLobbyType,
        "political_lobby_appeasement" => Scopes::PoliticalLobbyAppeasement,
        "political_movement" => Scopes::PoliticalMovement,
        "political_movement_type" => Scopes::PoliticalMovementType,
        "pop" => Scopes::Pop,
        "pop_type" => Scopes::PopType,
        "power_bloc" => Scopes::PowerBloc,
        "power_bloc_identity" => Scopes::PowerBlocIdentity,
        "power_bloc_principle" => Scopes::PowerBlocPrinciple,
        "power_bloc_principle_group" => Scopes::PowerBlocPrincipleGroup,
        "province" => Scopes::Province,
        "religion" => Scopes::Religion,
        "shipping_lane" => Scopes::ShippingLane,
        "state" => Scopes::State,
        "state_goods" => Scopes::StateGoods,
        "state_region" => Scopes::StateRegion,
        "state_trait" => Scopes::StateTrait,
        "strategic_region" => Scopes::StrategicRegion,
        "sway" => Scopes::Sway,
        "technology" => Scopes::Technology,
        "technology_status" => Scopes::TechnologyStatus,
        "theater" => Scopes::Theater,
        "trade_route" => Scopes::TradeRoute,
        "travel_connection" => Scopes::TravelConnection,
        "travel_connection_definition" => Scopes::TravelConnectionDefinition,
        "travel_node" => Scopes::TravelNode,
        "travel_node_definition" => Scopes::TravelNodeDefinition,
        "war" => Scopes::War,
        _ => return std::option::Option::None,
    })
}

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
    if s.contains(Scopes::Color) {
        vec.push("color");
    }
    if s.contains(Scopes::Country) {
        vec.push("country");
    }
    if s.contains(Scopes::Battle) {
        vec.push("battle");
    }
    if s.contains(Scopes::BattleSide) {
        vec.push("battle side");
    }
    if s.contains(Scopes::Building) {
        vec.push("building");
    }
    if s.contains(Scopes::BuildingType) {
        vec.push("building type");
    }
    if s.contains(Scopes::CanalType) {
        vec.push("canal type");
    }
    if s.contains(Scopes::Character) {
        vec.push("character");
    }
    if s.contains(Scopes::CivilWar) {
        vec.push("civil war");
    }
    if s.contains(Scopes::CombatUnitType) {
        vec.push("combat unit type");
    }
    if s.contains(Scopes::NewCombatUnit) {
        vec.push("new combat unit");
    }
    if s.contains(Scopes::CommanderOrderType) {
        vec.push("commander order type");
    }
    if s.contains(Scopes::Company) {
        vec.push("company");
    }
    if s.contains(Scopes::CompanyType) {
        vec.push("company type");
    }
    if s.contains(Scopes::CountryCreation) {
        vec.push("country creation");
    }
    if s.contains(Scopes::CountryDefinition) {
        vec.push("country definition");
    }
    if s.contains(Scopes::CountryFormation) {
        vec.push("country formation");
    }
    if s.contains(Scopes::CulturalCommunity) {
        vec.push("cultural community");
    }
    if s.contains(Scopes::Culture) {
        vec.push("culture");
    }
    if s.contains(Scopes::Decree) {
        vec.push("decree");
    }
    if s.contains(Scopes::DiplomaticAction) {
        vec.push("diplomatic action");
    }
    if s.contains(Scopes::DiplomaticDemand) {
        vec.push("diplomatic demand");
    }
    if s.contains(Scopes::DiplomaticPact) {
        vec.push("diplomatic pact");
    }
    if s.contains(Scopes::DiplomaticPlay) {
        vec.push("diplomatic play");
    }
    if s.contains(Scopes::DiplomaticPlayType) {
        vec.push("diplomatic play type");
    }
    if s.contains(Scopes::DiplomaticRelations) {
        vec.push("diplomatic relations");
    }
    if s.contains(Scopes::DiplomaticCatalyst) {
        vec.push("diplomatic catalyst");
    }
    if s.contains(Scopes::DiplomaticCatalystType) {
        vec.push("diplomatic catalyst type");
    }
    if s.contains(Scopes::DiplomaticCatalystCategory) {
        vec.push("diplomatic catalyst category");
    }
    if s.contains(Scopes::PoliticalLobby) {
        vec.push("political lobby");
    }
    if s.contains(Scopes::PoliticalLobbyType) {
        vec.push("political lobby type");
    }
    if s.contains(Scopes::PoliticalLobbyAppeasement) {
        vec.push("political lobby appeasement");
    }
    if s.contains(Scopes::Front) {
        vec.push("front");
    }
    if s.contains(Scopes::Goods) {
        vec.push("goods");
    }
    if s.contains(Scopes::HarvestCondition) {
        vec.push("harvest condition");
    }
    if s.contains(Scopes::HarvestConditionType) {
        vec.push("harvest condition type");
    }
    if s.contains(Scopes::Hq) {
        vec.push("hq");
    }
    if s.contains(Scopes::Ideology) {
        vec.push("ideology");
    }
    if s.contains(Scopes::Institution) {
        vec.push("institution");
    }
    if s.contains(Scopes::InstitutionType) {
        vec.push("institution type");
    }
    if s.contains(Scopes::InterestMarker) {
        vec.push("interest marker");
    }
    if s.contains(Scopes::InterestGroup) {
        vec.push("interest group");
    }
    if s.contains(Scopes::InterestGroupTrait) {
        vec.push("interest group_trait");
    }
    if s.contains(Scopes::InterestGroupType) {
        vec.push("interest group_type");
    }
    if s.contains(Scopes::JournalEntry) {
        vec.push("journalentry");
    }
    if s.contains(Scopes::Law) {
        vec.push("law");
    }
    if s.contains(Scopes::LawType) {
        vec.push("law type");
    }
    if s.contains(Scopes::Market) {
        vec.push("market");
    }
    if s.contains(Scopes::MarketGoods) {
        vec.push("market goods");
    }
    if s.contains(Scopes::MilitaryFormation) {
        vec.push("military formation");
    }
    if s.contains(Scopes::MobilizationOption) {
        vec.push("mobilization option");
    }
    if s.contains(Scopes::NavalInvasion) {
        vec.push("naval invasion");
    }
    if s.contains(Scopes::Objective) {
        vec.push("objective");
    }
    if s.contains(Scopes::Party) {
        vec.push("party");
    }
    if s.contains(Scopes::PoliticalMovement) {
        vec.push("political movement");
    }
    if s.contains(Scopes::PoliticalMovementType) {
        vec.push("political movement type");
    }
    if s.contains(Scopes::Pop) {
        vec.push("pop");
    }
    if s.contains(Scopes::PopType) {
        vec.push("pop type");
    }
    if s.contains(Scopes::PowerBloc) {
        vec.push("power bloc");
    }
    if s.contains(Scopes::PowerBlocIdentity) {
        vec.push("power bloc identity");
    }
    if s.contains(Scopes::PowerBlocPrinciple) {
        vec.push("power bloc principle");
    }
    if s.contains(Scopes::PowerBlocPrincipleGroup) {
        vec.push("power bloc principle group");
    }
    if s.contains(Scopes::Province) {
        vec.push("province");
    }
    if s.contains(Scopes::Religion) {
        vec.push("religion");
    }
    if s.contains(Scopes::ShippingLane) {
        vec.push("shipping lane");
    }
    if s.contains(Scopes::State) {
        vec.push("state");
    }
    if s.contains(Scopes::StateGoods) {
        vec.push("state goods");
    }
    if s.contains(Scopes::StateRegion) {
        vec.push("state region");
    }
    if s.contains(Scopes::StateTrait) {
        vec.push("state trait");
    }
    if s.contains(Scopes::StrategicRegion) {
        vec.push("strategic region");
    }
    if s.contains(Scopes::Sway) {
        vec.push("sway");
    }
    if s.contains(Scopes::Technology) {
        vec.push("technology");
    }
    if s.contains(Scopes::TechnologyStatus) {
        vec.push("technology status");
    }
    if s.contains(Scopes::Theater) {
        vec.push("theater");
    }
    if s.contains(Scopes::TradeRoute) {
        vec.push("trade route");
    }
    if s.contains(Scopes::TravelConnection) {
        vec.push("travel connection");
    }
    if s.contains(Scopes::TravelConnectionDefinition) {
        vec.push("travel connection definition");
    }
    if s.contains(Scopes::TravelNode) {
        vec.push("travel node");
    }
    if s.contains(Scopes::TravelNodeDefinition) {
        vec.push("travel node definition");
    }
    if s.contains(Scopes::War) {
        vec.push("war");
    }
    display_choices(f, &vec, "or")
}

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    use crate::item::Item;
    if scopes == Scopes::Building && data.item_exists(Item::BuildingType, arg) {
        return Some("b");
    }
    if scopes == Scopes::BuildingType && data.item_exists(Item::BuildingType, arg) {
        return Some("bt");
    }
    if scopes == Scopes::Country && data.item_exists(Item::Country, arg) {
        return Some("c");
    }
    if scopes == Scopes::CountryDefinition && data.item_exists(Item::Country, arg) {
        return Some("cd");
    }
    if scopes == Scopes::CompanyType && data.item_exists(Item::CompanyType, arg) {
        return Some("company_type");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("cu");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::Ideology && data.item_exists(Item::Ideology, arg) {
        return Some("i");
    }
    if scopes == Scopes::InterestGroup && data.item_exists(Item::InterestGroup, arg) {
        return Some("ig");
    }
    if scopes == Scopes::InterestGroupTrait && data.item_exists(Item::InterestGroupTrait, arg) {
        return Some("ig_trait");
    }
    if scopes == Scopes::InterestGroupType && data.item_exists(Item::InterestGroup, arg) {
        return Some("ig_type");
    }
    if scopes == Scopes::Institution && data.item_exists(Item::Institution, arg) {
        return Some("institution");
    }
    if scopes == Scopes::JournalEntry && data.item_exists(Item::JournalEntry, arg) {
        return Some("je");
    }
    if scopes == Scopes::LawType && data.item_exists(Item::LawType, arg) {
        return Some("law_type");
    }
    if scopes == Scopes::MarketGoods && data.item_exists(Item::Goods, arg) {
        return Some("mg");
    }
    if scopes == Scopes::MobilizationOption && data.item_exists(Item::MobilizationOption, arg) {
        return Some("mobilization_option");
    }
    if scopes == Scopes::Decree && data.item_exists(Item::Decree, arg) {
        return Some("nf");
    }
    if scopes == Scopes::Province && data.item_exists(Item::Province, arg) {
        return Some("p");
    }
    if scopes == Scopes::PopType && data.item_exists(Item::PopType, arg) {
        return Some("pop_type");
    }
    if scopes == Scopes::Party && data.item_exists(Item::Party, arg) {
        return Some("py");
    }
    if scopes == Scopes::Religion && data.item_exists(Item::Religion, arg) {
        return Some("rel");
    }
    if scopes == Scopes::StateRegion && data.item_exists(Item::StateRegion, arg) {
        return Some("s");
    }
    if scopes == Scopes::StrategicRegion && data.item_exists(Item::StrategicRegion, arg) {
        return Some("sr");
    }
    if scopes == Scopes::CombatUnitType && data.item_exists(Item::CombatUnit, arg) {
        return Some("unit_type");
    }
    None
}

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

/// LAST UPDATED VIC3 VERSION 1.7.6
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Scopes::TradeRoute, "actor_market", Scopes::Market),
    (Scopes::Country, "army_size", Scopes::Value),
    (Scopes::Country, "army_size_including_conscripts", Scopes::Value),
    (Scopes::Battle, "attacker_side", Scopes::BattleSide),
    (Scopes::War, "attacker_warleader", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "average_expected_sol", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "average_sol", Scopes::Value),
    (Scopes::BattleSide, "battle", Scopes::Battle),
    (Scopes::NewCombatUnit, "building", Scopes::Building),
    (Scopes::Country, "building_levels", Scopes::Value),
    (Scopes::Country, "cached_ai_coastal_population", Scopes::Value),
    (Scopes::Country, "cached_ai_incorporated_coastal_population", Scopes::Value),
    (Scopes::Country, "cached_ai_incorporated_population", Scopes::Value),
    (Scopes::Country, "cached_ai_overseas_subject_population", Scopes::Value),
    (Scopes::Country, "cached_ai_subject_population", Scopes::Value),
    (Scopes::Country, "cached_ai_total_population", Scopes::Value),
    (Scopes::Country, "cached_ai_unincorporated_coastal_population", Scopes::Value),
    (Scopes::Country, "cached_ai_unincorporated_population", Scopes::Value),
    (Scopes::Country, "capital", Scopes::State),
    (Scopes::PoliticalMovement, "civil_war", Scopes::CivilWar),
    (Scopes::Country, "civil_war_origin_country", Scopes::Country),
    (Scopes::Country, "colonial_growth_per_colony", Scopes::Value),
    (Scopes::Province, "combat_width", Scopes::Value),
    (Scopes::Character, "command_limit_num_units", Scopes::Value),
    (Scopes::NewCombatUnit, "commander", Scopes::Character),
    (Scopes::Character, "commander_military_formation", Scopes::MilitaryFormation),
    (Scopes::Province.union(Scopes::State), "controller", Scopes::Country),
    (Scopes::MilitaryFormation, "country", Scopes::Country),
    (Scopes::Country, "country_definition", Scopes::CountryDefinition),
    (Scopes::Country, "credit", Scopes::Value),
    (
        Scopes::Character
            .union(Scopes::NewCombatUnit)
            .union(Scopes::PoliticalMovement)
            .union(Scopes::Pop),
        "culture",
        Scopes::Culture,
    ),
    (Scopes::MilitaryFormation, "current_hq", Scopes::Hq),
    (Scopes::Law, "currently_active_law_in_group", Scopes::Law),
    (Scopes::Country, "currently_enacting_law", Scopes::Law),
    (Scopes::Battle, "defender_side", Scopes::BattleSide),
    (Scopes::War, "defender_warleader", Scopes::Country),
    (Scopes::NewCombatUnit, "defense", Scopes::Value),
    (Scopes::NewCombatUnit, "demoralized", Scopes::Value),
    (Scopes::DiplomaticPact, "diplomatic_pact_other_country(", Scopes::Country),
    (Scopes::War, "diplomatic_play", Scopes::DiplomaticPlay),
    (Scopes::Country, "expenses", Scopes::Value),
    (Scopes::TradeRoute, "exporter", Scopes::Market),
    (Scopes::DiplomaticPact, "first_country", Scopes::Country),
    (Scopes::Country, "fixed_expenses", Scopes::Value),
    (
        Scopes::Battle
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::MilitaryFormation),
        "front",
        Scopes::Front,
    ),
    (Scopes::Front, "front_length", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "gdp", Scopes::Value),
    (Scopes::None, "global_gdp", Scopes::Value),
    (Scopes::JournalEntry, "goal_value", Scopes::Value),
    (
        Scopes::MarketGoods.union(Scopes::StateGoods).union(Scopes::TradeRoute),
        "goods",
        Scopes::Goods,
    ),
    (Scopes::Country, "government_size", Scopes::Value),
    (Scopes::Country, "heir", Scopes::Character),
    (Scopes::MilitaryFormation, "highest_ranked_commander", Scopes::Character),
    (Scopes::Character.union(Scopes::Pop), "home_country", Scopes::Country),
    (Scopes::MilitaryFormation, "home_hq", Scopes::Hq),
    (Scopes::PowerBloc, "identity", Scopes::PowerBlocIdentity),
    (Scopes::Character, "ideology", Scopes::Ideology),
    (Scopes::TradeRoute, "importer", Scopes::Market),
    (Scopes::Country, "imposed_law", Scopes::Law),
    (Scopes::Country.union(Scopes::Law), "imposer_of_law", Scopes::Country),
    (Scopes::Country, "income", Scopes::Value),
    (Scopes::Country, "infamy", Scopes::Value),
    (Scopes::DiplomaticPlay, "initiator", Scopes::Country),
    (Scopes::Character, "interest_group", Scopes::InterestGroup),
    (Scopes::Character, "interest_group_type", Scopes::InterestGroupType),
    (Scopes::Institution, "investment", Scopes::Value),
    (Scopes::Institution, "investment_max", Scopes::Value),
    (Scopes::Country, "investment_pool_income", Scopes::Value),
    (Scopes::None, "is_setup", Scopes::Value),
    (Scopes::None, "je_tutorial", Scopes::JournalEntry),
    (Scopes::Province.union(Scopes::State), "land_controller_hq", Scopes::Hq),
    (Scopes::Province.union(Scopes::State), "land_hq", Scopes::Hq),
    (Scopes::InterestGroup, "leader", Scopes::Character),
    (Scopes::Country, "legitimacy", Scopes::Value),
    (Scopes::Building.union(Scopes::TradeRoute), "level", Scopes::Value),
    (Scopes::NewCombatUnit, "manpower", Scopes::Value),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Market)
            .union(Scopes::MarketGoods)
            .union(Scopes::Province)
            .union(Scopes::State)
            .union(Scopes::StateGoods)
            .union(Scopes::StateRegion),
        "market",
        Scopes::Market,
    ),
    (Scopes::Country, "market_capital", Scopes::State),
    (Scopes::State, "mass_migration_culture", Scopes::Culture),
    (Scopes::Country.union(Scopes::State), "migration_pull", Scopes::Value),
    (Scopes::Country, "military_expenses", Scopes::Value),
    (Scopes::Country, "military_expenses_share", Scopes::Value),
    (Scopes::NewCombatUnit, "mobilization", Scopes::Value),
    (Scopes::Party, "momentum", Scopes::Value),
    (Scopes::NewCombatUnit, "morale", Scopes::Value),
    (Scopes::PoliticalMovement, "most_desired_law", Scopes::LawType),
    (Scopes::Province, "naval_controller_hq", Scopes::Hq),
    (Scopes::Province, "naval_hq", Scopes::Hq),
    (Scopes::NavalInvasion, "naval_invasion_attacker", Scopes::Country),
    (Scopes::NavalInvasion, "naval_invasion_defender", Scopes::Country),
    (Scopes::Country, "navy_size", Scopes::Value),
    (Scopes::None, "no", Scopes::Bool),
    (Scopes::None, "NO", Scopes::Bool),
    (Scopes::Country, "num_active_declared_interests", Scopes::Value),
    (Scopes::Country, "num_active_interests", Scopes::Value),
    (Scopes::Country, "num_active_natural_interests", Scopes::Value),
    (Scopes::Country, "num_active_plays", Scopes::Value),
    (Scopes::Country, "num_admirals", Scopes::Value),
    (Scopes::Country, "num_alliances", Scopes::Value),
    (Scopes::Character, "num_battalions", Scopes::Value),
    (Scopes::Country, "num_characters", Scopes::Value),
    (Scopes::Country, "num_colony_projects", Scopes::Value),
    (Scopes::MilitaryFormation, "num_commanderless_units", Scopes::Value),
    (Scopes::Country, "num_commanders", Scopes::Value),
    (Scopes::Country, "num_convoys_available", Scopes::Value),
    (Scopes::Country, "num_convoys_required", Scopes::Value),
    (Scopes::Country, "num_declared_interests", Scopes::Value),
    (Scopes::Country, "num_defensive_pacts", Scopes::Value),
    (Scopes::MarketGoods, "num_export_trade_routes", Scopes::Value),
    (Scopes::Hq, "num_garrison_units", Scopes::Value),
    (Scopes::Country, "num_generals", Scopes::Value),
    (Scopes::MarketGoods, "num_import_trade_routes", Scopes::Value),
    (Scopes::Country, "num_income_transfer_pacts", Scopes::Value),
    (Scopes::Country, "num_incorporated_states", Scopes::Value),
    (Scopes::Country, "num_interests", Scopes::Value),
    (Scopes::PowerBloc, "num_mandates", Scopes::Value),
    (Scopes::Character, "num_mobilized_battalions", Scopes::Value),
    (Scopes::Country, "num_natural_interests", Scopes::Value),
    (Scopes::Country, "num_obligations_earned", Scopes::Value),
    (Scopes::Country, "num_pending_events", Scopes::Value),
    (Scopes::Country, "num_politicians", Scopes::Value),
    (Scopes::Country, "num_positive_relations", Scopes::Value),
    (Scopes::Front.union(Scopes::State).union(Scopes::StateRegion), "num_provinces", Scopes::Value),
    (Scopes::Country, "num_queued_constructions", Scopes::Value),
    (Scopes::Country, "num_queued_government_constructions", Scopes::Value),
    (Scopes::Country, "num_queued_private_constructions", Scopes::Value),
    (Scopes::Country, "num_rivals", Scopes::Value),
    (Scopes::Country, "num_ruling_igs", Scopes::Value),
    (Scopes::Country, "num_states", Scopes::Value),
    (Scopes::Country, "num_trade_routes", Scopes::Value),
    (Scopes::Country, "num_treaty_ports", Scopes::Value),
    (Scopes::Country, "num_unincorporated_states", Scopes::Value),
    (Scopes::Character.union(Scopes::MilitaryFormation), "num_units", Scopes::Value),
    (Scopes::Character, "num_units_share", Scopes::Value),
    (Scopes::NewCombatUnit, "offense", Scopes::Value),
    (Scopes::Character, "opposing_commander", Scopes::Character),
    (Scopes::BattleSide, "opposite_battle_side", Scopes::BattleSide),
    (Scopes::Country, "overlord", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::NewCombatUnit)
            .union(Scopes::Decree)
            .union(Scopes::Institution)
            .union(Scopes::InterestMarker)
            .union(Scopes::InterestGroup)
            .union(Scopes::JournalEntry)
            .union(Scopes::Law)
            .union(Scopes::Market)
            .union(Scopes::MarketGoods)
            .union(Scopes::Pop)
            .union(Scopes::Province)
            .union(Scopes::State)
            .union(Scopes::TradeRoute)
            .union(Scopes::MilitaryFormation)
            .union(Scopes::PoliticalMovement),
        "owner",
        Scopes::Country,
    ),
    (Scopes::Market, "participants", Scopes::Value),
    (Scopes::InterestGroup, "party", Scopes::Party),
    (Scopes::None, "player", Scopes::Country), // TODO "do not use this outside tutorial"
    (Scopes::DiplomaticRelations, "player_owed_obligation_days_left", Scopes::Value),
    (Scopes::Character.union(Scopes::CivilWar), "political_movement", Scopes::PoliticalMovement),
    (Scopes::Pop, "pop_weight_modifier_scale", Scopes::Value),
    (Scopes::Character, "popularity", Scopes::Value),
    (Scopes::State, "population_below_expected_sol", Scopes::Value),
    (Scopes::Country, "power_bloc", Scopes::PowerBloc),
    (Scopes::PowerBloc, "power_bloc_leader", Scopes::Country),
    (Scopes::PowerBloc, "power_struggle_contender", Scopes::Country),
    (Scopes::Company, "prosperity", Scopes::Value),
    (Scopes::BattleSide, "province", Scopes::Province),
    (
        Scopes::Building
            .union(Scopes::DiplomaticPlay)
            .union(Scopes::Hq)
            .union(Scopes::InterestMarker)
            .union(Scopes::Province)
            .union(Scopes::State)
            .union(Scopes::StateRegion),
        "region",
        Scopes::StrategicRegion,
    ),
    (
        Scopes::Country
            .union(Scopes::Character)
            .union(Scopes::CountryDefinition)
            .union(Scopes::PoliticalMovement)
            .union(Scopes::Pop),
        "religion",
        Scopes::Religion,
    ),
    (Scopes::Country, "ruler", Scopes::Character),
    (Scopes::DiplomaticRelations, "scope_relations", Scopes::Value),
    (Scopes::DiplomaticRelations, "scope_tension", Scopes::Value),
    (Scopes::DiplomaticPact, "second_country", Scopes::Country),
    (Scopes::BuildingType, "slaves_role", Scopes::PopType),
    (
        Scopes::Building.union(Scopes::Market).union(Scopes::Pop).union(Scopes::Province),
        "state",
        Scopes::State,
    ),
    (Scopes::State, "state_region", Scopes::StateRegion),
    (
        Scopes::DiplomaticPlay
            .union(Scopes::DiplomaticCatalyst)
            .union(Scopes::PoliticalLobby)
            .union(Scopes::JournalEntry),
        "target",
        Scopes::all(),
    ), // TODO: scope type?
    (Scopes::TradeRoute, "target_market", Scopes::Market),
    (Scopes::Country, "technology_being_researched", Scopes::Technology),
    (Scopes::Country, "techs_researched", Scopes::Value),
    (Scopes::BattleSide.union(Scopes::State), "theater", Scopes::Theater),
    (Scopes::Country, "top_overlord", Scopes::Country),
    (Scopes::Market, "trade_center", Scopes::State),
    (Scopes::Building, "training_rate", Scopes::Value),
    // The input and output scopes for this are special cased
    (
        Scopes::Building
            .union(Scopes::Company)
            .union(Scopes::DiplomaticPlay)
            .union(Scopes::DiplomaticCatalyst)
            .union(Scopes::PoliticalLobby)
            .union(Scopes::Institution)
            .union(Scopes::InterestGroup)
            .union(Scopes::Law)
            .union(Scopes::PoliticalMovement)
            .union(Scopes::HarvestCondition),
        "type",
        Scopes::BuildingType
            .union(Scopes::CompanyType)
            .union(Scopes::DiplomaticPlayType)
            .union(Scopes::DiplomaticCatalystType)
            .union(Scopes::PoliticalLobbyType)
            .union(Scopes::InstitutionType)
            .union(Scopes::InterestGroupType)
            .union(Scopes::LawType)
            .union(Scopes::PoliticalMovementType)
            .union(Scopes::HarvestConditionType),
    ),
    (Scopes::DiplomaticPlay, "war", Scopes::War),
    (Scopes::Company, "weekly_prosperity_change", Scopes::Value),
    (Scopes::Pop, "workplace", Scopes::Building),
    (Scopes::None, "yes", Scopes::Bool),
    (Scopes::None, "YES", Scopes::Bool),
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

/// LAST UPDATED VIC3 VERSION 1.7.6
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
const SCOPE_PREFIX: &[(Scopes, &str, Scopes, ArgumentValue)] = {
    use crate::item::Item;
    use ArgumentValue::*;
    &[
        (Scopes::Country, "active_law", Scopes::Law, Item(Item::LawGroup)),
        (Scopes::Country, "ai_army_comparison", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_gdp_comparison", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_ideological_opinion", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_navy_comparison", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::State, "ai_state_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_subject_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::State, "ai_treaty_port_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "array_define", Scopes::Value, UncheckedValue),
        (Scopes::Front, "average_defense", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Front, "average_offense", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::State, "b", Scopes::Building, Item(Item::BuildingType)),
        (Scopes::None, "bt", Scopes::BuildingType, Item(Item::BuildingType)),
        (Scopes::None, "c", Scopes::Country, Item(Item::Country)),
        (
            Scopes::None,
            "catalyst_type",
            Scopes::DiplomaticCatalystType,
            Item(Item::DiplomaticCatalyst),
        ),
        (Scopes::None, "cd", Scopes::CountryDefinition, Item(Item::Country)),
        (Scopes::Country, "company", Scopes::Company, Item(Item::CompanyType)),
        (Scopes::None, "company_type", Scopes::CompanyType, Item(Item::CompanyType)),
        (Scopes::None, "cu", Scopes::Culture, Item(Item::Culture)),
        (Scopes::State, "decree_cost", Scopes::Value, Item(Item::Decree)),
        (Scopes::None, "define", Scopes::Value, UncheckedValue),
        (Scopes::None, "flag", Scopes::Flag, UncheckedValue),
        (Scopes::None, "g", Scopes::Goods, Item(Item::Goods)),
        (Scopes::Country, "get_ruler_for", Scopes::Character, Item(Item::TransferOfPower)),
        (Scopes::None, "global_var", Scopes::all(), UncheckedValue),
        (
            Scopes::None,
            "harvest_condition_type",
            Scopes::HarvestConditionType,
            Item(Item::HarvestConditionType),
        ),
        (Scopes::None, "i", Scopes::Ideology, Item(Item::Ideology)),
        (Scopes::None, "identity", Scopes::PowerBlocIdentity, Item(Item::PowerBlocIdentity)),
        (Scopes::None, "ideology", Scopes::Ideology, Item(Item::Ideology)), // TODO difference with i:
        (Scopes::Country, "ig", Scopes::InterestGroup, Item(Item::InterestGroup)),
        (Scopes::None, "ig_trait", Scopes::InterestGroupTrait, Item(Item::InterestGroupTrait)),
        (Scopes::None, "ig_type", Scopes::InterestGroupType, Item(Item::InterestGroup)),
        (Scopes::None, "infamy_threshold", Scopes::Value, Item(Item::InfamyThreshold)),
        (Scopes::Country, "institution", Scopes::Institution, Item(Item::Institution)),
        (Scopes::Country, "je", Scopes::JournalEntry, Item(Item::JournalEntry)),
        (Scopes::None, "law_type", Scopes::LawType, Item(Item::LawType)),
        (Scopes::None, "list_size", Scopes::Value, UncheckedValue),
        (Scopes::Country, "lobby_foreign_anti_clout", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "lobby_foreign_pro_clout", Scopes::Value, Scope(Scopes::Country)),
        (
            Scopes::Country,
            "lobby_in_government_foreign_anti_clout",
            Scopes::Value,
            Scope(Scopes::Country),
        ),
        (
            Scopes::Country,
            "lobby_in_government_foreign_pro_clout",
            Scopes::Value,
            Scope(Scopes::Country),
        ),
        (Scopes::InterestGroup, "lobby_join_weight", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "lobby_type", Scopes::PoliticalLobbyType, Item(Item::PoliticalLobby)),
        (Scopes::Country, "lobby_war_opposition", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "lobby_war_support", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "local_var", Scopes::all(), UncheckedValue),
        (Scopes::Market, "mg", Scopes::MarketGoods, Item(Item::Goods)),
        (
            Scopes::None,
            "mobilization_option",
            Scopes::MobilizationOption,
            Item(Item::MobilizationOption),
        ),
        (
            Scopes::Country
                .union(Scopes::BattleSide)
                .union(Scopes::Building)
                .union(Scopes::Character)
                .union(Scopes::InterestGroup)
                .union(Scopes::Market)
                .union(Scopes::PowerBloc)
                .union(Scopes::State),
            "modifier",
            Scopes::Value.union(Scopes::Bool),
            Modif,
        ),
        (
            Scopes::None,
            "movement_type",
            Scopes::PoliticalMovementType,
            Item(Item::PoliticalMovement),
        ),
        (Scopes::State, "nf", Scopes::Decree, Item(Item::Decree)),
        (
            Scopes::Country,
            "num_alliances_and_defensive_pacts_with_allies",
            Scopes::Value,
            Scope(Scopes::Country),
        ),
        (
            Scopes::Country,
            "num_alliances_and_defensive_pacts_with_rivals",
            Scopes::Value,
            Scope(Scopes::Country),
        ),
        (Scopes::Front, "num_defending_battalions", Scopes::Value, Scope(Scopes::Country)),
        (
            Scopes::Country,
            "num_mutual_trade_route_levels_with_country",
            Scopes::Value,
            Scope(Scopes::Country),
        ),
        (Scopes::Country, "num_pending_events", Scopes::Value, Item(Item::EventCategory)),
        (Scopes::Country, "num_shared_rivals", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Front, "num_total_battalions", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "p", Scopes::Province, Item(Item::Province)),
        (Scopes::None, "play_type", Scopes::DiplomaticPlayType, Item(Item::DiplomaticPlay)),
        (Scopes::None, "pop_type", Scopes::PopType, Item(Item::PopType)),
        (Scopes::None, "principle", Scopes::PowerBlocPrinciple, Item(Item::Principle)),
        (
            Scopes::None,
            "principle_group",
            Scopes::PowerBlocPrincipleGroup,
            Item(Item::PrincipleGroup),
        ),
        (Scopes::Country, "py", Scopes::Party, Item(Item::Party)),
        (Scopes::None, "rank_value", Scopes::Value, Item(Item::CountryRank)),
        (Scopes::StateRegion, "region_state", Scopes::State, Item(Item::Country)), // undocumented
        (Scopes::None, "rel", Scopes::Religion, Item(Item::Religion)),
        (Scopes::Country, "relations", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "relations_change_rate", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "relations_threshold", Scopes::Value, Item(Item::RelationsThreshold)),
        (Scopes::None, "s", Scopes::StateRegion, Item(Item::StateRegion)),
        (Scopes::None, "scope", Scopes::all(), UncheckedValue),
        (Scopes::State, "sg", Scopes::StateGoods, Item(Item::Goods)),
        (Scopes::None, "sr", Scopes::StrategicRegion, Item(Item::StrategicRegion)),
        (Scopes::Country, "tension", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::None, "tension_threshold", Scopes::Value, UncheckedValue),
        (Scopes::None, "unit_type", Scopes::CombatUnitType, Item(Item::CombatUnit)),
        (Scopes::all(), "var", Scopes::all(), UncheckedValue),
    ]
};

#[inline]
pub fn scope_iterator(name: &str) -> Option<(Scopes, Scopes)> {
    SCOPE_ITERATOR_MAP.get(name).copied()
}

static SCOPE_ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, to) in SCOPE_ITERATOR.iter().copied() {
            hash.insert(s, (from, to));
        }
        hash
    });

/// LAST UPDATED VIC3 VERSION 1.8.1
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Country, "active_law", Scopes::Law),
    (Scopes::Country, "active_party", Scopes::Party),
    (Scopes::None, "character", Scopes::Character),
    (Scopes::None, "character_in_exile_pool", Scopes::Character),
    (Scopes::None, "character_in_void", Scopes::Character),
    (Scopes::Country, "civil_war", Scopes::CivilWar),
    (Scopes::Country, "cobelligerent_in_diplo_play", Scopes::Country),
    (Scopes::Country, "cobelligerent_in_war", Scopes::Country),
    (
        Scopes::Battle
            .union(Scopes::Building)
            .union(Scopes::Front)
            .union(Scopes::Hq)
            .union(Scopes::MilitaryFormation),
        "combat_unit",
        Scopes::NewCombatUnit,
    ),
    (Scopes::Country, "company", Scopes::Company),
    (Scopes::None, "country", Scopes::Country),
    (Scopes::Country, "diplomatic_catalyst", Scopes::DiplomaticCatalyst),
    (Scopes::None, "diplomatic_play", Scopes::DiplomaticPlay),
    (Scopes::Country, "diplomatically_relevant_country", Scopes::Country),
    (Scopes::Country, "direct_subject", Scopes::Country),
    (Scopes::Country, "enemy_in_diplo_play", Scopes::Country),
    (Scopes::Country, "enemy_in_war", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::State)
            .union(Scopes::StateRegion)
            .union(Scopes::StrategicRegion),
        "harvest_condition",
        Scopes::HarvestConditionType,
    ),
    (Scopes::None, "in_global_list", Scopes::all_but_none()),
    (Scopes::Country, "in_hierarchy", Scopes::Country),
    (Scopes::None, "in_list", Scopes::all_but_none()),
    (Scopes::None, "in_local_list", Scopes::all_but_none()),
    (Scopes::PoliticalMovement, "influenced_interest_group", Scopes::InterestGroup),
    (Scopes::Country, "interest_group", Scopes::InterestGroup),
    (Scopes::Country, "law", Scopes::Law),
    (Scopes::PoliticalLobby, "lobby_member", Scopes::InterestGroup),
    (Scopes::None, "market", Scopes::Market),
    (Scopes::Market, "market_goods", Scopes::MarketGoods),
    (Scopes::Party, "member", Scopes::InterestGroup),
    (
        Scopes::Country.union(Scopes::Front).union(Scopes::Hq),
        "military_formation",
        Scopes::MilitaryFormation,
    ),
    (
        Scopes::Country
            .union(Scopes::State)
            .union(Scopes::StateRegion)
            .union(Scopes::StrategicRegion),
        "neighbouring_state",
        Scopes::State,
    ),
    (Scopes::Country, "overlord_or_above", Scopes::Country),
    (Scopes::DiplomaticPact, "participant", Scopes::Country),
    (Scopes::Country.union(Scopes::InterestGroup), "political_lobby", Scopes::PoliticalLobby),
    (Scopes::Country, "political_movement", Scopes::PoliticalMovement),
    (Scopes::Country, "potential_party", Scopes::Party),
    (Scopes::None, "power_bloc", Scopes::PowerBloc),
    (Scopes::PowerBloc, "power_bloc_member", Scopes::Country),
    (Scopes::InterestGroup, "preferred_law", Scopes::Law),
    (Scopes::Country.union(Scopes::CountryDefinition), "primary_culture", Scopes::Culture),
    // TODO: verify. The docs have State and Province reversed.
    (Scopes::State, "province", Scopes::Province),
    (Scopes::Country, "rival_country", Scopes::Country),
    (Scopes::Country, "rivaling_country", Scopes::Country),
    // TODO: Scopes::Front is in the docs but doesn't make sense for admirals
    (
        Scopes::Country
            .union(Scopes::Front)
            .union(Scopes::InterestGroup)
            .union(Scopes::MilitaryFormation),
        "scope_admiral",
        Scopes::Character,
    ),
    (Scopes::Country, "scope_ally", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "scope_building", Scopes::Building),
    (
        Scopes::Country
            .union(Scopes::Front)
            .union(Scopes::InterestGroup)
            .union(Scopes::MilitaryFormation),
        "scope_character",
        Scopes::Character,
    ),
    (Scopes::Market.union(Scopes::StrategicRegion), "scope_country", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "scope_culture", Scopes::Culture),
    (Scopes::Country, "scope_diplomatic_pact", Scopes::DiplomaticPact),
    (Scopes::War, "scope_front", Scopes::Front),
    (
        Scopes::Country
            .union(Scopes::Front)
            .union(Scopes::InterestGroup)
            .union(Scopes::MilitaryFormation),
        "scope_general",
        Scopes::Character,
    ),
    (Scopes::Country, "scope_held_interest_marker", Scopes::InterestMarker),
    (Scopes::DiplomaticPlay, "scope_initiator_ally", Scopes::Country),
    (
        Scopes::Country.union(Scopes::StrategicRegion),
        "scope_interest_marker",
        Scopes::InterestMarker,
    ),
    (Scopes::DiplomaticPlay, "scope_play_involved", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Front)
            .union(Scopes::InterestGroup)
            .union(Scopes::MilitaryFormation),
        "scope_politician",
        Scopes::Character,
    ),
    (
        Scopes::Country.union(Scopes::Culture).union(Scopes::InterestGroup).union(Scopes::State),
        "scope_pop",
        Scopes::Pop,
    ),
    (
        Scopes::Country
            .union(Scopes::Front)
            .union(Scopes::StateRegion)
            .union(Scopes::StrategicRegion)
            .union(Scopes::Theater),
        "scope_state",
        Scopes::State,
    ),
    (Scopes::DiplomaticPlay, "scope_target_ally", Scopes::Country),
    (Scopes::Country, "scope_theater", Scopes::Theater),
    (Scopes::Country, "scope_violate_sovereignty_interested_parties", Scopes::Country),
    (Scopes::Country, "scope_violate_sovereignty_wars", Scopes::War),
    (Scopes::Country, "scope_war", Scopes::War),
    // TODO: check if the scoped state is a sea node
    (Scopes::State, "sea_node_adjacent_state", Scopes::State),
    (Scopes::None, "state", Scopes::State),
    (Scopes::None, "state_region", Scopes::StateRegion),
    (Scopes::Country, "strategic_objective", Scopes::State),
    (Scopes::Country, "subject_of_subject", Scopes::Country),
    (Scopes::Country, "subject_or_below", Scopes::Country),
    (Scopes::PoliticalMovement, "supporting_character", Scopes::Character),
    (
        Scopes::Country.union(Scopes::Market).union(Scopes::MarketGoods),
        "trade_route",
        Scopes::TradeRoute,
    ),
    (Scopes::Country, "valid_mass_migration_culture", Scopes::Culture),
    (Scopes::War, "war_participant", Scopes::Country),
];

pub fn scope_iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in SCOPE_ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const SCOPE_ITERATOR_REMOVED: &[(&str, &str, &str)] = &[
    (
        "scope_cobelligerent",
        "1.4.0",
        "replaced with _cobelligerent_in_diplo_play, _cobelligerent_in_war",
    ),
    ("supporting_interest_group", "1.8", "replaced with `_influenced_interest_group`"),
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
    ("num_commanded_units", "1.6", ""),
    ("num_enemy_units", "1.6", ""),
    ("num_units_not_in_battle", "1.6", ""),
    ("supply", "1.6", ""),
    ("active_diplomatic_play", "1.7", ""),
];
