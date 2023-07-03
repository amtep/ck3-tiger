#[cfg(feature = "ck3")]
pub use crate::ck3::scopes::*;
#[cfg(feature = "vic3")]
pub use crate::vic3::scopes::*;

impl Scopes {
    pub fn non_primitive() -> Scopes {
        Scopes::all() ^ (Scopes::None | Scopes::Value | Scopes::Bool | Scopes::Flag)
    }

    pub fn primitive() -> Scopes {
        Scopes::Value | Scopes::Bool | Scopes::Flag
    }

    pub fn all_but_none() -> Scopes {
        Scopes::all() ^ Scopes::None
    }
}
