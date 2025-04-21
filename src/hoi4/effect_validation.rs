use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::effect::validate_effect_control;
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validator::{Validator, ValueValidator};

pub fn validate_add_ace(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_value("name");
    vd.req_field("surname");
    vd.field_value("surname");
    vd.req_field("callsign");
    vd.field_value("callsign");
    vd.field_item("type", Item::AceModifier);
}

pub fn validate_add_advisor_role(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    if !sc.scopes().contains(Scopes::Character) {
        vd.req_field("character");
    }
    // TODO: if scope is a country literal, check that this character belongs to it.
    vd.field_item("character", Item::Character);
    vd.field_bool("activate");

    vd.field_validated_block("advisor", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("slot", Item::AdvisorSlot);
        vd.field_numeric("cost");
        vd.field_bool("can_be_fired");
        vd.field_value("idea_token"); // TODO does this need to be registered or validated
        vd.field_list_items("traits", Item::CountryLeaderTrait);
        vd.field_validated_block("allowed", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });
}

pub fn validate_flag_name(name: &Token) {
    let v = name.split('@');
    #[allow(clippy::comparison_chain)]
    if v.len() > 2 {
        let msg = "too many tags in flag name";
        let info = "each flag may only have one @-tag";
        err(ErrorKey::Validation).msg(msg).info(info).loc(name).push();
    } else if v.len() == 2 {
        let sfx = &v[1];
        if !(sfx.starts_with("ROOT")
            || sfx.starts_with("PREV")
            || sfx.starts_with("FROM")
            || sfx.starts_with("THIS"))
        {
            let msg = "invalid tag in flag name";
            let info = "must be @ROOT, @PREV, @FROM, or @THIS";
            err(ErrorKey::Validation).msg(msg).info(info).loc(name).push();
        }
    }
}

pub fn validate_clr_flag(
    key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    if key.is("clr_unit_leader_flag") {
        let msg = "deprecated in favor of clr_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    validate_flag_name(vd.value());
    vd.accept();
}

pub fn validate_modify_flag(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    if key.is("modify_unit_leader_flag") {
        let msg = "deprecated in favor of modify_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    vd.req_field("flag");
    vd.field_value("flag").map(validate_flag_name);
    vd.field_integer("value");
}

pub fn validate_set_flag(
    key: &Token,
    bv: &BV,
    data: &Everything,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    if key.is("set_unit_leader_flag") {
        let msg = "deprecated in favor of set_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    match bv {
        BV::Value(name) => {
            validate_flag_name(name);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field("flag");
            vd.field_value("flag").map(validate_flag_name);
            vd.field_integer("days");
            vd.field_integer("value");
        }
    }
}

/// A specific validator for the `random_list` effect, which has a unique syntax.
/// This one is for the hoi4 version, which is different from the jomini games.
pub fn validate_random_list(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    vd.field_bool("log");
    // TODO: validate variable expression if not 'const' or 'random'
    vd.field_value("seed"); // var_name/const/random
    vd.unknown_block_fields(|key, block| {
        // TODO: validate variable expression in else branch
        if let Some(n) = key.get_number() {
            // TODO: verify these claims for hoi4
            if n < 0.0 {
                let msg = "negative weights make the whole `random_list` fail";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            } else if n > 0.0 && n < 1.0 {
                let msg = "fractional weights are treated as just 0 in `random_list`";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            } else if n.fract() != 0.0 {
                let msg = "fractions are discarded in `random_list` weights";
                warn(ErrorKey::Range).strong().msg(msg).loc(key).push();
            }
        }
        validate_effect_control(&caller, block, data, sc, tooltipped);
    });
}
