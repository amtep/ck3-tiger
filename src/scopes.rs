//! The core [`Scopes`] type which tracks our knowledge about the types of in-game values.

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::game::Game;
use crate::item::Item;
use crate::report::{err, ErrorKey};
use crate::token::Token;

/// vic3 needs more than 64 bits, but the others don't.
#[cfg(feature = "vic3")]
type ScopesBits = u128;
#[cfg(not(feature = "vic3"))]
type ScopesBits = u64;

bitflags! {
    /// This type represents our knowledge about the set of scope types that a script value can
    /// have. In most cases it's narrowed down to a single scope type, but not always.
    ///
    /// The available scope types depend on the game.
    /// They are listed in `event_scopes.log` from the game data dumps.
    // LAST UPDATED CK3 VERSION 1.12.1
    // LAST UPDATED VIC3 VERSION 1.5.3
    // LAST UPDATED IR VERSION 2.0.4
    //
    // Each scope type gets one bitflag. In order to keep the bit count down, scope types from
    // the different games have overlapping bitflags. Therefore, scope types from different games
    // should be kept carefully separated.
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    #[rustfmt::skip] // having the cfg and the flag on one line is much more readable
    pub struct Scopes: ScopesBits {
        // Generic scope types
        const None = 0x0000_0001;
        const Value = 0x0000_0002;
        const Bool = 0x0000_0004;
        const Flag = 0x0000_0008;

        // Scope types shared by multiple games

        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Color = 0x0000_0010;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Country = 0x0000_0020;
        const Character = 0x0000_0040;
        const Culture = 0x0000_0080;
        const Province = 0x0000_0100;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Pop = 0x0000_0200;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Party = 0x0000_0400;
        const Religion = 0x0000_0800;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const State = 0x0000_1000;
        const War = 0x0000_2000;

        // Scope types for CK3
        #[cfg(feature = "ck3")] const Accolade = 0x0001_0000;
        #[cfg(feature = "ck3")] const AccoladeType = 0x0002_0000;
        #[cfg(feature = "ck3")] const Activity = 0x0004_0000;
        #[cfg(feature = "ck3")] const ActivityType = 0x0008_0000;
        #[cfg(feature = "ck3")] const Army = 0x0010_0000;
        #[cfg(feature = "ck3")] const Artifact = 0x0020_0000;
        #[cfg(feature = "ck3")] const CasusBelli = 0x0040_0000;
        #[cfg(feature = "ck3")] const CharacterMemory = 0x0080_0000;
        #[cfg(feature = "ck3")] const Combat = 0x0100_0000;
        #[cfg(feature = "ck3")] const CombatSide = 0x0200_0000;
        #[cfg(feature = "ck3")] const CouncilTask = 0x0400_0000;
        #[cfg(feature = "ck3")] const CulturePillar = 0x0800_0000;
        #[cfg(feature = "ck3")] const CultureTradition = 0x1000_0000;
        #[cfg(feature = "ck3")] const Decision = 0x2000_0000;
        #[cfg(feature = "ck3")] const Doctrine = 0x4000_0000;
        #[cfg(feature = "ck3")] const Dynasty = 0x8000_0000;
        #[cfg(feature = "ck3")] const DynastyHouse = 0x0000_0001_0000_0000;
        #[cfg(feature = "ck3")] const Faction = 0x0000_0002_0000_0000;
        #[cfg(feature = "ck3")] const Faith = 0x0000_0004_0000_0000;
        #[cfg(feature = "ck3")] const GovernmentType = 0x0000_0008_0000_0000;
        #[cfg(feature = "ck3")] const GreatHolyWar = 0x0000_0010_0000_0000;
        #[cfg(feature = "ck3")] const HolyOrder = 0x0000_0020_0000_0000;
        #[cfg(feature = "ck3")] const Inspiration = 0x0000_0040_0000_0000;
        #[cfg(feature = "ck3")] const LandedTitle = 0x0000_0080_0000_0000;
        #[cfg(feature = "ck3")] const MercenaryCompany = 0x0000_0100_0000_0000;
        #[cfg(feature = "ck3")] const Scheme = 0x0000_0200_0000_0000;
        #[cfg(feature = "ck3")] const Secret = 0x0000_0400_0000_0000;
        #[cfg(feature = "ck3")] const StoryCycle = 0x0000_0800_0000_0000;
        #[cfg(feature = "ck3")] const Struggle = 0x0000_1000_0000_0000;
        #[cfg(feature = "ck3")] const TitleAndVassalChange = 0x0000_2000_0000_0000;
        #[cfg(feature = "ck3")] const Trait = 0x0000_4000_0000_0000;
        #[cfg(feature = "ck3")] const TravelPlan = 0x0000_8000_0000_0000;
        #[cfg(feature = "ck3")] const VassalContract = 0x0001_0000_0000_0000;
        #[cfg(feature = "ck3")] const VassalObligationLevel = 0x0002_0000_0000_0000;
        // CK3 1.11
        #[cfg(feature = "ck3")] const HoldingType = 0x0004_0000_0000_0000;
        #[cfg(feature = "ck3")] const TaxSlot = 0x0008_0000_0000_0000;
        // CK3 1.12
        #[cfg(feature = "ck3")] const EpidemicType = 0x0010_0000_0000_0000;
        #[cfg(feature = "ck3")] const Epidemic = 0x0020_0000_0000_0000;
        #[cfg(feature = "ck3")] const LegendType = 0x0040_0000_0000_0000;
        #[cfg(feature = "ck3")] const Legend = 0x0080_0000_0000_0000;
        #[cfg(feature = "ck3")] const GeographicalRegion = 0x0100_0000_0000_0000;

        #[cfg(feature = "vic3")] const Battle = 0x0001_0000;
        #[cfg(feature = "vic3")] const BattleSide = 0x0002_0000;
        #[cfg(feature = "vic3")] const Building = 0x0004_0000;
        #[cfg(feature = "vic3")] const BuildingType = 0x0008_0000;
        #[cfg(feature = "vic3")] const CanalType = 0x0010_0000;
        #[cfg(feature = "vic3")] const CivilWar = 0x0020_0000;
        #[cfg(feature = "vic3")] const CombatUnit = 0x0040_0000;
        #[cfg(feature = "vic3")] const NewCombatUnit = 0x0080_0000;
        #[cfg(feature = "vic3")] const CommanderOrderType = 0x0100_0000;
        #[cfg(feature = "vic3")] const CountryCreation = 0x0200_0000;
        #[cfg(feature = "vic3")] const CountryDefinition = 0x0400_0000;
        #[cfg(feature = "vic3")] const CountryFormation = 0x0800_0000;
        #[cfg(feature = "vic3")] const Decree = 0x1000_0000;
        #[cfg(feature = "vic3")] const DiplomaticAction = 0x2000_0000;
        #[cfg(feature = "vic3")] const DiplomaticPact = 0x4000_0000;
        #[cfg(feature = "vic3")] const DiplomaticPlay = 0x8000_0000;
        #[cfg(feature = "vic3")] const DiplomaticRelations = 0x0000_0001_0000_0000;
        #[cfg(feature = "vic3")] const Front = 0x0000_0002_0000_0000;
        #[cfg(feature = "vic3")] const Goods = 0x0000_0004_0000_0000;
        #[cfg(feature = "vic3")] const Hq = 0x0000_0008_0000_0000;
        #[cfg(feature = "vic3")] const Ideology = 0x0000_0010_0000_0000;
        #[cfg(feature = "vic3")] const Institution = 0x0000_0020_0000_0000;
        #[cfg(feature = "vic3")] const InstitutionType = 0x0000_0040_0000_0000;
        #[cfg(feature = "vic3")] const InterestMarker = 0x0000_0080_0000_0000;
        #[cfg(feature = "vic3")] const InterestGroup = 0x0000_0100_0000_0000;
        #[cfg(feature = "vic3")] const InterestGroupTrait = 0x0000_0200_0000_0000;
        #[cfg(feature = "vic3")] const InterestGroupType = 0x0000_0400_0000_0000;
        #[cfg(feature = "vic3")] const JournalEntry = 0x0000_0800_0000_0000;
        #[cfg(feature = "vic3")] const Law = 0x0000_1000_0000_0000;
        #[cfg(feature = "vic3")] const LawType = 0x0000_2000_0000_0000;
        #[cfg(feature = "vic3")] const Market = 0x0000_4000_0000_0000;
        #[cfg(feature = "vic3")] const MarketGoods = 0x0000_8000_0000_0000;
        #[cfg(feature = "vic3")] const Objective = 0x0001_0000_0000_0000;
        #[cfg(feature = "vic3")] const PoliticalMovement = 0x0002_0000_0000_0000;
        #[cfg(feature = "vic3")] const PopType = 0x0004_0000_0000_0000;
        #[cfg(feature = "vic3")] const ShippingLane = 0x0008_0000_0000_0000;
        #[cfg(feature = "vic3")] const StateRegion = 0x0010_0000_0000_0000;
        #[cfg(feature = "vic3")] const StateTrait = 0x0020_0000_0000_0000;
        #[cfg(feature = "vic3")] const StrategicRegion = 0x0040_0000_0000_0000;
        #[cfg(feature = "vic3")] const Technology = 0x0080_0000_0000_0000;
        #[cfg(feature = "vic3")] const TechnologyStatus = 0x0100_0000_0000_0000;
        #[cfg(feature = "vic3")] const Theater = 0x0200_0000_0000_0000;
        #[cfg(feature = "vic3")] const TradeRoute = 0x0400_0000_0000_0000;
        #[cfg(feature = "vic3")] const CombatUnitType = 0x1000_0000_0000_0000;
        #[cfg(feature = "vic3")] const MilitaryFormation = 0x2000_0000_0000_0000;
        #[cfg(feature = "vic3")] const Sway = 0x4000_0000_0000_0000;
        #[cfg(feature = "vic3")] const StateGoods = 0x8000_0000_0000_0000;
        #[cfg(feature = "vic3")] const DiplomaticDemand = 0x0000_0000_0000_0001_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const Company = 0x0000_0000_0000_0002_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const CompanyType = 0x0000_0000_0000_0004_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const TravelNode = 0x0000_0000_0000_0008_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const TravelNodeDefinition = 0x0000_0000_0000_0010_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const TravelConnection = 0x0000_0000_0000_0020_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const TravelConnectionDefinition = 0x0000_0000_0000_0040_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const NavalInvasion = 0x0000_0000_0000_0080_0000_0000_0000_0000;
        #[cfg(feature = "vic3")] const MobilizationOption = 0x0000_0000_0000_0100_0000_0000_0000_0000;

        #[cfg(feature = "imperator")] const Area = 0x0001_0000;
        #[cfg(feature = "imperator")] const CountryCulture = 0x0002_0000;
        #[cfg(feature = "imperator")] const CultureGroup = 0x0004_0000;
        #[cfg(feature = "imperator")] const Deity = 0x0008_0000;
        #[cfg(feature = "imperator")] const Family = 0x0010_0000;
        #[cfg(feature = "imperator")] const Governorship = 0x0020_0000;
        #[cfg(feature = "imperator")] const GreatWork = 0x0040_0000;
        #[cfg(feature = "imperator")] const Job = 0x0080_0000;
        #[cfg(feature = "imperator")] const Legion = 0x0100_0000;
        #[cfg(feature = "imperator")] const LevyTemplate = 0x0200_0000;
        #[cfg(feature = "imperator")] const Region = 0x0400_0000;
        #[cfg(feature = "imperator")] const Siege = 0x0800_0000;
        #[cfg(feature = "imperator")] const SubUnit = 0x1000_0000;
        #[cfg(feature = "imperator")] const Treasure = 0x2000_0000;
        #[cfg(feature = "imperator")] const Unit = 0x4000_0000;
    }
}

