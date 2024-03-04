use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;

use crate::datatype::{Arg, Args, CaseInsensitiveStr, Ck3Datatype, Datatype};
use crate::helpers::BiFnvHashMap;
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Ck3Datatype::*;
use Datatype::*;

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

pub static PROMOTES_MAP: Lazy<FnvHashMap<&'static str, (Datatype, Args, Datatype)>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::default();
        for (name, from, args, to) in PROMOTES.iter().copied() {
            map.insert(name, (from, args, to));
        }
        map
    });

pub static FUNCTIONS_MAP: Lazy<FnvHashMap<&'static str, (Datatype, Args, Datatype)>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::default();
        for (name, from, args, to) in FUNCTIONS.iter().copied() {
            map.insert(name, (from, args, to));
        }
        map
    });

// The include/ files are converted from the game's data_type_* output files.

const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
    (Ck3(Character), Scopes::Character),
    (Ck3(Title), Scopes::LandedTitle),
    (Ck3(Activity), Scopes::Activity),
    (Ck3(Secret), Scopes::Secret),
    (Ck3(Province), Scopes::Province),
    (Ck3(Scheme), Scopes::Scheme),
    (Ck3(Combat), Scopes::Combat),
    (Ck3(CombatSide), Scopes::CombatSide),
    (Ck3(Faith), Scopes::Faith),
    (Ck3(GreatHolyWar), Scopes::GreatHolyWar),
    (Ck3(Religion), Scopes::Religion),
    (Ck3(War), Scopes::War),
    (Ck3(Story), Scopes::StoryCycle),
    (Ck3(CasusBelliItem), Scopes::CasusBelli),
    (Ck3(Dynasty), Scopes::Dynasty),
    (Ck3(DynastyHouse), Scopes::DynastyHouse),
    (Ck3(Faction), Scopes::Faction),
    (Ck3(Culture), Scopes::Culture),
    (Ck3(Army), Scopes::Army),
    (Ck3(HolyOrder), Scopes::HolyOrder),
    (Ck3(ActiveCouncilTask), Scopes::CouncilTask),
    (Ck3(MercenaryCompany), Scopes::MercenaryCompany),
    (Ck3(Artifact), Scopes::Artifact),
    (Ck3(Inspiration), Scopes::Inspiration),
    (Ck3(Struggle), Scopes::Struggle),
    (Ck3(CharacterMemory), Scopes::CharacterMemory),
    (Ck3(TravelPlan), Scopes::TravelPlan),
    (Ck3(Accolade), Scopes::Accolade),
    (Ck3(AccoladeType), Scopes::AccoladeType),
    (Ck3(Decision), Scopes::Decision),
    (Ck3(FaithDoctrine), Scopes::Doctrine),
    (Ck3(ActivityType), Scopes::ActivityType),
    (Ck3(CultureTradition), Scopes::CultureTradition),
    (Ck3(CulturePillar), Scopes::CulturePillar),
    (Ck3(GovernmentType), Scopes::GovernmentType),
    (Ck3(HoldingType), Scopes::HoldingType),
    (Ck3(Trait), Scopes::Trait),
    (Ck3(TaxSlot), Scopes::TaxSlot),
    (Ck3(VassalContract), Scopes::VassalContract),
    (Ck3(ObligationLevel), Scopes::VassalObligationLevel),
];

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
