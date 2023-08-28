#![allow(non_upper_case_globals)]

use std::fmt::Formatter;

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::Severity;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::trigger::validate_target;

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
        "combat_unit" => Scopes::CombatUnit,
        "commander_order" => Scopes::CommanderOrder,
        "commander_order_type" => Scopes::CommanderOrderType,
        "country_creation" => Scopes::CountryCreation,
        "country_definition" => Scopes::CountryDefinition,
        "country_formation" => Scopes::CountryFormation,
        "culture" => Scopes::Culture,
        "decree" => Scopes::Decree,
        "diplomatic_action" => Scopes::DiplomaticAction,
        "diplomatic_pact" => Scopes::DiplomaticPact,
        "diplomatic_play" => Scopes::DiplomaticPlay,
        "diplomatic_relations" => Scopes::DiplomaticRelations,
        "front" => Scopes::Front,
        "goods" => Scopes::Goods,
        "hq" => Scopes::Hq,
        "ideology" => Scopes::Ideology,
        "institution" => Scopes::Institution,
        "institution_type" => Scopes::InstitutionType,
        "interest_marker" => Scopes::InterestMarker,
        "interest_group" => Scopes::InterestGroup,
        "interest_group_trait" => Scopes::InterestGroupTrait,
        "interest_group_type" => Scopes::InterestGroupType,
        "journalentry" => Scopes::Journalentry,
        "law" => Scopes::Law,
        "law_type" => Scopes::LawType,
        "market" => Scopes::Market,
        "market_goods" => Scopes::MarketGoods,
        "objective" => Scopes::Objective,
        "party" => Scopes::Party,
        "political_movement" => Scopes::PoliticalMovement,
        "pop" => Scopes::Pop,
        "pop_type" => Scopes::PopType,
        "province" => Scopes::Province,
        "religion" => Scopes::Religion,
        "shipping_lane" => Scopes::ShippingLane,
        "state" => Scopes::State,
        "state_region" => Scopes::StateRegion,
        "state_trait" => Scopes::StateTrait,
        "strategic_region" => Scopes::StrategicRegion,
        "technology" => Scopes::Technology,
        "technology_status" => Scopes::TechnologyStatus,
        "theater" => Scopes::Theater,
        "trade_route" => Scopes::TradeRoute,
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
    if s.contains(Scopes::CombatUnit) {
        vec.push("combat unit");
    }
    if s.contains(Scopes::CommanderOrder) {
        vec.push("commander order");
    }
    if s.contains(Scopes::CommanderOrderType) {
        vec.push("commander order_type");
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
    if s.contains(Scopes::Culture) {
        vec.push("culture");
    }
    if s.contains(Scopes::Decree) {
        vec.push("decree");
    }
    if s.contains(Scopes::DiplomaticAction) {
        vec.push("diplomatic action");
    }
    if s.contains(Scopes::DiplomaticPact) {
        vec.push("diplomatic pact");
    }
    if s.contains(Scopes::DiplomaticPlay) {
        vec.push("diplomatic play");
    }
    if s.contains(Scopes::DiplomaticRelations) {
        vec.push("diplomatic relations");
    }
    if s.contains(Scopes::Front) {
        vec.push("front");
    }
    if s.contains(Scopes::Goods) {
        vec.push("goods");
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
    if s.contains(Scopes::Journalentry) {
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
    if s.contains(Scopes::Objective) {
        vec.push("objective");
    }
    if s.contains(Scopes::Party) {
        vec.push("party");
    }
    if s.contains(Scopes::PoliticalMovement) {
        vec.push("political movement");
    }
    if s.contains(Scopes::Pop) {
        vec.push("pop");
    }
    if s.contains(Scopes::PopType) {
        vec.push("pop type");
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
    if s.contains(Scopes::StateRegion) {
        vec.push("state region");
    }
    if s.contains(Scopes::StateTrait) {
        vec.push("state trait");
    }
    if s.contains(Scopes::StrategicRegion) {
        vec.push("strategic region");
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
    if s.contains(Scopes::War) {
        vec.push("war");
    }
    display_choices(f, &vec, "or")
}

pub fn validate_prefix_reference(
    prefix: &Token,
    arg: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
) {
    // TODO there are more to match
    // TODO integrate this to the SCOPE_FROM_PREFIX table
    match prefix.as_str() {
        "active_law" => data.verify_exists(Item::LawGroup, arg),
        "ai_army_comparison"
        | "ai_gdp_comparison"
        | "ai_ideological_opinion"
        | "ai_navy_comparison"
        | "average_defense"
        | "average_offense"
        | "num_defending_battalions"
        | "num_total_battalions"
        | "relations"
        | "tension"
        | "num_alliances_and_defensive_pacts_with_allies"
        | "num_alliances_and_defensive_pacts_with_rivals"
        | "num_mutual_trade_route_levels_with_country" => {
            validate_target(arg, data, sc, Scopes::Country);
        }
        // "array_define"
        "b" => data.verify_exists(Item::BuildingType, arg),
        "c" | "cd" | "region_state" => data.verify_exists(Item::Country, arg),
        "cu" => data.verify_exists(Item::Culture, arg),
        "decree_cost" | "nf" => data.verify_exists(Item::Decree, arg),
        // "define"
        // "diplomatic_pact_other_country"
        // "flag"
        "g" | "mg" => data.verify_exists(Item::Goods, arg),
        // "get_ruler_for"
        // "global_var"
        "i" | "ideology" => data.verify_exists(Item::Ideology, arg),
        "ig" => data.verify_exists(Item::InterestGroup, arg),
        "ig_trait" => data.verify_exists(Item::InterestGroupTrait, arg),
        // "ig_type" => data.verify_exists(Item::InterestGroupType, arg),
        "infamy_threshold" => data.verify_exists(Item::InfamyThreshold, arg),
        "institution" => data.verify_exists(Item::Institution, arg),
        "je" => data.verify_exists(Item::Journalentry, arg),
        "law_type" => data.verify_exists(Item::LawType, arg),
        // "local_var"
        "modifier" => verify_modif_exists(arg, data, ModifKinds::all(), Severity::Error),
        "num_enemy_units" => validate_target(arg, data, sc, Scopes::Character), // TODO verify type
        // "num_pending_events" =>
        "p" => data.verify_exists(Item::Province, arg),
        "pop_type" => data.verify_exists(Item::PopType, arg),
        "py" => data.verify_exists(Item::Party, arg),
        "rank_value" => data.verify_exists(Item::CountryRank, arg),
        "rel" => data.verify_exists(Item::Religion, arg),
        "relations_threshold" => data.verify_exists(Item::RelationsThreshold, arg),
        "s" => data.verify_exists(Item::StateRegion, arg),
        // "scope"
        "sr" => data.verify_exists(Item::StrategicRegion, arg),
        // "tension_threshold" =>
        // "var"
        &_ => (),
    }
}

pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    if scopes == Scopes::Building && data.item_exists(Item::BuildingType, arg) {
        return Some("b");
    }
    if scopes == Scopes::Country && data.item_exists(Item::Country, arg) {
        return Some("c");
    }
    if scopes == Scopes::CountryDefinition && data.item_exists(Item::Country, arg) {
        return Some("cd");
    }
    if scopes == Scopes::Culture && data.item_exists(Item::Culture, arg) {
        return Some("cu");
    }
    if scopes == Scopes::Flag {
        return Some("flag");
    }
    if scopes == Scopes::Goods && data.item_exists(Item::Goods, arg) {
        return Some("g");
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
    if scopes == Scopes::Journalentry && data.item_exists(Item::Journalentry, arg) {
        return Some("je");
    }
    if scopes == Scopes::LawType && data.item_exists(Item::LawType, arg) {
        return Some("law_type");
    }
    if scopes == Scopes::MarketGoods && data.item_exists(Item::Goods, arg) {
        return Some("mg");
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
    None
}

/// LAST UPDATED VIC3 VERSION 1.4.0
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
pub const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (
        Scopes::Country.union(Scopes::StrategicRegion),
        "active_diplomatic_play",
        Scopes::DiplomaticPlay,
    ),
    (Scopes::TradeRoute, "actor_market", Scopes::Market),
    (Scopes::Country, "army_size", Scopes::Value),
    (Scopes::Country, "army_size_including_conscripts", Scopes::Value),
    (Scopes::Battle, "attacker_side", Scopes::BattleSide),
    (Scopes::War, "attacker_warleader", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "average_expected_sol", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "average_sol", Scopes::Value),
    (Scopes::BattleSide, "battle", Scopes::Battle),
    (Scopes::CombatUnit, "building", Scopes::Building),
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
    (Scopes::CombatUnit, "commander", Scopes::Character),
    (Scopes::Province.union(Scopes::State), "controller", Scopes::Country),
    (Scopes::Country, "country_definition", Scopes::CountryDefinition),
    (Scopes::Country, "credit", Scopes::Value),
    (Scopes::Character.union(Scopes::CombatUnit).union(Scopes::Pop), "culture", Scopes::Culture),
    (Scopes::Law, "currently_active_law_in_group", Scopes::Law),
    (Scopes::Country, "currently_enacting_law", Scopes::Law),
    (Scopes::Battle, "defender_side", Scopes::BattleSide),
    (Scopes::War, "defender_warleader", Scopes::Country),
    (Scopes::CombatUnit, "defense", Scopes::Value),
    (Scopes::CombatUnit, "demoralized", Scopes::Value),
    (Scopes::PoliticalMovement, "desired_law", Scopes::LawType),
    (Scopes::DiplomaticPact, "diplomatic_pact_other_country(", Scopes::Country),
    (Scopes::TradeRoute, "exporter", Scopes::Market),
    (Scopes::DiplomaticPact, "first_country", Scopes::Country),
    (Scopes::Battle.union(Scopes::Character), "front", Scopes::Front),
    (Scopes::Front, "front_length", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "gdp", Scopes::Value),
    (Scopes::None, "global_gdp", Scopes::Value),
    (Scopes::Journalentry, "goal_value", Scopes::Value),
    (Scopes::MarketGoods.union(Scopes::TradeRoute), "goods", Scopes::Goods),
    (Scopes::Country, "government_size", Scopes::Value),
    (Scopes::Country, "heir", Scopes::Character),
    (Scopes::Character.union(Scopes::Pop), "home_country", Scopes::Country),
    (Scopes::Character, "ideology", Scopes::Ideology),
    (Scopes::TradeRoute, "importer", Scopes::Market),
    (Scopes::Country, "income", Scopes::Value),
    (Scopes::Country, "infamy", Scopes::Value),
    (Scopes::DiplomaticPlay, "initiator", Scopes::Country),
    (Scopes::Character, "interest_group", Scopes::InterestGroup),
    (Scopes::Character, "interest_group_type", Scopes::InterestGroupType),
    (Scopes::Institution, "investment", Scopes::Value),
    (Scopes::Institution, "investment_max", Scopes::Value),
    (Scopes::None, "is_setup", Scopes::Value),
    (Scopes::Country, "je_tutorial", Scopes::Journalentry),
    (Scopes::Province.union(Scopes::State), "land_hq", Scopes::Hq),
    (Scopes::InterestGroup, "leader", Scopes::Character),
    (Scopes::Country, "legitimacy", Scopes::Value),
    (Scopes::Building.union(Scopes::TradeRoute), "level", Scopes::Value),
    (Scopes::CombatUnit, "manpower", Scopes::Value),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Market)
            .union(Scopes::MarketGoods)
            .union(Scopes::Province)
            .union(Scopes::State)
            .union(Scopes::StateRegion),
        "market",
        Scopes::Market,
    ),
    (Scopes::Country, "market_capital", Scopes::State),
    (Scopes::CombatUnit, "mobilization", Scopes::Value),
    (Scopes::Party, "momentum", Scopes::Value),
    (Scopes::CombatUnit, "morale", Scopes::Value),
    (Scopes::Province, "naval_hq", Scopes::Hq),
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
    (Scopes::Character, "num_commanded_units", Scopes::Value),
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
    (Scopes::Character, "num_units", Scopes::Value),
    (Scopes::Character, "num_units_not_in_battle", Scopes::Value),
    (Scopes::CombatUnit, "offense", Scopes::Value),
    (Scopes::Character, "opposing_commander", Scopes::Character),
    (Scopes::Country, "overlord", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::CombatUnit)
            .union(Scopes::Decree)
            .union(Scopes::Institution)
            .union(Scopes::InterestMarker)
            .union(Scopes::InterestGroup)
            .union(Scopes::Journalentry)
            .union(Scopes::Law)
            .union(Scopes::Market)
            .union(Scopes::MarketGoods)
            .union(Scopes::Pop)
            .union(Scopes::Province)
            .union(Scopes::State)
            .union(Scopes::TradeRoute),
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
    (Scopes::BattleSide, "province", Scopes::Province),
    (
        Scopes::Building
            .union(Scopes::DiplomaticPlay)
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
    (Scopes::Character, "supply", Scopes::Value),
    (Scopes::DiplomaticPlay.union(Scopes::Journalentry), "target", Scopes::all()), // TODO: scope type?
    (Scopes::TradeRoute, "target_market", Scopes::Market),
    (Scopes::Country, "technology_being_researched", Scopes::Technology),
    (Scopes::Country, "techs_researched", Scopes::Value),
    (Scopes::Country, "top_overlord", Scopes::Country),
    (Scopes::Market, "trade_center", Scopes::State),
    (Scopes::Building, "training_rate", Scopes::Value),
    // The input and output scopes for this are special cased
    (
        Scopes::Building
            .union(Scopes::CommanderOrder)
            .union(Scopes::Institution)
            .union(Scopes::InterestGroup)
            .union(Scopes::Law),
        "type",
        Scopes::BuildingType
            .union(Scopes::CommanderOrderType)
            .union(Scopes::InstitutionType)
            .union(Scopes::InterestGroupType)
            .union(Scopes::LawType),
    ),
    (Scopes::DiplomaticPlay, "war", Scopes::War),
    (Scopes::Pop, "workplace", Scopes::Building),
    (Scopes::None, "yes", Scopes::Bool),
    (Scopes::None, "YES", Scopes::Bool),
];

/// LAST UPDATED VIC3 VERSION 1.4.0
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
/// TODO: add the Item type here, so that it can be checked for existence.
pub const SCOPE_FROM_PREFIX: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Country, "active_law", Scopes::Law),
    (Scopes::Country, "ai_army_comparison", Scopes::Value),
    (Scopes::Country, "ai_gdp_comparison", Scopes::Value),
    (Scopes::Country, "ai_ideological_opinion", Scopes::Value),
    (Scopes::Country, "ai_navy_comparison", Scopes::Value),
    (Scopes::None, "array_define", Scopes::Value),
    (Scopes::Front, "average_defense", Scopes::Value),
    (Scopes::Front, "average_offense", Scopes::Value),
    (Scopes::State, "b", Scopes::Building),
    (Scopes::None, "c", Scopes::Country),
    (Scopes::None, "cd", Scopes::CountryDefinition),
    (Scopes::None, "cu", Scopes::Culture),
    (Scopes::Country, "decree_cost", Scopes::Value),
    (Scopes::None, "define", Scopes::Value),
    (Scopes::None, "flag", Scopes::Flag),
    (Scopes::None, "g", Scopes::Goods),
    (Scopes::Country, "get_ruler_for", Scopes::Character),
    (Scopes::None, "global_var", Scopes::all()),
    (Scopes::None, "i", Scopes::Ideology),
    (Scopes::None, "ideology", Scopes::Ideology), // TODO difference with i:
    (Scopes::Country, "ig", Scopes::InterestGroup),
    (Scopes::None, "ig_trait", Scopes::InterestGroupTrait),
    (Scopes::None, "ig_type", Scopes::InterestGroupType),
    (Scopes::None, "infamy_threshold", Scopes::Value),
    (Scopes::Country, "institution", Scopes::Institution),
    (Scopes::Country, "je", Scopes::Journalentry),
    (Scopes::None, "law_type", Scopes::LawType),
    (Scopes::None, "local_var", Scopes::all()),
    (Scopes::Market, "mg", Scopes::MarketGoods),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::InterestGroup)
            .union(Scopes::Market)
            .union(Scopes::State),
        "modifier",
        Scopes::Value.union(Scopes::Bool),
    ),
    (Scopes::State, "nf", Scopes::Decree),
    (Scopes::Country, "num_alliances_and_defensive_pacts_with_allies", Scopes::Value),
    (Scopes::Country, "num_alliances_and_defensive_pacts_with_rivals", Scopes::Value),
    (Scopes::Front, "num_defending_battalions", Scopes::Value),
    (Scopes::Front, "num_enemy_units", Scopes::Value),
    (Scopes::Country, "num_mutual_trade_route_levels_with_country", Scopes::Value),
    (Scopes::Country, "num_pending_events", Scopes::Value),
    (Scopes::Front, "num_total_battalions", Scopes::Value),
    (Scopes::None, "p", Scopes::Province),
    (Scopes::None, "pop_type", Scopes::PopType),
    (Scopes::Country, "py", Scopes::Party),
    (Scopes::None, "rank_value", Scopes::Value),
    (Scopes::StateRegion, "region_state", Scopes::State), // undocumented
    (Scopes::None, "rel", Scopes::Religion),
    (Scopes::Country, "relations", Scopes::Value),
    (Scopes::None, "relations_threshold", Scopes::Value),
    (Scopes::None, "s", Scopes::StateRegion),
    (Scopes::None, "scope", Scopes::all()),
    (Scopes::None, "sr", Scopes::StrategicRegion),
    (Scopes::Country, "tension", Scopes::Value),
    (Scopes::None, "tension_threshold", Scopes::Value),
    (Scopes::all(), "var", Scopes::all()),
];

