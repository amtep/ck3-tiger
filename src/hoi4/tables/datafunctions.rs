use std::sync::LazyLock;

use crate::datatype::{Arg, Args, CaseInsensitiveStr, Datatype, Hoi4Datatype};
use crate::helpers::{BiTigerHashMap, TigerHashMap, TigerHashSet};
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use Hoi4Datatype::*;

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

const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
    (Hoi4(Ace), Scopes::Ace),
    (Hoi4(Character), Scopes::Character),
    (Hoi4(Country), Scopes::Country),
    (Hoi4(IndustrialOrg), Scopes::IndustrialOrg),
    (Hoi4(Operation), Scopes::Operation),
    (Hoi4(PurchaseContract), Scopes::PurchaseContract),
    (Hoi4(SpecialProject), Scopes::SpecialProject),
    (Hoi4(State), Scopes::State),
    (Hoi4(UnitLeader), Scopes::Character),
];

// Hoi4 does not have these
const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = &[];

// Hoi4 does not have these
const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] = &[];

const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
