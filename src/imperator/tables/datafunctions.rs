use crate::datatype::{Arg, Args, Datatype, ImperatorDatatype};
use crate::item::Item;
use crate::scopes::Scopes;

use Arg::*;
use Datatype::*;
use ImperatorDatatype::*;

// The include/ files are converted from the game's data_type_* output files.

pub const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
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

pub const GLOBAL_PROMOTES: &[(&str, Args, Datatype)] = include!("include/data_global_promotes.rs");

pub const GLOBAL_FUNCTIONS: &[(&str, Args, Datatype)] =
    include!("include/data_global_functions.rs");

pub const PROMOTES: &[(&str, Datatype, Args, Datatype)] = include!("include/data_promotes.rs");

pub const FUNCTIONS: &[(&str, Datatype, Args, Datatype)] = include!("include/data_functions.rs");