// These have to be expressed a bit awkwardly because the binary operators are not `const`.
// TODO: Scopes::all() returns a too-large set if multiple features are enabled.
impl Scopes {
    pub const fn non_primitive() -> Scopes {
        Scopes::all()
            .difference(Scopes::None.union(Scopes::Value).union(Scopes::Bool).union(Scopes::Flag))
    }

    pub const fn primitive() -> Scopes {
        Scopes::Value.union(Scopes::Bool).union(Scopes::Flag)
    }

    pub const fn all_but_none() -> Scopes {
        Scopes::all().difference(Scopes::None)
    }

    /// Read a scope type in string form and return it as a [`Scopes`] value.
    pub fn from_snake_case(s: &str) -> Option<Scopes> {
        match s {
            "none" => return Some(Scopes::None),
            "value" => return Some(Scopes::Value),
            "bool" => return Some(Scopes::Bool),
            "flag" => return Some(Scopes::Flag),
            _ => (),
        };
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::scopes::scope_from_snake_case(s),
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::scopes::scope_from_snake_case(s),
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::scopes::scope_from_snake_case(s),
        }
    }

    /// Similar to `from_snake_case`, but allows multiple scopes separated by `|`
    /// Returns None if any of the conversions fail.
    pub fn from_snake_case_multi(s: &str) -> Option<Scopes> {
        let mut scopes = Scopes::empty();
        for part in s.split('|') {
            if let Some(scope) = Scopes::from_snake_case(part) {
                scopes |= scope;
            } else {
                return None;
            }
        }
        // If `scopes` is still empty then probably `s` was empty.
        // Remember that `Scopes::empty()` is different from a bitfield containing `Scopes::None`.
        if scopes == Scopes::empty() {
            return None;
        }
        Some(scopes)
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        if *self == Scopes::all() {
            write!(f, "any scope")
        } else if *self == Scopes::primitive() {
            write!(f, "any primitive scope")
        } else if *self == Scopes::non_primitive() {
            write!(f, "non-primitive scope")
        } else if *self == Scopes::all_but_none() {
            write!(f, "any except none scope")
        } else {
            match Game::game() {
                #[cfg(feature = "ck3")]
                Game::Ck3 => crate::ck3::scopes::display_fmt(*self, f),
                #[cfg(feature = "vic3")]
                Game::Vic3 => crate::vic3::scopes::display_fmt(*self, f),
                #[cfg(feature = "imperator")]
                Game::Imperator => crate::imperator::scopes::display_fmt(*self, f),
            }
        }
    }
}

