use std::sync::LazyLock;

use crate::helpers::TigerHashSet;
use crate::lowercase::Lowercase;

/// A hashed version of [`SOUNDS`], for quick case-insensitive lookup.
pub static SOUNDS_SET: LazyLock<TigerHashSet<Lowercase<'static>>> = LazyLock::new(|| {
    let mut set = TigerHashSet::default();
    for sound in SOUNDS.iter().copied() {
        set.insert(Lowercase::new(sound));
    }
    set
});

// LAST UPDATED VIC3 VERSION 1.7.1
// Taken from the object browser
const SOUNDS: &[&str] = &[
    // TODO
];
