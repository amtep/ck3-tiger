#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::{warn_info, ErrorKey, Severity};
use crate::token::Token;
use crate::trigger::validate_target;

bitflags! {
    /// LAST UPDATED VIC3 VERSION 1.3.6
    /// See `event_scopes.log` from the game data dumps.
    /// Keep in sync with the module constants below.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Scopes: u64 {
        const None = 0x0000_0001;
        const Value = 0x0000_0002;
        const Bool = 0x0000_0004;
        const Flag = 0x0000_0008;
        const Color = 0x0000_0010;
        const Country = 0x0000_0020;
        const Battle = 0x0000_0040;
        const BattleSide = 0x0000_0080;
        const Building = 0x0000_0100;
        const BuildingType = 0x0000_0200;
        const CanalType = 0x0000_0400;
        const Character = 0x0000_0800;
        const CivilWar = 0x0000_1000;
        const CombatUnit = 0x0000_2000;
        const CommanderOrder = 0x0000_4000;
        const CommanderOrderType = 0x0000_8000;
        const CountryCreation = 0x0001_0000;
        const CountryDefinition = 0x0002_0000;
        const CountryFormation = 0x0004_0000;
        const Culture = 0x0008_0000;
        const Decree = 0x0010_0000;
        const DiplomaticAction = 0x0020_0000;
        const DiplomaticPact = 0x0040_0000;
        const DiplomaticPlay = 0x0080_0000;
        const DiplomaticRelations = 0x0100_0000;
        const Front = 0x0200_0000;
        const Goods = 0x0400_0000;
        const Hq = 0x0800_0000;
        const Ideology = 0x1000_0000;
        const Institution = 0x2000_0000;
        const InstitutionType = 0x4000_0000;
        const InterestMarker = 0x8000_0000;
        const InterestGroup = 0x0000_0001_0000_0000;
        const InterestGroupTrait = 0x0000_0002_0000_0000;
        const InterestGroupType = 0x0000_0004_0000_0000;
        const Journalentry = 0x0000_0008_0000_0000;
        const Law = 0x0000_0010_0000_0000;
        const LawType = 0x0000_0020_0000_0000;
        const Market = 0x0000_0040_0000_0000;
        const MarketGoods = 0x0000_0080_0000_0000;
        const Objective = 0x0000_0100_0000_0000;
        const Party = 0x0000_0200_0000_0000;
        const PoliticalMovement = 0x0000_0400_0000_0000;
        const Pop = 0x0000_0800_0000_0000;
        const PopType = 0x0000_1000_0000_0000;
        const Province = 0x0000_2000_0000_0000;
        const Religion = 0x0000_4000_0000_0000;
        const ShippingLane = 0x0000_8000_0000_0000;
        const State = 0x0001_0000_0000_0000;
        const StateRegion = 0x0002_0000_0000_0000;
        const StateTrait = 0x0004_0000_0000_0000;
        const StrategicRegion = 0x0008_0000_0000_0000;
        const Technology = 0x0010_0000_0000_0000;
        const TechnologyStatus = 0x0020_0000_0000_0000;
        const Theater = 0x0040_0000_0000_0000;
        const TradeRoute = 0x0080_0000_0000_0000;
        const War = 0x0100_0000_0000_0000;
    }
}

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

pub fn scope_to_scope(name: &Token, inscopes: Scopes) -> Option<(Scopes, Scopes)> {
    let name_lc = name.as_str().to_lowercase();

    for (from, s, to) in SCOPE_TO_SCOPE {
        if name_lc == *s {
            // Special case for "type" because it goes from specific scope types to specific other scope types.
            if *s == "type" {
                let mut outscopes = Scopes::empty();
                if inscopes.contains(Scopes::Building) {
                    outscopes |= Scopes::BuildingType;
                }
                if inscopes.contains(Scopes::CommanderOrder) {
                    outscopes |= Scopes::CommanderOrderType;
                }
                if inscopes.contains(Scopes::Institution) {
                    outscopes |= Scopes::InstitutionType;
                }
                if inscopes.contains(Scopes::InterestGroup) {
                    outscopes |= Scopes::InterestGroupType;
                }
                if inscopes.contains(Scopes::Law) {
                    outscopes |= Scopes::LawType;
                }
                if outscopes.is_empty() {
                    outscopes = *to;
                }
                return Some((*from, outscopes));
            }
            return Some((*from, *to));
        }
    }
    for (s, version, explanation) in SCOPE_TO_SCOPE_REMOVED {
        if name.is(s) {
            let msg = format!("`{name}` was removed in {version}");
            warn_info(name, ErrorKey::Removed, &msg, explanation);
            return Some((Scopes::all(), Scopes::all_but_none()));
        }
    }
    std::option::Option::None
}