/// A description of the constraints on a value with a prefix such as `var:` or `list_size:`
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ArgumentValue {
    /// The value must be an expression that resolves to a scope object of the given type.
    #[cfg(not(feature = "imperator"))]
    Scope(Scopes),
    /// The value must be the name of an item of the given item type.
    Item(Item),
    /// The value can be either a Scope or an Item
    #[cfg(feature = "ck3")]
    ScopeOrItem(Scopes, Item),
    /// The value must be the name of a modif
    #[cfg(feature = "vic3")]
    Modif,
    /// The value can be anything
    UncheckedValue,
}

/// Look up an "event link", which is a script token that looks up something related
/// to a scope value and returns another scope value.
///
/// `name` is the token. `inscopes` is the known scope context of this token.
/// `inscopes` is only used for some special-case event links whose output scope type
/// depends on their input scope type.
///
/// Returns a pair of `Scopes`. The first is the scope types this token can accept as input,
/// and the second is the scope types it may return.
#[allow(unused_variables)] // inscopes is only used for vic3
pub fn scope_to_scope(name: &Token, inscopes: Scopes) -> Option<(Scopes, Scopes)> {
    let scope_to_scope = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::scope_to_scope,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::scope_to_scope,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::scope_to_scope,
    };
    let scope_to_scope_removed = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::scope_to_scope_removed,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::scope_to_scope_removed,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::scope_to_scope_removed,
    };

    let name_lc = name.as_str().to_lowercase();
    if let scopes @ Some((from, _)) = scope_to_scope(&name_lc) {
        #[cfg(feature = "vic3")]
        if Game::is_vic3() && name_lc == "type" {
            // Special case for "type" because it goes from specific scope types to specific
            // other scope types.
            let mut outscopes = Scopes::empty();
            if inscopes.contains(Scopes::Building) {
                outscopes |= Scopes::BuildingType;
            }
            if inscopes.contains(Scopes::Institution) {
                outscopes |= Scopes::InstitutionType;
            }
            if inscopes.contains(Scopes::InterestGroup) {
                outscopes |= Scopes::InterestGroupType;
            }
            if inscopes.contains(Scopes::Law) {
                outscopes |= Scopes::LawType;
            }
            if inscopes.contains(Scopes::Company) {
                outscopes |= Scopes::CompanyType;
            }
            if !outscopes.is_empty() {
                return Some((from, outscopes));
            }
        }
        scopes
    } else if let Some((version, explanation)) = scope_to_scope_removed(&name_lc) {
        let msg = format!("`{name}` was removed in {version}");
        err(ErrorKey::Removed).strong().msg(msg).info(explanation).loc(name).push();
        return Some((Scopes::all(), Scopes::all_but_none()));
    } else {
        None
    }
}

