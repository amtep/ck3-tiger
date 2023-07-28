//! "items" are all the things that can be looked up in the game databases.
//! Anything that can be looked up in script with a literal string key, or that's loaded into
//! tiger's database and needs a unique key, is an `Item`.
//!
//! There is some overlap with scopes, for example "culture" is both an `Item` and a scope type,
//! but the difference is that scopes are runtime values while items are always strings.
//!
//! For example if a trigger takes a culture *scope*, you could supply either `culture:german` or
//! `scope:target_culture`, while if a trigger takes a culture *item*, you would have to supply just
//! `german` and don't have the option of supplying something determined at runtime.

use std::fmt::{Display, Formatter};

#[cfg(feature = "ck3")]
pub use crate::ck3::item::*;
#[cfg(feature = "vic3")]
pub use crate::vic3::item::*;

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let s: &'static str = self.into();
        write!(f, "{}", s.replace('_', " "))
    }
}
