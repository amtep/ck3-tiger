use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::vic3::effect_validation::{EvB, EvBv, EvV};

use Effect::*;

#[derive(Copy, Clone, Debug)]
pub enum Effect {
    /// no special value, just effect = yes
    Yes,
    /// yes and no are both meaningful
    Boolean,
    Integer,
    ScriptValue,
    /// warn if literal negative
    NonNegativeValue,
    Scope(Scopes),
    ScopeOkThis(Scopes),
    Item(Item),
    ScopeOrItem(Scopes, Item),
    Target(&'static str, Scopes),
    TargetValue(&'static str, Scopes, &'static str),
    ItemTarget(&'static str, Item, &'static str, Scopes),
    ItemValue(&'static str, Item),
    Desc,
    /// days/weeks/months/years
    Timespan,
    AddModifier,
    Control,
    ControlOrLabel,
    /// so special that we just accept whatever argument
    Unchecked,
    Choice(&'static [&'static str]),
    Removed(&'static str, &'static str),
    VB(EvB),
    VBv(EvBv),
    VV(EvV),
}

pub fn scope_effect(name: &Token, data: &Everything) -> Option<(Scopes, Effect)> {
    let lwname = name.as_str().to_lowercase();

    for (from, s, effect) in SCOPE_EFFECT {
        if lwname == *s {
            return Some((Scopes::from_bits_truncate(*from), *effect));
        }
    }
    std::option::Option::None
}

/// LAST UPDATED VERSION 1.9.2
/// See `effects.log` from the game data dumps
const SCOPE_EFFECT: &[(u64, &str, Effect)] = &[
    // TODO
];
