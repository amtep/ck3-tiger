#[cfg(feature = "ck3")]
pub use crate::ck3::modif::*;
#[cfg(feature = "vic3")]
pub use crate::vic3::modif::*;

use crate::block::validator::Validator;
use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::ck3::tables::modifs::lookup_modif;
use crate::everything::Everything;
#[cfg(feature = "ck3")]
use crate::item::Item;
use crate::report::{err, error, ErrorKey};
use crate::scriptvalue::validate_non_dynamic_scriptvalue;
use crate::token::Token;
#[cfg(feature = "vic3")]
use crate::vic3::tables::modifs::lookup_modif;

impl ModifKinds {
    pub fn require(self, other: Self, token: &Token) {
        if !self.intersects(other) {
            let msg = format!("`{token}` is a modifier for {other} but expected {self}");
            error(token, ErrorKey::Modifiers, &msg);
        }
    }
}

pub fn validate_modifs<'a>(
    _block: &Block,
    data: &'a Everything,
    kinds: ModifKinds,
    mut vd: Validator<'a>,
) {
    for (key, bv) in vd.unknown_fields() {
        if let Some(mk) = lookup_modif(key, data, true) {
            kinds.require(mk, key);
            validate_non_dynamic_scriptvalue(bv, data);
            #[cfg(feature = "ck3")]
            if !key.is("health") && !key.is("negate_health_penalty_add") {
                data.verify_exists(Item::ModifierFormat, key);
            }
        } else {
            let msg = format!("unknown modifier `{key}`");
            err(ErrorKey::UnknownField).strong().msg(msg).loc(key).push();
        }
    }
}

pub fn verify_modif_exists(key: &Token, data: &Everything, kinds: ModifKinds) {
    if let Some(mk) = lookup_modif(key, data, true) {
        kinds.require(mk, key);
    } else {
        let msg = format!("unknown modifier `{key}`");
        err(ErrorKey::UnknownField).strong().msg(msg).loc(key).push();
    }
}