/// Look up a prefixed token that is used to look up items in the game database.
///
/// For example, `character:alexander_the_great` to fetch that character as a scope value.
///
/// Some prefixes have an input scope, and they look up something related to the input scope value.
///
/// Returns a pair of `Scopes` and the type of argument it accepts.
/// The first `Scopes` is the scope types this token can accept as input, and the second one is
/// the scope types it may return. The first will be `Scopes::None` if it needs no input.
pub fn scope_prefix(prefix: &Token) -> Option<(Scopes, Scopes, ArgumentValue)> {
    let scope_prefix = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::scope_prefix,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::scope_prefix,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::scope_prefix,
    };
    let prefix_lc = prefix.as_str().to_lowercase();
    scope_prefix(&prefix_lc)
}

/// Look up a token that's an invalid target, and see if it might be missing a prefix.
/// Return the prefix if one was found.
///
/// `scopes` should be a singular `Scopes` flag.
///
/// Example: if the token is "irish" and `scopes` is `Scopes::Culture` then return
/// `Some("culture")` to indicate that the token should have been "culture:irish".
pub fn needs_prefix(arg: &str, data: &Everything, scopes: Scopes) -> Option<&'static str> {
    match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::needs_prefix(arg, data, scopes),
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::needs_prefix(arg, data, scopes),
    }
}

