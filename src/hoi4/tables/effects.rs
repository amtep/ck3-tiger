use std::sync::LazyLock;

use crate::effect::Effect;
use crate::effect_validation::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;

use Effect::*;

pub fn scope_effect(name: &Token, _data: &Everything) -> Option<(Scopes, Effect)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    SCOPE_EFFECT_MAP.get(&*name_lc).copied()
}

/// A hashed version of [`SCOPE_EFFECT`], for quick lookup by effect name.
static SCOPE_EFFECT_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Effect)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, effect) in SCOPE_EFFECT.iter().copied() {
            hash.insert(s, (from, effect));
        }
        hash
    });

// LAST UPDATED VIC3 VERSION 1.8.4
// See `effects.log` from the game data dumps
const SCOPE_EFFECT: &[(Scopes, &str, Effect)] = &[
    // TODO
];
