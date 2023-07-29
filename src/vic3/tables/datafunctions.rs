#![allow(non_camel_case_types)]

use std::str::FromStr;

use crate::datatype::{Arg, Args, Datatype, LookupResult, Vic3Datatype};
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use Vic3Datatype::*;

// The include/ files are converted from the game's data_type_* output files.

pub fn lookup_global_promote(lookup_name: &str) -> Option<(Args, Datatype)> {
    if let Ok(idx) = GLOBAL_PROMOTES.binary_search_by_key(&lookup_name, |(name, _, _)| name) {
        let (_name, args, rtype) = GLOBAL_PROMOTES[idx];
        return Some((args, rtype));
    }

    // Datatypes can be used directly as global promotes, taking their value from the gui context.
    if let Ok(dtype) = Datatype::from_str(lookup_name) {
        return Some((Args(&[]), dtype));
    }

    None
}

pub fn lookup_global_function(lookup_name: &str) -> Option<(Args, Datatype)> {
    if let Ok(idx) = GLOBAL_FUNCTIONS.binary_search_by_key(&lookup_name, |(name, _, _)| name) {
        let (_name, args, rtype) = GLOBAL_FUNCTIONS[idx];
        return Some((args, rtype));
    }
    None
}

fn lookup_promote_or_function(
    lookup_name: &str,
    ltype: Datatype,
    global: &[(&str, Datatype, Args, Datatype)],
) -> LookupResult {
    let start = global.partition_point(|(name, _, _, _)| name < &lookup_name);
    let mut found_any = false;
    let mut possible_args = None;
    let mut possible_rtype = None;
    for (name, intype, args, rtype) in global.iter().skip(start) {
        if lookup_name != *name {
            break;
        }
        found_any = true;
        if ltype == Datatype::Unknown {
            if possible_rtype.is_none() {
                possible_args = Some(*args);
                possible_rtype = Some(*rtype);
            } else if possible_rtype != Some(*rtype) {
                possible_rtype = Some(Datatype::Unknown);
            }
        } else if ltype == *intype {
            return LookupResult::Found(*args, *rtype);
        }
    }

    if found_any {
        if ltype == Datatype::Unknown {
            LookupResult::Found(possible_args.unwrap(), possible_rtype.unwrap())
        } else {
            LookupResult::WrongType
        }
    } else {
        LookupResult::NotFound
    }
}

pub fn lookup_promote(lookup_name: &str, ltype: Datatype) -> LookupResult {
    lookup_promote_or_function(lookup_name, ltype, PROMOTES)
}

pub fn lookup_function(lookup_name: &str, ltype: Datatype) -> LookupResult {
    lookup_promote_or_function(lookup_name, ltype, FUNCTIONS)
}

/// Find an alternative datafunction to suggest when `lookup_name` has not been found.
/// This is a fairly expensive lookup.
/// Currently it only looks for different-case variants.
/// TODO: make it consider misspellings as well
pub fn lookup_alternative(
    lookup_name: &str,
    first: std::primitive::bool,
    last: std::primitive::bool,
) -> Option<&'static str> {
    let lc = lookup_name.to_lowercase();
    if first {
        for (name, _, _) in GLOBAL_PROMOTES {
            if name.to_lowercase() == lc {
                return Some(name);
            }
        }
        if last {
            for (name, _, _) in GLOBAL_FUNCTIONS {
                if name.to_lowercase() == lc {
                    return Some(name);
                }
            }
        }
    } else {
        for (name, _, _, _) in PROMOTES {
            if name.to_lowercase() == lc {
                return Some(name);
            }
        }
        if last {
            for (name, _, _, _) in FUNCTIONS {
                if name.to_lowercase() == lc {
                    return Some(name);
                }
            }
        }
    }
    None
}

/// TODO: make a lookup for this table, rather than sequential scanning
// The ones not found among the datatypes, but which might be there under another name, are commented out.
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
    (Vic3(CommanderOrder), Scopes::CommanderOrder),
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
    (Vic3(JournalEntry), Scopes::Journalentry),
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

/// Return the scope type that best matches `dtype`, or `None` if there is no match.
/// Nearly every scope type has a matching datatype, but there are far more data types than scope types.
pub fn scope_from_datatype(dtype: Datatype) -> Option<Scopes> {
    for (dt, s) in DATATYPE_AND_SCOPE {
        if dtype == *dt {
            return Some(*s);
        }
    }
    None
}

/// Return the datatype that best matches `scopes`, or `Datatype::Unknown` if there is no match.
/// Nearly every scope type has a matching datatype, but there are far more datatypes than scope types.
/// Note that only `Scopes` values that are narrowed down to a single scope type can be matched.
pub fn datatype_from_scopes(scopes: Scopes) -> Datatype {
    for (dt, s) in DATATYPE_AND_SCOPE {
        if scopes == *s {
            return *dt;
        }
    }
    Datatype::Unknown
}

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
