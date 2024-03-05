use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;

use crate::datatype::{Arg, Args, CaseInsensitiveStr, Datatype, Vic3Datatype};
use crate::helpers::BiFnvHashMap;
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use Vic3Datatype::*;

pub static LOWERCASE_DATATYPE_SET: Lazy<FnvHashSet<CaseInsensitiveStr>> = Lazy::new(|| {
    let mut set = FnvHashSet::default();

    for (name, _, _) in GLOBAL_PROMOTES.iter().copied() {
        set.insert(CaseInsensitiveStr(name));
    }

    for (name, _, _) in GLOBAL_FUNCTIONS.iter().copied() {
        set.insert(CaseInsensitiveStr(name));
    }

    for (name, _, _, _) in PROMOTES.iter().copied() {
        set.insert(CaseInsensitiveStr(name));
    }

    for (name, _, _, _) in FUNCTIONS.iter().copied() {
        set.insert(CaseInsensitiveStr(name));
    }
    set
});

pub static DATATYPE_AND_SCOPE_MAP: Lazy<BiFnvHashMap<Datatype, Scopes>> = Lazy::new(|| {
    let mut map = BiFnvHashMap::default();
    for (datatype, scope) in DATATYPE_AND_SCOPE.iter().copied() {
        map.insert(datatype, scope);
    }
    map
});

pub static GLOBAL_PROMOTES_MAP: Lazy<FnvHashMap<&'static str, (Args, Datatype)>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::default();
        for (name, args, datatype) in GLOBAL_PROMOTES.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

pub static GLOBAL_FUNCTIONS_MAP: Lazy<FnvHashMap<&'static str, (Args, Datatype)>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::default();
        for (name, args, datatype) in GLOBAL_FUNCTIONS.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

pub static PROMOTES_MAP: Lazy<FnvHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in PROMOTES.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

pub static FUNCTIONS_MAP: Lazy<FnvHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in FUNCTIONS.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });
// The include/ files are converted from the game's data_type_* output files.

// TODO: find the right datatypes for the commented out ones
const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
    (Vic3(Country), Scopes::Country),
    (Vic3(Battle), Scopes::Battle),
    // (Vic3(BattleSide),Scopes::BattleSide),
    (Vic3(Building), Scopes::Building),
    (Vic3(BuildingType), Scopes::BuildingType),
    (Vic3(CanalType), Scopes::CanalType),
    (Vic3(Character), Scopes::Character),
    (Vic3(CivilWar), Scopes::CivilWar),
    (Vic3(CombatUnit), Scopes::CombatUnit),
    (Vic3(CommanderOrderType), Scopes::CommanderOrderType),
    (Vic3(CountryCreation), Scopes::CountryCreation),
    (Vic3(CountryDefinition), Scopes::CountryDefinition),
    (Vic3(CountryFormation), Scopes::CountryFormation),
    (Vic3(Culture), Scopes::Culture),
    (Vic3(Decree), Scopes::Decree),
    (Vic3(DiplomaticAction), Scopes::DiplomaticAction),
    (Vic3(DiplomaticPact), Scopes::DiplomaticPact),
    (Vic3(DiplomaticPlay), Scopes::DiplomaticPlay),
    (Vic3(DiplomaticRelations), Scopes::DiplomaticRelations),
    (Vic3(Front), Scopes::Front),
    (Vic3(Goods), Scopes::Goods),
    (Vic3(Hq), Scopes::Hq),
    (Vic3(Ideology), Scopes::Ideology),
    (Vic3(Institution), Scopes::Institution),
    (Vic3(InstitutionType), Scopes::InstitutionType),
    // (Vic3(InterestMarker),Scopes::InterestMarker),
    (Vic3(InterestGroup), Scopes::InterestGroup),
    (Vic3(InterestGroupTrait), Scopes::InterestGroupTrait),
    // (Vic3(InterestGroupType),Scopes::InterestGroupType),
    (Vic3(JournalEntry), Scopes::JournalEntry),
    (Vic3(Law), Scopes::Law),
    (Vic3(LawType), Scopes::LawType),
    (Vic3(Market), Scopes::Market),
    (Vic3(MarketGoods), Scopes::MarketGoods),
    (Vic3(Objective), Scopes::Objective),
    (Vic3(Party), Scopes::Party),
    (Vic3(PoliticalMovement), Scopes::PoliticalMovement),
    (Vic3(Pop), Scopes::Pop),
    (Vic3(PopType), Scopes::PopType),
    (Vic3(Province), Scopes::Province),
    (Vic3(Religion), Scopes::Religion),
    (Vic3(ShippingLane), Scopes::ShippingLane),
    (Vic3(State), Scopes::State),
    (Vic3(StateRegion), Scopes::StateRegion),
    (Vic3(StateTrait), Scopes::StateTrait),
    (Vic3(StrategicRegion), Scopes::StrategicRegion),
    (Vic3(Technology), Scopes::Technology),
    // (Vic3(TechnologyStatus),Scopes::TechnologyStatus),
    (Vic3(Theater), Scopes::Theater),
    (Vic3(TradeRoute), Scopes::TradeRoute),
    (Vic3(War), Scopes::War),
];

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
