use crate::block::{Block, DefinitionItem, BV};
use crate::context::ScopeContext;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

pub fn validate_desc_map_block(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    f: &impl Fn(&Token, &Everything),
    triggered: bool,
) {
    for def in block.iter_definitions_warn() {
        match def {
            DefinitionItem::Assignment(key, t) if key.is("desc") => {
                if !t.as_str().contains(' ') {
                    f(t, data);
                }
            }
            DefinitionItem::Assignment(key, _) | DefinitionItem::Keyword(key) => {
                warn(key, ErrorKey::Validation, "unexpected key in description");
            }
            DefinitionItem::Definition(key, b) => {
                if key.is("desc") || key.is("first_valid") || key.is("random_valid") {
                    validate_desc_map_block(b, data, sc, f, false);
                } else if key.is("triggered_desc") {
                    validate_desc_map_block(b, data, sc, f, true);
                } else if triggered && key.is("trigger") {
                    validate_normal_trigger(b, data, sc, Tooltipped::No);
                } else {
                    warn(key, ErrorKey::Validation, "unexpected key in description");
                }
            }
        }
    }
}

pub fn validate_desc_map(
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    f: impl Fn(&Token, &Everything),
) {
    match bv {
        BV::Value(t) => {
            if !t.as_str().contains(' ') {
                f(t, data);
            }
        }
        BV::Block(b) => {
            validate_desc_map_block(b, data, sc, &f, false);
        }
    }
}

pub fn validate_desc(bv: &BV, data: &Everything, sc: &mut ScopeContext) {
    validate_desc_map(bv, data, sc, |token, data| {
        data.localization.verify_exists(token);
    });
}
