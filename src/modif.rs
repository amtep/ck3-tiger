#![allow(non_upper_case_globals)]

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::block::validator::Validator;
use crate::block::Block;
#[cfg(feature = "ck3")]
use crate::ck3::tables::modifs::lookup_modif;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{error, old_warn, ErrorKey};
use crate::scriptvalue::validate_non_dynamic_scriptvalue;
use crate::token::Token;
#[cfg(feature = "vic3")]
use crate::vic3::tables::modifs::lookup_modif;

bitflags! {
    #[derive(Debug, Copy, Clone)]
    pub struct ModifKinds: u8 {
        const Character = 0x01;
        const Province = 0x02;
        const County = 0x04;
        const Terrain = 0x08;
        const Culture = 0x10;
        const Scheme = 0x20;
        const TravelPlan = 0x40;
    }
}

impl ModifKinds {
    pub fn require(self, other: Self, token: &Token) {
        if !self.intersects(other) {
            let msg = format!("`{token}` is a modifier for {other} but expected {self}");
            error(token, ErrorKey::Modifiers, &msg);
        }
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        let mut vec = Vec::new();
        if self.contains(ModifKinds::Character) {
            vec.push("character");
        }
        if self.contains(ModifKinds::Province) {
            vec.push("province");
        }
        if self.contains(ModifKinds::County) {
            vec.push("county");
        }
        if self.contains(ModifKinds::Terrain) {
            vec.push("terrain");
        }
        if self.contains(ModifKinds::Culture) {
            vec.push("culture");
        }
        if self.contains(ModifKinds::Scheme) {
            vec.push("scheme");
        }
        if self.contains(ModifKinds::TravelPlan) {
            vec.push("travel plan");
        }
        write!(f, "{}", vec.join(", "))
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
            old_warn(key, ErrorKey::UnknownField, &msg);
        }
    }
}
