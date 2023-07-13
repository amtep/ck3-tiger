#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::fmt::{Display, Formatter};

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::display_choices;
use crate::item::Item;
use crate::modif::{verify_modif_exists, ModifKinds};
use crate::report::{warn_info, ErrorKey};
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

/// LAST UPDATED VIC3 VERSION 1.3.6
pub const None: u64 = 0x0000_0001;
pub const Value: u64 = 0x0000_0002;
pub const Bool: u64 = 0x0000_0004;
pub const Flag: u64 = 0x0000_0008;
#[allow(dead_code)]
pub const Color: u64 = 0x0000_0010;
pub const Country: u64 = 0x0000_0020;
pub const Battle: u64 = 0x0000_0040;
pub const BattleSide: u64 = 0x0000_0080;
pub const Building: u64 = 0x0000_0100;
pub const BuildingType: u64 = 0x0000_0200;
#[allow(dead_code)]
pub const CanalType: u64 = 0x0000_0400;
pub const Character: u64 = 0x0000_0800;
pub const CivilWar: u64 = 0x0000_1000;
pub const CombatUnit: u64 = 0x0000_2000;
pub const CommanderOrder: u64 = 0x0000_4000;
pub const CommanderOrderType: u64 = 0x0000_8000;
#[allow(dead_code)]
pub const CountryCreation: u64 = 0x0001_0000;
pub const CountryDefinition: u64 = 0x0002_0000;
#[allow(dead_code)]
pub const CountryFormation: u64 = 0x0004_0000;
pub const Culture: u64 = 0x0008_0000;
pub const Decree: u64 = 0x0010_0000;
#[allow(dead_code)]
pub const DiplomaticAction: u64 = 0x0020_0000;
pub const DiplomaticPact: u64 = 0x0040_0000;
pub const DiplomaticPlay: u64 = 0x0080_0000;
pub const DiplomaticRelations: u64 = 0x0100_0000;
pub const Front: u64 = 0x0200_0000;
pub const Goods: u64 = 0x0400_0000;
pub const Hq: u64 = 0x0800_0000;
pub const Ideology: u64 = 0x1000_0000;
pub const Institution: u64 = 0x2000_0000;
pub const InstitutionType: u64 = 0x4000_0000;
pub const InterestMarker: u64 = 0x8000_0000;
pub const InterestGroup: u64 = 0x0000_0001_0000_0000;
pub const InterestGroupTrait: u64 = 0x0000_0002_0000_0000;
pub const InterestGroupType: u64 = 0x0000_0004_0000_0000;
pub const Journalentry: u64 = 0x0000_0008_0000_0000;
pub const Law: u64 = 0x0000_0010_0000_0000;
pub const LawType: u64 = 0x0000_0020_0000_0000;
pub const Market: u64 = 0x0000_0040_0000_0000;
pub const MarketGoods: u64 = 0x0000_0080_0000_0000;
#[allow(dead_code)]
pub const Objective: u64 = 0x0000_0100_0000_0000;
pub const Party: u64 = 0x0000_0200_0000_0000;
pub const PoliticalMovement: u64 = 0x0000_0400_0000_0000;
pub const Pop: u64 = 0x0000_0800_0000_0000;
pub const PopType: u64 = 0x0000_1000_0000_0000;
pub const Province: u64 = 0x0000_2000_0000_0000;
pub const Religion: u64 = 0x0000_4000_0000_0000;
#[allow(dead_code)]
pub const ShippingLane: u64 = 0x0000_8000_0000_0000;
pub const State: u64 = 0x0001_0000_0000_0000;
pub const StateRegion: u64 = 0x0002_0000_0000_0000;
#[allow(dead_code)]
pub const StateTrait: u64 = 0x0004_0000_0000_0000;
pub const StrategicRegion: u64 = 0x0008_0000_0000_0000;
pub const Technology: u64 = 0x0010_0000_0000_0000;
#[allow(dead_code)]
pub const TechnologyStatus: u64 = 0x0020_0000_0000_0000;
pub const Theater: u64 = 0x0040_0000_0000_0000;
pub const TradeRoute: u64 = 0x0080_0000_0000_0000;
pub const War: u64 = 0x0100_0000_0000_0000;

pub const ALL: u64 = 0x7fff_ffff_ffff_ffff;
pub const ALL_BUT_NONE: u64 = 0x7fff_ffff_ffff_fffe;
#[allow(dead_code)]
pub const PRIMITIVE: u64 = 0x0000_000e;

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

pub fn scope_to_scope(name: &Token) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_TO_SCOPE {
        if name.is(s) {
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
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
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
        }
    }
    std::option::Option::None
}

