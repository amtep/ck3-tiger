use std::sync::LazyLock;

use crate::helpers::TigerHashSet;

pub(crate) static BUILTIN_MACROS_HOI4: LazyLock<TigerHashSet<&'static str>> =
    LazyLock::new(|| BUILTIN_MACROS.iter().copied().collect());

// LAST UPDATED HOI4 VERSION 1.16.4
// The table entries were collected by analyzing tiger's own output.
const BUILTIN_MACROS: &[&str] = &[
    // TODO
];
