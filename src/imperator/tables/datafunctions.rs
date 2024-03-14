use crate::Item;
use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;

use crate::datatype::{Arg, Args, CaseInsensitiveStr, Datatype, ImperatorDatatype};
use crate::helpers::BiFnvHashMap;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use ImperatorDatatype::*;

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

#[allow(clippy::type_complexity)]
pub static PROMOTES_MAP: Lazy<FnvHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in PROMOTES.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

#[allow(clippy::type_complexity)]
pub static FUNCTIONS_MAP: Lazy<FnvHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    Lazy::new(|| {
        let mut map = FnvHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in FUNCTIONS.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

// The include/ files are converted from the game's data_type_* output files.

const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
    (Imperator(Country), Scopes::Country),
    (Imperator(Character), Scopes::Character),
    (Imperator(Province), Scopes::Province),
    (Imperator(Siege), Scopes::Siege),
    (Imperator(Unit), Scopes::Unit),
    (Imperator(Pop), Scopes::Pop),
    (Imperator(Family), Scopes::Family),
    (Imperator(Party), Scopes::Party),
    (Imperator(Religion), Scopes::Religion),
    (Imperator(Culture), Scopes::Culture),
    (Imperator(CharacterJob), Scopes::Job),
    (Imperator(CultureGroup), Scopes::CultureGroup),
    (Imperator(CountryCulture), Scopes::CountryCulture),
    (Imperator(Area), Scopes::Area),
    (Imperator(State), Scopes::State),
    (Imperator(SubUnit), Scopes::SubUnit),
    (Imperator(Governorship), Scopes::Governorship),
    (Imperator(Region), Scopes::Region),
    (Imperator(Deity), Scopes::Deity),
    (Imperator(GreatWork), Scopes::GreatWork),
    (Imperator(Treasure), Scopes::Treasure),
    (Imperator(War), Scopes::War),
    (Imperator(Legion), Scopes::Legion),
    (Imperator(LevyTemplate), Scopes::LevyTemplate),
];

const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = include!("include/data_global_functions.rs");

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
