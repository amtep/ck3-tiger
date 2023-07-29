#[cfg(feature = "ck3")]
pub use crate::ck3::modif::*;
#[cfg(feature = "ck3")]
use crate::ck3::tables::modifs::lookup_modif;
#[cfg(feature = "imperator")]
pub use crate::imperator::modif::*;
#[cfg(feature = "imperator")]
use crate::imperator::tables::modifs::lookup_modif;
#[cfg(feature = "vic3")]
pub use crate::vic3::modif::*;
#[cfg(feature = "vic3")]
use crate::vic3::tables::modifs::lookup_modif;

use crate::block::validator::Validator;
use crate::block::Block;
use crate::everything::Everything;
use crate::game::Game;
use crate::item::Item;
use crate::report::{err, error, ErrorKey, Severity};
use crate::scriptvalue::validate_non_dynamic_scriptvalue;
use crate::token::Token;

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
    vd.unknown_fields(|key, bv| {
        if let Some(mk) = lookup_modif(key, data, Some(Severity::Error)) {
            kinds.require(mk, key);
            validate_non_dynamic_scriptvalue(bv, data);
            #[cfg(feature = "ck3")]
            if Game::is_ck3() && !key.is("health") && !key.is("negate_health_penalty_add") {
                data.verify_exists(Item::ModifierFormat, key);
                // TODO: some modifiers have the loc as MOD_ and then all caps.
                // data.verify_exists(Item::Localization, key);
            }
            #[cfg(feature = "vic3")]
            if Game::is_vic3() {
                // The Item::ModifierType doesn't need to exist if the defaults are ok,
                // but the loca should exist.
                let loca = format!("modifier_{key}");
                data.verify_exists_implied(Item::Localization, &loca, key);
                let loca = format!("modifier_{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        } else {
            let msg = format!("unknown modifier `{key}`");
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    });
}

pub fn verify_modif_exists(key: &Token, data: &Everything, kinds: ModifKinds, sev: Severity) {
    if let Some(mk) = lookup_modif(key, data, Some(sev)) {
        kinds.require(mk, key);
    } else {
        let msg = format!("unknown modifier `{key}`");
        err(ErrorKey::UnknownField).msg(msg).loc(key).push();
    }
}
