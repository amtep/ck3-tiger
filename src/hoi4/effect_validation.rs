use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, warn, ErrorKey, ErrorLoc};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target;
use crate::trigger::validate_trigger;
use crate::validate::{validate_color, validate_optional_duration};
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
        vd.field_item("idea_token", Item::Character); // TODO what is this
        vd.field_list_items("traits", Item::CharacterTrait);
        vd.field_validated_block("allowed", |block, data| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
    });
}
