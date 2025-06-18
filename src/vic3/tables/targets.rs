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

/// LAST UPDATED VIC3 VERSION 1.7.6
/// See `event_targets.log` from the game data dumps
/// These are scope transitions that can be chained like `root.joined_faction.faction_leader`
const SCOPE_TO_SCOPE: &[(Scopes, &str, Scopes)] = &[
    (Scopes::Treaty.union(Scopes::TreatyOptions), "amended_treaty", Scopes::Treaty),
    (Scopes::Country, "army_size", Scopes::Value),
    (Scopes::Country, "army_size_including_conscripts", Scopes::Value),
    (Scopes::Battle, "attacker_side", Scopes::BattleSide),
    (Scopes::War, "attacker_warleader", Scopes::Country),
    (Scopes::Country.union(Scopes::State), "average_expected_sol", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "average_sol", Scopes::Value),
    (Scopes::Goods, "base_price", Scopes::Value),
    (Scopes::BattleSide, "battle", Scopes::Battle),
    (Scopes::Treaty.union(Scopes::TreatyOptions), "binding_period", Scopes::Value),
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
    (Scopes::Treaty, "enforced_on_country", Scopes::Country),
    (Scopes::Treaty, "enforcer_country", Scopes::Country),
    (Scopes::Company, "executive", Scopes::Character),
    (Scopes::Country, "expenses", Scopes::Value),
    (
        Scopes::DiplomaticPact
            .union(Scopes::TreatyArticle)
            .union(Scopes::TreatyOptions)
            .union(Scopes::TreatyArticleOptions)
            .union(Scopes::Treaty),
        "first_country",
        Scopes::Country,
    ),
    (Scopes::Country, "fixed_expenses", Scopes::Value),
    (
        Scopes::Battle
            .union(Scopes::Character)
            .union(Scopes::Province)
            .union(Scopes::Invasion)
            .union(Scopes::MilitaryFormation),
        "front",
        Scopes::Front,
    ),
    (Scopes::Front, "front_length", Scopes::Value),
    (Scopes::Country.union(Scopes::State), "gdp", Scopes::Value),
    (Scopes::None, "global_gdp", Scopes::Value),
    (Scopes::JournalEntry, "goal_value", Scopes::Value),
    (Scopes::MarketGoods.union(Scopes::StateGoods), "goods", Scopes::Goods),
    (Scopes::Country.union(Scopes::PoliticalMovement), "government_size", Scopes::Value),
    (Scopes::Country, "heir", Scopes::Character),
    (Scopes::MilitaryFormation, "highest_ranked_commander", Scopes::Character),
    (Scopes::Character.union(Scopes::Pop), "home_country", Scopes::Country),
    (Scopes::MilitaryFormation, "home_hq", Scopes::Hq),
    (Scopes::PowerBloc, "identity", Scopes::PowerBlocIdentity),
    (Scopes::Character, "ideology", Scopes::Ideology),
    (Scopes::Country, "imposed_law", Scopes::Law),
    (Scopes::Country.union(Scopes::Law), "imposer_of_law", Scopes::Country),
    (Scopes::Country, "income", Scopes::Value),
    (Scopes::Country, "income_transfer_expenses", Scopes::Value),
    (Scopes::Country, "income_transfer_relative_expenses", Scopes::Value),
    (Scopes::Country, "infamy", Scopes::Value),
    (Scopes::DiplomaticPlay, "initiator", Scopes::Country),
    (
        Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions),
        "input_building_type",
        Scopes::BuildingType,
    ),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_company", Scopes::Company),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_country", Scopes::Country),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_goods", Scopes::Goods),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_law", Scopes::LawType),
    (
        Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions),
        "input_market_goods",
        Scopes::MarketGoods,
    ),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_quantity", Scopes::Value),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "input_state", Scopes::State),
    (
        Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions),
        "input_strategic_region",
        Scopes::StrategicRegion,
    ),
    (Scopes::Character, "interest_group", Scopes::InterestGroup),
    (Scopes::Character, "interest_group_type", Scopes::InterestGroupType),
    (Scopes::Front, "invasion", Scopes::Invasion),
    (Scopes::Invasion, "invasion_attacker", Scopes::Country),
    (Scopes::Invasion, "invasion_defender", Scopes::Country),
    (Scopes::Institution, "investment", Scopes::Value),
    (Scopes::Institution, "investment_max", Scopes::Value),
    (Scopes::Country, "investment_pool_income", Scopes::Value),
    (Scopes::None, "is_setup", Scopes::Value),
    (Scopes::None, "je_tutorial", Scopes::JournalEntry),
    (Scopes::Province.union(Scopes::State), "land_controller_hq", Scopes::Hq),
    (Scopes::Province.union(Scopes::State), "land_hq", Scopes::Hq),
    (Scopes::InterestGroup, "leader", Scopes::Character),
    (Scopes::Country, "legitimacy", Scopes::Value),
    (Scopes::Building, "level", Scopes::Value),
    (Scopes::Building, "level_after_queued_constructions", Scopes::Value),
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
    (Scopes::Hq, "num_garrison_units", Scopes::Value),
    (Scopes::Country, "num_generals", Scopes::Value),
    (Scopes::Country, "num_income_transfer_pacts", Scopes::Value),
    (Scopes::Country, "num_income_transfer_treaty_articles", Scopes::Value),
    (Scopes::Country, "num_income_transfers", Scopes::Value),
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
    (Scopes::Country, "num_unincorporated_states", Scopes::Value),
    (Scopes::Character.union(Scopes::MilitaryFormation), "num_units", Scopes::Value),
    (Scopes::Character.union(Scopes::MilitaryFormation), "num_units_in_battle", Scopes::Value),
    (Scopes::Character.union(Scopes::MilitaryFormation), "num_units_not_in_battle", Scopes::Value),
    (Scopes::Character, "num_units_share", Scopes::Value),
    (Scopes::Country, "num_world_market_hub_trade_center_levels", Scopes::Value),
    (Scopes::NewCombatUnit, "offense", Scopes::Value),
    (Scopes::Character, "opposing_commander", Scopes::Character),
    (Scopes::BattleSide, "opposite_battle_side", Scopes::BattleSide),
    (Scopes::Country, "overlord", Scopes::Country),
    (
        Scopes::Country
            .union(Scopes::Building)
            .union(Scopes::Character)
            .union(Scopes::NewCombatUnit)
            .union(Scopes::Company)
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
            .union(Scopes::MilitaryFormation)
            .union(Scopes::PoliticalMovement),
        "owner",
        Scopes::Country,
    ),
    (Scopes::Country, "owning_company", Scopes::Company),
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
    (Scopes::TreatyOptions.union(Scopes::Treaty), "remaining_binding_period", Scopes::Value),
    (Scopes::Country, "ruler", Scopes::Character),
    (Scopes::DiplomaticRelations, "scope_relations", Scopes::Value),
    (Scopes::DiplomaticRelations, "scope_tension", Scopes::Value),
    (
        Scopes::DiplomaticPact
            .union(Scopes::TreatyArticle)
            .union(Scopes::TreatyOptions)
            .union(Scopes::TreatyArticleOptions)
            .union(Scopes::Treaty),
        "second_country",
        Scopes::Country,
    ),
    (
        Scopes::MilitaryFormation.union(Scopes::TreatyArticle),
        "shipping_lane",
        Scopes::ShippingLanes,
    ),
    (Scopes::BuildingType, "slaves_role", Scopes::PopType),
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "source_country", Scopes::Country),
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
    (Scopes::TreatyArticle.union(Scopes::TreatyArticleOptions), "target_country", Scopes::Country),
    (Scopes::Country, "technology_being_researched", Scopes::Technology),
    (Scopes::Country, "techs_researched", Scopes::Value),
    (Scopes::BattleSide.union(Scopes::State), "theater", Scopes::Theater),
    (Scopes::Country, "top_overlord", Scopes::Country),
    (Scopes::Market, "trade_center", Scopes::State),
    (Scopes::Building, "training_rate", Scopes::Value),
    (Scopes::TreatyArticle, "treaty", Scopes::Treaty),
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
        (Scopes::Country, "ai_transit_rights_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::MarketGoods, "ai_treaty_export_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_treaty_fairness", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::MarketGoods, "ai_treaty_import_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::State, "ai_treaty_port_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::MarketGoods, "ai_treaty_trade_value", Scopes::Value, Scope(Scopes::Country)),
        (Scopes::Country, "ai_treaty_value", Scopes::Value, Scope(Scopes::Country)),
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
        (Scopes::None, "define", Scopes::Value, Item(Item::Define)),
        (Scopes::None, "flag", Scopes::Flag, UncheckedValue),
        (Scopes::None, "g", Scopes::Goods, Item(Item::Goods)),
        (Scopes::Country, "get_ruler_for", Scopes::Character, Item(Item::TransferOfPower)),
        // TODO: figure out how this one works
        (Scopes::None, "global_productivity", Scopes::Value, Scope(Scopes::Value)),
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
        (Scopes::InterestGroup, "lobby_join_weight", Scopes::Value, Scope(Scopes::PoliticalLobby)),
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
        (Scopes::Country, "mutual_trade_value_with_country", Scopes::Value, Scope(Scopes::Country)),
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
    ("supply", "1.6", ""),
    ("active_diplomatic_play", "1.7", ""),
    ("actor_market", "1.9", "replaced by world market system"),
    ("commander", "1.9", ""),
    ("exporter", "1.9", "replaced by world market system"),
    ("importer", "1.9", "replaced by world market system"),
    ("naval_invasion_attacker", "1.9", "replaced with `invasion_attacker`"),
    ("naval_invasion_defender", "1.9", "replaced with `invasion_defender`"),
    ("num_export_trade_routes", "1.9", "replaced by world market system"),
    ("num_import_trade_routes", "1.9", "replaced by world market system"),
    ("num_mutual_trade_route_levels_with_country", "1.9", "replaced by world market system"),
    ("num_trade_routes", "1.9", "replaced by world market system"),
    ("num_treaty_ports", "1.9", "replaced by world market system"),
    ("target_market", "1.9", "replaced by world market system"),
];
