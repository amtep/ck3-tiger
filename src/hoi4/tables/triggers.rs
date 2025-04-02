use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::hoi4::tables::misc::*;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::trigger::Trigger;

use Trigger::*;

pub fn scope_trigger(name: &Token, _data: &Everything) -> Option<(Scopes, Trigger)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    TRIGGER_MAP.get(&*name_lc).copied()
}

static TRIGGER_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Trigger)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, trigger) in TRIGGER.iter().copied() {
        hash.insert(s, (from, trigger));
    }
    hash
});

/// LAST UPDATED VIC3 VERSION 1.8.1
/// See `triggers.log` from the game data dumps
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(Scopes, &str, Trigger)] = &[
    // TODO
];

#[inline]
pub fn scope_trigger_complex(name: &str) -> Option<(Scopes, ArgumentValue, Scopes)> {
    TRIGGER_COMPLEX_MAP.get(name).copied()
}

static TRIGGER_COMPLEX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, ArgumentValue, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, trigger, outscopes) in TRIGGER_COMPLEX.iter().copied() {
            hash.insert(s, (from, trigger, outscopes));
        }
        hash
    });

// Hoi4 doesn't use these.
const TRIGGER_COMPLEX: &[(Scopes, &str, ArgumentValue, Scopes)] = &[];
