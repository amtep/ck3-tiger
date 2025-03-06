use std::sync::LazyLock;

use crate::datatype::{Arg, Args, CaseInsensitiveStr, Datatype, ImperatorDatatype};
use crate::helpers::{BiTigerHashMap, TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use ImperatorDatatype::*;

pub static LOWERCASE_DATATYPE_SET: LazyLock<TigerHashSet<CaseInsensitiveStr>> =
    LazyLock::new(|| {
        let mut set = TigerHashSet::default();

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

pub static DATATYPE_AND_SCOPE_MAP: LazyLock<BiTigerHashMap<Datatype, Scopes>> =
    LazyLock::new(|| {
        let mut map = BiTigerHashMap::default();
        for (datatype, scope) in DATATYPE_AND_SCOPE.iter().copied() {
            map.insert(datatype, scope);
        }
        map
    });

pub static GLOBAL_PROMOTES_MAP: LazyLock<TigerHashMap<&'static str, (Args, Datatype)>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::default();
        for (name, args, datatype) in GLOBAL_PROMOTES.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

pub static GLOBAL_FUNCTIONS_MAP: LazyLock<TigerHashMap<&'static str, (Args, Datatype)>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::default();
        for (name, args, datatype) in GLOBAL_FUNCTIONS.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

#[allow(clippy::type_complexity)]
pub static PROMOTES_MAP: LazyLock<TigerHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in PROMOTES.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

#[allow(clippy::type_complexity)]
pub static FUNCTIONS_MAP: LazyLock<TigerHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
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
