use RawTrigger::*;

use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;

/// A version of Trigger that uses u64 to represent Scopes values, because
/// constructing bitfield types in const values is not allowed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum RawTrigger {
    /// trigger = no or trigger = yes
    Boolean,
    /// can be a script value
    CompareValue,
    /// can be a script value; warn if =
    CompareValueWarnEq,
    /// can be a script value; no < or >
    SetValue,
    /// value must be a valid date
    CompareDate,
    /// trigger is compared to a scope object
    Scope(u64),
    /// trigger is compared to a scope object which may be `this`
    ScopeOkThis(u64),
    /// value is chosen from an item type
    Item(Item),
    ScopeOrItem(u64, Item),
    /// value is chosen from a list given here
    Choice(&'static [&'static str]),
    /// For Block, if a field name in the array starts with ? it means that field is optional
    /// trigger takes a block with these fields
    Block(&'static [(&'static str, RawTrigger)]),
    /// trigger takes a block with these fields
    ScopeOrBlock(u64, &'static [(&'static str, RawTrigger)]),
    /// trigger takes a block with these fields
    ItemOrBlock(Item, &'static [(&'static str, RawTrigger)]),
    /// can be part of a scope chain but also a standalone trigger
    CompareValueOrBlock(&'static [(&'static str, RawTrigger)]),
    /// trigger takes a block of values of this scope type
    ScopeList(u64),
    /// trigger takes a block comparing two scope objects
    ScopeCompare(u64),
    /// this is for inside a Block, where a key is compared to a scope object
    CompareToScope(u64),

    /// this key opens another trigger block
    Control,
    /// this has specific code for validation
    Special,

    UncheckedValue,
}

/// A version of Trigger that has real Scopes values instead of u64 bitfields
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Trigger {
    Boolean,
    CompareValue,
    CompareValueWarnEq,
    SetValue,
    CompareDate,
    Scope(Scopes),
    ScopeOkThis(Scopes),
    Item(Item),
    ScopeOrItem(Scopes, Item),
    Choice(&'static [&'static str]),
    Block(Vec<(&'static str, Trigger)>),
    ScopeOrBlock(Scopes, Vec<(&'static str, Trigger)>),
    ItemOrBlock(Item, Vec<(&'static str, Trigger)>),
    CompareValueOrBlock(Vec<(&'static str, Trigger)>),
    ScopeList(Scopes),
    ScopeCompare(Scopes),
    CompareToScope(Scopes),

    Control,
    Special,

    UncheckedValue,
}

impl Trigger {
    fn from_raw(raw: &RawTrigger) -> Self {
        match raw {
            RawTrigger::Boolean => Trigger::Boolean,
            RawTrigger::CompareValue => Trigger::CompareValue,
            RawTrigger::CompareValueWarnEq => Trigger::CompareValueWarnEq,
            RawTrigger::SetValue => Trigger::SetValue,
            RawTrigger::CompareDate => Trigger::CompareDate,
            RawTrigger::Scope(s) => Trigger::Scope(Scopes::from_bits_truncate(*s)),
            RawTrigger::ScopeOkThis(s) => Trigger::ScopeOkThis(Scopes::from_bits_truncate(*s)),
            RawTrigger::Item(i) => Trigger::Item(*i),
            RawTrigger::ScopeOrItem(s, i) => {
                Trigger::ScopeOrItem(Scopes::from_bits_truncate(*s), *i)
            }
            RawTrigger::Choice(choices) => Trigger::Choice(choices),
            RawTrigger::Block(fields) => Trigger::Block(Trigger::from_raw_fields(fields)),
            RawTrigger::ScopeOrBlock(s, fields) => Trigger::ScopeOrBlock(
                Scopes::from_bits_truncate(*s),
                Trigger::from_raw_fields(fields),
            ),
            RawTrigger::ItemOrBlock(i, fields) => {
                Trigger::ItemOrBlock(*i, Trigger::from_raw_fields(fields))
            }
            RawTrigger::CompareValueOrBlock(fields) => {
                Trigger::CompareValueOrBlock(Trigger::from_raw_fields(fields))
            }
            RawTrigger::ScopeList(s) => Trigger::ScopeList(Scopes::from_bits_truncate(*s)),
            RawTrigger::ScopeCompare(s) => Trigger::ScopeCompare(Scopes::from_bits_truncate(*s)),
            RawTrigger::CompareToScope(s) => {
                Trigger::CompareToScope(Scopes::from_bits_truncate(*s))
            }
            RawTrigger::Control => Trigger::Control,
            RawTrigger::Special => Trigger::Special,
            RawTrigger::UncheckedValue => Trigger::UncheckedValue,
        }
    }

    fn from_raw_fields(
        fields: &'static [(&'static str, RawTrigger)],
    ) -> Vec<(&'static str, Trigger)> {
        fields
            .iter()
            .map(|(field, trigger)| (*field, Trigger::from_raw(trigger)))
            .collect()
    }
}

pub fn scope_trigger(name: &Token, data: &Everything) -> Option<(Scopes, Trigger)> {
    let name_lc = name.as_str().to_lowercase();

    for (from, s, trigger) in TRIGGER {
        if name_lc == *s {
            return Some((
                Scopes::from_bits_truncate(*from),
                Trigger::from_raw(trigger),
            ));
        }
    }
    std::option::Option::None
}

pub fn trigger_comparevalue(name: &Token, data: &Everything) -> Option<Scopes> {
    match scope_trigger(name, data) {
        Some((
            s,
            Trigger::CompareValue
            | Trigger::CompareValueWarnEq
            | Trigger::CompareDate
            | Trigger::SetValue
            | Trigger::CompareValueOrBlock(_),
        )) => Some(s),
        _ => std::option::Option::None,
    }
}

/// LAST UPDATED VERSION 1.9.2
/// See `triggers.log` from the game data dumps
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(u64, &str, RawTrigger)] = &[];
