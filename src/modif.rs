//! Validator for `modifs` which is our name for the basic things that modifiers modify.
//!
//! The main entry points are the [`validate_modifs`] function and the [`ModifKinds`] type.

use std::fmt::{Display, Formatter};

use bitflags::bitflags;

use crate::block::Block;
use crate::everything::Everything;
use crate::game::Game;
#[cfg(any(feature = "ck3", feature = "vic3"))]
use crate::item::Item;
use crate::report::{err, ErrorKey, Severity};
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::validator::Validator;

bitflags! {
    /// All the things a modif can apply to.
    /// Many modifs are for multiple things, so this is a bitflags type.
    ///
    /// This type is used to warn when a modif is used inappropriately.
    /// The logic for it is only really applicable to CK3, because in Vic3 all modifs are accepted
    /// in most places; for example you can add Building and Unit modifiers to a State.
    /// For Imperator it is not yet known how important this is.
    // LAST UPDATED CK3 1.11.3
    // LAST UPDATED VIC3 1.6.0
    // LAST UPDATED IMPERATOR 2.0.4
    // Taken from the game's `modifers.log`
    // Remember to update the display_fmt functions when ModifKinds changes.
    #[derive(Debug, Copy, Clone)]
    #[rustfmt::skip] // table looks better with cfg on same line
    pub struct ModifKinds: u16 {
        // ModifKinds used by more than one game
        const Character = 0x0001;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const Country = 0x0002;
        #[cfg(any(feature = "vic3", feature = "imperator"))]
        const State = 0x0004;
        #[cfg(any(feature = "ck3", feature = "imperator"))]
        const Province = 0x0008;

        #[cfg(feature = "ck3")] const County = 0x0010;
        #[cfg(feature = "ck3")] const Terrain = 0x0020;
        #[cfg(feature = "ck3")] const Culture = 0x0040;
        #[cfg(feature = "ck3")] const Scheme = 0x0080;
        #[cfg(feature = "ck3")] const TravelPlan = 0x0100;

        #[cfg(feature = "vic3")] const Battle = 0x0010;
        #[cfg(feature = "vic3")] const Building = 0x0020;
        #[cfg(feature = "vic3")] const InterestGroup = 0x0040;
        #[cfg(feature = "vic3")] const Market = 0x0080;
        #[cfg(feature = "vic3")] const PoliticalMovement = 0x0100;
        #[cfg(feature = "vic3")] const Tariff = 0x0200;
        #[cfg(feature = "vic3")] const Tax = 0x0400;
        #[cfg(feature = "vic3")] const Unit = 0x0800;
        #[cfg(feature = "vic3")] const Goods = 0x1000;
        #[cfg(feature = "vic3")] const MilitaryFormation = 0x2000;
    }
}

impl Display for ModifKinds {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => crate::ck3::modif::display_fmt(*self, f),
            #[cfg(feature = "vic3")]
            Game::Vic3 => crate::vic3::modif::display_fmt(*self, f),
            #[cfg(feature = "imperator")]
            Game::Imperator => crate::imperator::modif::display_fmt(*self, f),
        }
    }
}

impl ModifKinds {
    pub fn require(self, other: Self, token: &Token) {
        if (self & other).is_empty() {
            let msg = format!("`{token}` is a modifier for {other} but expected {self}");
            err(ErrorKey::Modifiers).msg(msg).loc(token).push();
        }
    }
}

pub fn validate_modifs<'a>(
    _block: &Block,
    data: &'a Everything,
    kinds: ModifKinds,
    mut vd: Validator<'a>,
) {
    let lookup_modif = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::modifs::lookup_modif,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::modifs::lookup_modif,
        #[cfg(feature = "imperator")]
        Game::Imperator => crate::imperator::tables::modifs::lookup_modif,
    };

    vd.unknown_fields(|key, bv| {
        if let Some(mk) = lookup_modif(key, data, Some(Severity::Error)) {
            kinds.require(mk, key);
            validate_non_dynamic_script_value(bv, data);
            #[cfg(feature = "ck3")]
            if Game::is_ck3() && !key.is("health") && !key.is("negate_health_penalty_add") {
                data.verify_exists(Item::ModifierFormat, key);
            }
            #[cfg(feature = "vic3")]
            if Game::is_vic3() {
                // The Item::ModifierType doesn't need to exist if the defaults are ok,
                // but the loca should exist.
                // TODO: should the key be lowercased?
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

#[cfg(not(feature = "imperator"))]
pub fn verify_modif_exists(key: &Token, data: &Everything, kinds: ModifKinds, sev: Severity) {
    let lookup_modif = match Game::game() {
        #[cfg(feature = "ck3")]
        Game::Ck3 => crate::ck3::tables::modifs::lookup_modif,
        #[cfg(feature = "vic3")]
        Game::Vic3 => crate::vic3::tables::modifs::lookup_modif,
    };

    if let Some(mk) = lookup_modif(key, data, Some(sev)) {
        kinds.require(mk, key);
    } else {
        let msg = format!("unknown modifier `{key}`");
        err(ErrorKey::UnknownField).msg(msg).loc(key).push();
    }
}