/// `name` is without the `every_`, `ordered_`, `random_`, or `any_`
pub fn scope_iterator(name: &Token, data: &Everything) -> Option<(Scopes, Scopes)> {
    for (from, s, to) in SCOPE_ITERATOR {
        if name.is(s) {
            return Some((Scopes::from_bits_truncate(*from), Scopes::from_bits_truncate(*to)));
        }
    }
    for (s, version, explanation) in SCOPE_REMOVED_ITERATOR {
        if name.is(s) {
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
        // "decree_cost"
        // "define"
        // "diplomatic_pact_other_country"
        // "flag"
        "g" | "mg" => data.verify_exists(Item::Goods, arg),
        // "get_ruler_for"
        // "global_var"
        "i" | "ideology" => data.verify_exists(Item::Ideology, arg),
        "ig" => data.verify_exists(Item::InterestGroup, arg),
        // "ig_trait" => data.verify_exists(Item::InterestGroupTrait, arg),
        // "ig_type" => data.verify_exists(Item::InterestGroupType, arg),
        // "infamy_threshold" => data.verify_exists(Item::InfamyThreshold, arg),
        "institution" => data.verify_exists(Item::Institution, arg),
        // "je" => data.verify_exists(Item::Journalentry, arg),
        "law_type" => data.verify_exists(Item::LawType, arg),
        // "local_var"
        // TODO: use the modif type corresponding to the scope this is used in
        "modifier" => verify_modif_exists(arg, data, ModifKinds::all()),
        // "nf" => data.verify_exists(Item::Decree, arg),
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
const SCOPE_TO_SCOPE: &[(u64, &str, u64)] = &[
    (Country | StrategicRegion, "active_diplomatic_play", DiplomaticPlay),
    (TradeRoute, "actor_market", Market),
    (Country, "army_size", Value),
    (Country, "army_size_including_conscripts", Value),
    (Battle, "attacker_side", BattleSide),
    (War, "attacker_warleader", Country),
    (Country | State, "average_expected_sol", Value),
    (Country | State, "average_sol", Value),
    (BattleSide, "battle", Battle),
    (CombatUnit, "building", Building),
    (Country, "building_levels", Value),
    (Country, "cached_ai_coastal_population", Value),
    (Country, "cached_ai_incorporated_coastal_population", Value),
    (Country, "cached_ai_incorporated_population", Value),
    (Country, "cached_ai_overseas_subject_population", Value),
    (Country, "cached_ai_subject_population", Value),
    (Country, "cached_ai_total_population", Value),
    (Country, "cached_ai_unincorporated_coastal_population", Value),
    (Country, "cached_ai_unincorporated_population", Value),
    (Country, "capital", State),
    (PoliticalMovement, "civil_war", CivilWar),
    (Country, "civil_war_origin_country", Country),
    (Country, "colonial_growth_per_colony", Value),
    (Province, "combat_width", Value),
    (CombatUnit, "commander", Character),
    (Province | State, "controller", Country),
    (Country, "country_definition", CountryDefinition),
    (Country, "credit", Value),
    (Character | CombatUnit | Pop, "culture", Culture),
    (Law, "currently_active_law_in_group", Law),
    (Country, "currently_enacting_law", Law),
    (Battle, "defender_side", BattleSide),
    (War, "defender_warleader", Country),
    (CombatUnit, "defense", Value),
    (CombatUnit, "demoralized", Value),
    (PoliticalMovement, "desired_law", LawType),
    (DiplomaticPact, "diplomatic_pact_other_country", Country),
    (TradeRoute, "exporter", Market),
    (DiplomaticPact, "first_country", Country),
    (Battle | Character, "front", Front),
    (Front, "front_length", Value),
    (Country | State, "gdp", Value),
    (None, "global_gdb", Value),
    (Journalentry, "goal_value", Value),
    (MarketGoods | TradeRoute, "goods", Goods),
    (Country, "government_size", Value),
    (Country, "heir", Character),
    (Character | Pop, "home_country", Country),
    (Character, "ideology", Ideology),
    (TradeRoute, "importer", Market),
    (Country, "income", Value),
    (Country, "infamy", Value),
    (DiplomaticPlay, "initiator", Country),
    (Character, "interest_group", InterestGroup),
    (Character, "interest_group_type", InterestGroupType),
    (Institution, "investment", Value),
    (Institution, "investment_max", Value),
    (None, "is_setup", Value),
    (Province | State, "land_hq", Hq),
    (InterestGroup, "leader", Character),
    (Country, "legitimacy", Value),
    (Building | TradeRoute, "level", Value),
    (CombatUnit, "manpower", Value),
    (Country | Building | Market | MarketGoods | Province | State | StateRegion, "market", Market),
    (Country, "market_capital", State),
    (CombatUnit, "mobilization", Value),
    (Party, "momentum", Value),
    (CombatUnit, "morale", Value),
    (Province, "naval_hq", Hq),
    (Country, "navy_size", Value),
    (None, "no", Bool),
    (Country, "num_active_declared_interests", Value),
    (Country, "num_active_interests", Value),
    (Country, "num_active_natural_interests", Value),
    (Country, "num_active_plays", Value),
    (Country, "num_admirals", Value),
    (Country, "num_alliances", Value),
    (Character, "num_battalions", Value),
    (Country, "num_characters", Value),
    (Country, "num_colony_projects", Value),
    (Character, "num_commanded_units", Value),
    (Country, "num_commanders", Value),
    (Country, "num_convoys_available", Value),
    (Country, "num_convoys_required", Value),
    (Country, "num_declared_interests", Value),
    (Country, "num_defensive_pacts", Value),
    (MarketGoods, "num_export_trade_routes", Value),
    (Hq, "num_garrison_units", Value),
    (Country, "num_generals", Value),
    (MarketGoods, "num_import_trade_routes", Value),
    (Country, "num_income_transfer_pacts", Value),
    (Country, "num_incorporated_states", Value),
    (Country, "num_interests", Value),
    (Character, "num_mobilized_battalions", Value),
    (Country, "num_natural_interests", Value),
    (Country, "num_obligations_earned", Value),
    (Country, "num_pending_events", Value),
    (Country, "num_politicians", Value),
    (Country, "num_positive_relations", Value),
    (Front | State | StateRegion, "num_provinces", Value),
    (Country, "num_queued_constructions", Value),
    (Country, "num_queued_government_constructions", Value),
    (Country, "num_queued_private_constructions", Value),
    (Country, "num_rivals", Value),
    (Country, "num_ruling_igs", Value),
    (Country, "num_states", Value),
    (Country, "num_trade_routes", Value),
    (Country, "num_treaty_ports", Value),
    (Country, "num_unincorporated_states", Value),
    (Character, "num_units", Value),
    (Character, "num_units_not_in_battle", Value),
    (CombatUnit, "offense", Value),
    (Character, "opposing_commander", Character),
    (Country, "overlord", Country),
    (
        Country
            | Building
            | Character
            | CombatUnit
            | Decree
            | Institution
            | InterestMarker
            | InterestGroup
            | Journalentry
            | Law
            | Market
            | MarketGoods
            | Pop
            | Province
            | State
            | TradeRoute,
        "owner",
        Country,
    ),
    (Market, "participants", Value),
    (InterestGroup, "party", Party),
    (None, "player", Country), // TODO "do not use this outside tutorial"
    (DiplomaticRelations, "player_owed_obligation_days_left", Value),
    (Character | CivilWar, "political_movement", PoliticalMovement),
    (Pop, "pop_weight_modifier_scale", Value),
    (Character, "popularity", Value),
    (State, "population_below_expected_sol", Value),
    (BattleSide, "province", Province),
    (
        Building | DiplomaticPlay | InterestMarker | Province | State | StateRegion,
        "region",
        StrategicRegion,
    ),
    (Country | Character | CountryDefinition | Pop, "religion", Religion),
    (Country, "ruler", Character),
    (DiplomaticRelations, "scope_relations", Value),
    (DiplomaticRelations, "scope_tension", Value),
    (DiplomaticPact, "second_country", Country),
    (BuildingType, "slaves_role", PopType),
    (Building | Market | Pop | Province, "state", State),
    (State, "state_region", StateRegion),
    (Character, "supply", Value),
    (DiplomaticPlay | Journalentry, "target", ALL), // TODO: scope type?
    (TradeRoute, "target_market", Market),
    (Country, "technology_being_researched", Technology),
    (Country, "techs_researched", Value),
    (Country, "top_overlord", Country),
    (Market, "trade_center", State),
    (Building, "training_rate", Value),
    // TODO: special case for the scope types here
    (
        Building | CommanderOrder | Institution | InterestGroup | Law,
        "type",
        BuildingType | CommanderOrderType | InstitutionType | InterestGroupType | LawType,
    ),
    (DiplomaticPlay, "war", War),
    (Pop, "workplace", Building),
    (None, "yes", Bool),
];

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `event_targets.log` from the game data dumps
/// These are absolute scopes (like character:100000) and scope transitions that require
/// a key (like `root.cp:councillor_steward`)
/// TODO: add the Item type here, so that it can be checked for existence.
const SCOPE_FROM_PREFIX: &[(u64, &str, u64)] = &[
    (Country, "active_law", Law),
    (Country, "ai_army_comparison", Value),
    (Country, "ai_gdp_comparison", Value),
    (Country, "ai_ideological_opinion", Value),
    (Country, "ai_navy_comparison", Value),
    (None, "array_define", Value),
    (Front, "average_defense", Value),
    (Front, "average_offense", Value),
    (State, "b", Building),
    (None, "c", Country),
    (None, "cd", CountryDefinition),
    (None, "cu", Culture),
    (Country, "decree_cost", Value),
    (None, "define", Value),
    (None, "flag", Flag),
    (None, "g", Goods),
    (Country, "get_ruler_for", Character),
    (None, "global_var", ALL),
    (None, "i", Ideology),
    (None, "ideology", Ideology), // TODO difference with i:
    (Country, "ig", InterestGroup),
    (None, "ig_trait", InterestGroupTrait),
    (None, "ig_type", InterestGroupType),
    (None, "infamy_threshold", Value),
    (Country, "institution", Institution),
    (Country, "je", Journalentry),
    (Country, "je_tutorial", Journalentry),
    (None, "law_type", LawType),
    (None, "local_var", ALL),
    (Market, "mg", MarketGoods),
    (Country | Building | Character | InterestGroup | Market | State, "modifier", Value | Bool),
    (State, "nf", Decree),
    (Country, "num_alliances_and_defensive_pacts_with_allies", Value),
    (Country, "num_alliances_and_defensive_pacts_with_rivals", Value),
    (Front, "num_defending_battalions", Value),
    (Front, "num_enemy_units", Value),
    (Country, "num_mutual_trade_route_levels_with_country", Value),
    (Country, "num_pending_events", Value),
    (Front, "num_total_battalions", Value),
    (None, "p", Province),
    (None, "pop_type", PopType),
    (Country, "py", Party),
    (None, "rank_value", Value),
    (StateRegion, "region_state", State), // undocumented
    (None, "rel", Religion),
    (Country, "relations", Value),
    (None, "relations_threshold", Value),
    (None, "s", StateRegion),
    (None, "scope", ALL),
    (None, "sr", StrategicRegion),
    (Country, "tension", Value),
    (None, "tension_threshold", Value),
    (ALL, "var", ALL),
];

/// LAST UPDATED VIC3 VERSION 1.3.6
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const SCOPE_ITERATOR: &[(u64, &str, u64)] = &[
    (Country, "active_party", Party),
    (None, "character", Character),
    (None, "character_in_exile_pool", Character),
    (None, "character_in_void", Character),
    (Country, "civil_war", CivilWar),
    (Building | Character, "combat_units", CombatUnit),
    (None, "country", Country),
    (None, "diplomatic_play", DiplomaticPlay),
    (None, "in_global_list", ALL_BUT_NONE),
    (Country, "in_hierarchy", Country),
    (None, "in_list", ALL_BUT_NONE),
    (None, "in_local_list", ALL_BUT_NONE),
    (Country, "interest_group", InterestGroup),
    (Country, "law", Law),
    (None, "market", Market),
    (Market, "market_goods", MarketGoods),
    (Party, "member", InterestGroup),
    (Country | State | StateRegion | StrategicRegion, "neighbouring_state", State),
    (Country, "overlord_or_above", Country),
    (DiplomaticPact, "participant", Country),
    (Country, "political_movement", PoliticalMovement),
    (Country, "potential_party", Party),
    (InterestGroup, "preferred_law", Law),
    (Country | CountryDefinition | State, "primary_culture", Culture),
    (Country, "rival_country", Country),
    (Country | Front | InterestGroup, "scope_admiral", Character),
    (Country, "scope_ally", Country),
    (Country | State, "scope_building", Building),
    (Country | Front | InterestGroup, "scope_character", Character),
    (Country, "scope_cobelligerent", Country),
    (Market, "scope_country", Country),
    (Country | State, "scope_culture", Culture),
    (Country, "scope_diplomatic_pact", DiplomaticPact),
    (War, "scope_front", Front),
    (Country | Front | InterestGroup, "scope_general", Character),
    (DiplomaticPlay, "scope_initiator_ally", Country),
    (Country | StrategicRegion, "scope_interest_marker", InterestMarker),
    (DiplomaticPlay, "scope_play_involved", Country),
    (Country | Front | InterestGroup, "scope_politician", Character),
    (Country | Culture | InterestGroup | State, "scope_pop", Pop),
    (Country | Front | StateRegion | StrategicRegion | Theater, "scope_state", State),
    (DiplomaticPlay, "scope_target_ally", Country),
    (Country, "scope_theater", Theater),
    (Country, "scope_violate_sovereignty_interested_parties", Country),
    (Country, "scope_violate_sovereignty_wars", War),
    (Country, "scope_war", War),
    (None, "state", State),
    (None, "state_region", StateRegion),
    (Country, "strategic_objective", State),
    (Country, "subject_or_below", Country),
    (PoliticalMovement, "supporting_character", Character),
    (PoliticalMovement, "supporting_interest_group", InterestGroup),
    (Country | Market | MarketGoods, "trade_route", TradeRoute),
];

const SCOPE_REMOVED_ITERATOR: &[(&str, &str, &str)] = &[];

const SCOPE_TO_SCOPE_REMOVED: &[(&str, &str, &str)] = &[];
