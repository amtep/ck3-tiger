#![allow(non_camel_case_types)]

use std::str::FromStr;

use strum_macros::{Display, EnumString};

use crate::datatype::{Arg, Args, LookupResult};
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;

// Validate the "code" blocks in localization files and in the gui files.
// The include/ files are converted from the game's data_type_* output files.

include!("include/datatypes.rs");

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

pub fn scope_from_datatype(dtype: Datatype) -> Option<Scopes> {
    // The ones not found among the datatypes, but which might be there under another name, are commented out.
    match dtype {
        Datatype::Country => Some(Scopes::Country),
        Datatype::Battle => Some(Scopes::Battle),
        // Datatype::BattleSide => Some(Scopes::BattleSide),
        Datatype::Building => Some(Scopes::Building),
        Datatype::BuildingType => Some(Scopes::BuildingType),
        Datatype::CanalType => Some(Scopes::CanalType),
        Datatype::Character => Some(Scopes::Character),
        Datatype::CivilWar => Some(Scopes::CivilWar),
        Datatype::CombatUnit => Some(Scopes::CombatUnit),
        Datatype::CommanderOrder => Some(Scopes::CommanderOrder),
        Datatype::CommanderOrderType => Some(Scopes::CommanderOrderType),
        Datatype::CountryCreation => Some(Scopes::CountryCreation),
        Datatype::CountryDefinition => Some(Scopes::CountryDefinition),
        Datatype::CountryFormation => Some(Scopes::CountryFormation),
        Datatype::Culture => Some(Scopes::Culture),
        Datatype::Decree => Some(Scopes::Decree),
        Datatype::DiplomaticAction => Some(Scopes::DiplomaticAction),
        Datatype::DiplomaticPact => Some(Scopes::DiplomaticPact),
        Datatype::DiplomaticPlay => Some(Scopes::DiplomaticPlay),
        Datatype::DiplomaticRelations => Some(Scopes::DiplomaticRelations),
        Datatype::Front => Some(Scopes::Front),
        Datatype::Goods => Some(Scopes::Goods),
        Datatype::Hq => Some(Scopes::Hq),
        Datatype::Ideology => Some(Scopes::Ideology),
        Datatype::Institution => Some(Scopes::Institution),
        Datatype::InstitutionType => Some(Scopes::InstitutionType),
        // Datatype::InterestMarker => Some(Scopes::InterestMarker),
        Datatype::InterestGroup => Some(Scopes::InterestGroup),
        Datatype::InterestGroupTrait => Some(Scopes::InterestGroupTrait),
        // Datatype::InterestGroupType => Some(Scopes::InterestGroupType),
        Datatype::JournalEntry => Some(Scopes::Journalentry),
        Datatype::Law => Some(Scopes::Law),
        Datatype::LawType => Some(Scopes::LawType),
        Datatype::Market => Some(Scopes::Market),
        Datatype::MarketGoods => Some(Scopes::MarketGoods),
        Datatype::Objective => Some(Scopes::Objective),
        Datatype::Party => Some(Scopes::Party),
        Datatype::PoliticalMovement => Some(Scopes::PoliticalMovement),
        Datatype::Pop => Some(Scopes::Pop),
        Datatype::PopType => Some(Scopes::PopType),
        Datatype::Province => Some(Scopes::Province),
        Datatype::Religion => Some(Scopes::Religion),
        Datatype::ShippingLane => Some(Scopes::ShippingLane),
        Datatype::State => Some(Scopes::State),
        Datatype::StateRegion => Some(Scopes::StateRegion),
        Datatype::StateTrait => Some(Scopes::StateTrait),
        Datatype::StrategicRegion => Some(Scopes::StrategicRegion),
        Datatype::Technology => Some(Scopes::Technology),
        // Datatype::TechnologyStatus => Some(Scopes::TechnologyStatus),
        Datatype::Theater => Some(Scopes::Theater),
        Datatype::TradeRoute => Some(Scopes::TradeRoute),
        Datatype::War => Some(Scopes::War),
        _ => None,
    }
}

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
