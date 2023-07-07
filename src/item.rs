use std::fmt::{Display, Formatter};

#[cfg(feature = "ck3")]
pub use crate::ck3::item::*;
#[cfg(feature = "vic3")]
pub use crate::vic3::item::*;

/// "items" are all the things that can be looked up in string-indexed databases.
/// There is some overlap with scopes, but the difference is that scopes are runtime values
/// while items are always strings.

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let s: &'static str = self.into();
        write!(f, "{}", s.replace('_', " "))
    }
}