/// LAST UPDATED VIC3 VERSION 1.4.0
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
pub const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Country, "active_party", Scopes::Party),
    (Scopes::None, "character", Scopes::Character),
    (Scopes::None, "character_in_exile_pool", Scopes::Character),
    (Scopes::None, "character_in_void", Scopes::Character),
    (Scopes::Country, "civil_war", Scopes::CivilWar),
    (Scopes::Country, "cobelligerent_in_diplo_play", Scopes::Country),
    (Scopes::Country, "cobelligerent_in_war", Scopes::Country),
    (Scopes::Building.union(Scopes::Character), "combat_units", Scopes::CombatUnit),
    (Scopes::None, "country", Scopes::Country),
    (Scopes::None, "diplomatic_play", Scopes::DiplomaticPlay),
    (Scopes::Country, "enemy_in_diplo_play", Scopes::Country),
    (Scopes::Country, "enemy_in_war", Scopes::Country),
    (Scopes::None, "in_global_list", Scopes::all_but_none()),
    (Scopes::Country, "in_hierarchy", Scopes::Country),
    (Scopes::None, "in_list", Scopes::all_but_none()),
    (Scopes::None, "in_local_list", Scopes::all_but_none()),
    (Scopes::Country, "interest_group", Scopes::InterestGroup),
    (Scopes::Country, "law", Scopes::Law),
    (Scopes::None, "market", Scopes::Market),
    (Scopes::Market, "market_goods", Scopes::MarketGoods),
    (Scopes::Party, "member", Scopes::InterestGroup),
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
    (Scopes::Country, "political_movement", Scopes::PoliticalMovement),
    (Scopes::Country, "potential_party", Scopes::Party),
    (Scopes::InterestGroup, "preferred_law", Scopes::Law),
    (
        Scopes::Country.union(Scopes::CountryDefinition).union(Scopes::State),
        "primary_culture",
        Scopes::Culture,
    ),
    (Scopes::Country, "rival_country", Scopes::Country),
    (
        Scopes::Country.union(Scopes::Front).union(Scopes::InterestGroup),
        "scope_admiral",
        Scopes::Character,
    ),
    (Scopes::Country, "scope_ally", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "scope_building", Scopes::Building),
    (
        Scopes::Country.union(Scopes::Front).union(Scopes::InterestGroup),
        "scope_character",
        Scopes::Character,
    ),
    (Scopes::Market, "scope_country", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "scope_culture", Scopes::Culture),
    (Scopes::Country, "scope_diplomatic_pact", Scopes::DiplomaticPact),
    (Scopes::War, "scope_front", Scopes::Front),
    (
        Scopes::Country.union(Scopes::Front).union(Scopes::InterestGroup),
        "scope_general",
        Scopes::Character,
    ),
    (Scopes::DiplomaticPlay, "scope_initiator_ally", Scopes::Country),
    (
        Scopes::Country.union(Scopes::StrategicRegion),
        "scope_interest_marker",
        Scopes::InterestMarker,
    ),
    (Scopes::DiplomaticPlay, "scope_play_involved", Scopes::Country),
    (
        Scopes::Country.union(Scopes::Front).union(Scopes::InterestGroup),
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
    (Scopes::None, "state", Scopes::State),
    (Scopes::None, "state_region", Scopes::StateRegion),
    (Scopes::Country, "strategic_objective", Scopes::State),
    (Scopes::Country, "subject_or_below", Scopes::Country),
    (Scopes::PoliticalMovement, "supporting_character", Scopes::Character),
    (Scopes::PoliticalMovement, "supporting_interest_group", Scopes::InterestGroup),
    (
        Scopes::Country.union(Scopes::Market).union(Scopes::MarketGoods),
        "trade_route",
        Scopes::TradeRoute,
    ),
];

pub const SCOPE_REMOVED_ITERATOR: &[(&str, &str, &str)] = &[(
    "scope_cobelligerent",
    "1.4.0",
    "replaced with _cobelligerent_in_diplo_play, _cobelligerent_in_war",
)];

pub const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