pub fn scope_prefix(prefix: &str) -> Option<(Scopes, Scopes)> {
    // Case insensitivity has been verified for at least S: in vic3
    let prefix = prefix.to_lowercase();
    for (from, s, to) in SCOPE_FROM_PREFIX {
        if *s == prefix {
            return Some((*from, *to));
        }
    }
    std::option::Option::None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    let name_lc = name.as_str().to_lowercase();
    for (from, s, to) in SCOPE_ITERATOR {
        if name_lc == *s {
            return Some((*from, *to));
        }
    }
    for (s, version, explanation) in SCOPE_REMOVED_ITERATOR {
        if name_lc == *s {
            let msg = format!("`{name}` iterators were removed in {version}");
            warn_info(name, ErrorKey::Removed, &msg, explanation);
            return Some((Scopes::all(), Scopes::all()));
        }
    }
    if data.scripted_lists.exists(name.as_str()) {
        return data.scripted_lists.base(name).and_then(|base| scope_iterator(base, data));
    }
    std::option::Option::None
}

impl Display for Scopes {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if *self == Scopes::all() {
            write!(f, "any scope")
        } else if *self == Scopes::primitive() {
            write!(f, "any primitive scope")
        } else if *self == Scopes::non_primitive() {
            write!(f, "non-primitive scope")
        } else if *self == Scopes::all_but_none() {
            write!(f, "any except none scope")
        } else {
            let mut vec = Vec::new();
            if self.contains(Scopes::None) {
                vec.push("none");
            }
            if self.contains(Scopes::Value) {
                vec.push("value");
            }
            if self.contains(Scopes::Bool) {
                vec.push("bool");
            }
            if self.contains(Scopes::Flag) {
                vec.push("flag");
            }
            if self.contains(Scopes::Color) {
                vec.push("color");
            }
            if self.contains(Scopes::Country) {
                vec.push("country");
            }
            if self.contains(Scopes::Battle) {
                vec.push("battle");
            }
            if self.contains(Scopes::BattleSide) {
                vec.push("battle side");
            }
            if self.contains(Scopes::Building) {
                vec.push("building");
            }
            if self.contains(Scopes::BuildingType) {
                vec.push("building type");
            }
            if self.contains(Scopes::CanalType) {
                vec.push("canal type");
            }
            if self.contains(Scopes::Character) {
                vec.push("character");
            }
            if self.contains(Scopes::CivilWar) {
                vec.push("civil war");
            }
            if self.contains(Scopes::CombatUnit) {
                vec.push("combat unit");
            }
            if self.contains(Scopes::CommanderOrder) {
                vec.push("commander order");
            }
            if self.contains(Scopes::CommanderOrderType) {
                vec.push("commander order_type");
            }
            if self.contains(Scopes::CountryCreation) {
                vec.push("country creation");
            }
            if self.contains(Scopes::CountryDefinition) {
                vec.push("country definition");
            }
            if self.contains(Scopes::CountryFormation) {
                vec.push("country formation");
            }
            if self.contains(Scopes::Culture) {
                vec.push("culture");
            }
            if self.contains(Scopes::Decree) {
                vec.push("decree");
            }
            if self.contains(Scopes::DiplomaticAction) {
                vec.push("diplomatic action");
            }
            if self.contains(Scopes::DiplomaticPact) {
                vec.push("diplomatic pact");
            }
            if self.contains(Scopes::DiplomaticPlay) {
                vec.push("diplomatic play");
            }
            if self.contains(Scopes::DiplomaticRelations) {
                vec.push("diplomatic relations");
            }
            if self.contains(Scopes::Front) {
                vec.push("front");
            }
            if self.contains(Scopes::Goods) {
                vec.push("goods");
            }
            if self.contains(Scopes::Hq) {
                vec.push("hq");
            }
            if self.contains(Scopes::Ideology) {
                vec.push("ideology");
            }
            if self.contains(Scopes::Institution) {
                vec.push("institution");
            }
            if self.contains(Scopes::InstitutionType) {
                vec.push("institution type");
            }
            if self.contains(Scopes::InterestMarker) {
                vec.push("interest marker");
            }
            if self.contains(Scopes::InterestGroup) {
                vec.push("interest group");
            }
            if self.contains(Scopes::InterestGroupTrait) {
                vec.push("interest group_trait");
            }
            if self.contains(Scopes::InterestGroupType) {
                vec.push("interest group_type");
            }
            if self.contains(Scopes::Journalentry) {
                vec.push("journalentry");
            }
            if self.contains(Scopes::Law) {
                vec.push("law");
            }
            if self.contains(Scopes::LawType) {
                vec.push("law type");
            }
            if self.contains(Scopes::Market) {
                vec.push("market");
            }
            if self.contains(Scopes::MarketGoods) {
                vec.push("market goods");
            }
            if self.contains(Scopes::Objective) {
                vec.push("objective");
            }
            if self.contains(Scopes::Party) {
                vec.push("party");
            }
            if self.contains(Scopes::PoliticalMovement) {
                vec.push("political movement");
            }
            if self.contains(Scopes::Pop) {
                vec.push("pop");
            }
            if self.contains(Scopes::PopType) {
                vec.push("pop type");
            }
            if self.contains(Scopes::Province) {
                vec.push("province");
            }
            if self.contains(Scopes::Religion) {
                vec.push("religion");
            }
            if self.contains(Scopes::ShippingLane) {
                vec.push("shipping lane");
            }
            if self.contains(Scopes::State) {
                vec.push("state");
            }
            if self.contains(Scopes::StateRegion) {
                vec.push("state region");
            }
            if self.contains(Scopes::StateTrait) {
                vec.push("state trait");
            }
            if self.contains(Scopes::StrategicRegion) {
                vec.push("strategic region");
            }
            if self.contains(Scopes::Technology) {
                vec.push("technology");
            }
            if self.contains(Scopes::TechnologyStatus) {
                vec.push("technology status");
            }
            if self.contains(Scopes::Theater) {
                vec.push("theater");
            }
            if self.contains(Scopes::TradeRoute) {
                vec.push("trade route");
            }
            if self.contains(Scopes::War) {
                vec.push("war");
            }
            display_choices(f, &vec, "or")
        }
    }
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
        // "active_law"
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
        // "py" => data.verify_exists(Item::Party, arg),
        // "rank_value" =>
        "rel" => data.verify_exists(Item::Religion, arg),
        // "relations_threshold" =>
        "s" => data.verify_exists(Item::StateRegion, arg),
        // "scope"
        "sr" => data.verify_exists(Item::StrategicRegion, arg),
        // "tension_threshold" =>
        // "var"
        &_ => (),
    }
}

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
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
    // TODO: special case for the scope types here
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

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
/// TODO: add the Item type here, so that it can be checked for existence.
const SCOPE_FROM_PREFIX: &[(Scopes, &str, Scopes)] = &[
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
    (Scopes::Country, "je_tutorial", Scopes::Journalentry),
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

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Country, "active_party", Scopes::Party),
    (Scopes::None, "character", Scopes::Character),
    (Scopes::None, "character_in_exile_pool", Scopes::Character),
    (Scopes::None, "character_in_void", Scopes::Character),
    (Scopes::Country, "civil_war", Scopes::CivilWar),
    (Scopes::Building.union(Scopes::Character), "combat_units", Scopes::CombatUnit),
    (Scopes::None, "country", Scopes::Country),
    (Scopes::None, "diplomatic_play", Scopes::DiplomaticPlay),
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
    (Scopes::Country, "scope_cobelligerent", Scopes::Country),
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

const SCOPE_REMOVED_ITERATOR: &[(&str, &str, &str)] = &[];

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
