use std::sync::LazyLock;

use crate::helpers::TigerHashMap;
use crate::scopes::Scopes;

#[inline]
pub fn iterator(name: &str) -> Option<(Scopes, Scopes)> {
    ITERATOR_MAP.get(name).copied()
}

static ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in ITERATOR.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// LAST UPDATED VIC3 VERSION 1.8.1
/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const ITERATOR: &[(Scopes, &str, Scopes)] = &[
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
    (Scopes::None, "decentralized_country", Scopes::Country),
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
    (Scopes::Treaty, "scope_article", Scopes::TreatyArticle),
    (
        Scopes::TreatyOptions.union(Scopes::Treaty),
        "scope_article_option",
        Scopes::TreatyArticleOptions,
    ),
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
    (Scopes::Company, "scope_regional_hqs", Scopes::Building),
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
    (Scopes::Country, "scope_treaty", Scopes::Treaty),
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
    (Scopes::None, "treaty", Scopes::Treaty),
    (Scopes::Country, "valid_mass_migration_culture", Scopes::Culture),
    (Scopes::War, "war_participant", Scopes::Country),
];

pub fn iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const ITERATOR_REMOVED: &[(&str, &str, &str)] = &[
    (
        "scope_cobelligerent",
        "1.4.0",
        "replaced with _cobelligerent_in_diplo_play, _cobelligerent_in_war",
    ),
    ("supporting_interest_group", "1.8", "replaced with `_influenced_interest_group`"),
    ("trade_route", "1.9", "replaced by world market system"),
];
