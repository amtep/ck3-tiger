#[cfg(feature = "ck3")]
pub use crate::ck3::scopes::*;
#[cfg(feature = "imperator")]
pub use crate::imperator::scopes::*;
#[cfg(feature = "vic3")]
pub use crate::vic3::scopes::*;

impl Scopes {
    // These have to be expressed a bit awkwardly because the binary operators are not `const`.
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
}