/// Look up an iterator, which is a script element that executes its block multiple times, once for
/// each applicable scope value. Iterators may be builtin (the usual case) or may be scripted lists.
///
/// `name` is the name of the iterator, without its `any_`, `every_`, `random_` or `ordered_` prefix.
/// `sc` is a [`ScopeContext`], only used for validating scripted lists.
///
/// Returns a pair of `Scopes`. The first is the scope types this token can accept as input,
/// and the second is the scope types it may return.
/// The first will be `Scopes::None` if it needs no input.
pub fn scope_iterator(
    name: &Token,
    data: &Everything,
    sc: &mut ScopeContext,
) -> Option<(Scopes, Scopes)> {
    let scope_iterator = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::scope_iterator,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::scope_iterator,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::scope_iterator,
    };
    let scope_iterator_removed = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::scopes::scope_iterator_removed,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::scopes::scope_iterator_removed,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::scopes::scope_iterator_removed,
    };

    let name_lc = name.as_str().to_lowercase();
    if let scopes @ Some(_) = scope_iterator(&name_lc) {
        return scopes;
    } else if let Some((version, explanation)) = scope_iterator_removed(&name_lc) {
        let msg = format!("`{name}` iterators were removed in {version}");
        err(ErrorKey::Removed).strong().msg(msg).info(explanation).loc(name).push();
        return Some((Scopes::all(), Scopes::all()));
    } else if data.scripted_lists.exists(name.as_str()) {
        data.scripted_lists.validate_call(name, data, sc);
        return data
            .scripted_lists
            .base(name)
            .and_then(|base| scope_iterator(&base.as_str().to_lowercase()));
    }
    None
}
