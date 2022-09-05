use crate::block::{Block, BlockOrValue, DefinitionItem};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validate::validate_trigger;

pub fn validate_desc_map_block(
    block: &Block,
    data: &Everything,
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
                    validate_desc_map_block(b, data, f, false);
                } else if key.is("triggered_desc") {
                    validate_desc_map_block(b, data, f, true);
                } else if triggered && key.is("trigger") {
                    // TODO: pass in correct scopes
                    validate_trigger(b, data, Scopes::all(), &[]);
                } else {
                    warn(key, ErrorKey::Validation, "unexpected key in description");
                }
            }
        }
    }
}

pub fn validate_desc_map(bv: &BlockOrValue, data: &Everything, f: impl Fn(&Token, &Everything)) {
    match bv {
        BlockOrValue::Token(t) => {
            if !t.as_str().contains(' ') {
                f(t, data);
            }
        }
        BlockOrValue::Block(b) => {
            validate_desc_map_block(b, data, &f, false);
        }
    }
}

pub fn validate_desc(bv: &BlockOrValue, data: &Everything) {
    validate_desc_map(bv, data, |token, data| {
        data.localization.verify_exists(token);
    });
}
