use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::{scope_from_snake_case, Scopes};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;

#[derive(Clone, Debug)]
pub struct ScriptedGui {}

impl ScriptedGui {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedGui, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedGui {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new_root(Scopes::None, key.clone());
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = scope_from_snake_case(token.as_str()) {
                sc = ScopeContext::new_root(scope, token.clone());
            } else {
                warn(token, ErrorKey::Scopes, "unknown scope type");
            }
        }

        vd.field_list("saved_scopes");

        vd.field_validated_block("is_shown", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("is_valid", |b, data| {
            validate_normal_trigger(b, data, &mut sc, Tooltipped::No);
        });
        vd.field_validated_block("effect", |b, data| {
            // TODO: whether this is tooltipped depends on whether the gui calls for it
            validate_normal_effect(b, data, &mut sc, Tooltipped::No);
        });
    }
}
