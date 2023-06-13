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
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct CustomLocalization {}

impl CustomLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CustomLocalization, key, block, Box::new(Self {}));
    }
}

impl DbKind for CustomLocalization {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        if block.has_key("parent") {
            vd.field_item("parent", Item::CustomLocalization);
            vd.req_field("suffix");
            if let Some(suffix) = vd.field_value("suffix") {
                if let Some(parent) = block.get_field_value("parent") {
                    data.validate_variant(Item::CustomLocalization, parent, suffix);
                }
            }
            return;
        }

        let mut sc;
        vd.req_field("type");
        if let Some(token) = vd.field_value("type") {
            if token.is("all") {
                sc = ScopeContext::new_root(Scopes::all(), token.clone());
            } else if let Some(scopes) = scope_from_snake_case(token.as_str()) {
                sc = ScopeContext::new_root(scopes, token.clone());
            } else {
                warn(token, ErrorKey::Scopes, "unknown scope type");
                sc = ScopeContext::new_root(Scopes::all(), token.clone());
            }
        } else {
            sc = ScopeContext::new_root(Scopes::all(), key.clone());
        }

        vd.field_bool("random_valid");
        vd.field_bool("log_loc_errors");

        vd.field_validated_blocks("text", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("setup_scope", |block, data| {
                validate_normal_effect(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_validated_block("trigger", |block, data| {
                validate_normal_trigger(block, data, &mut sc, Tooltipped::No);
            });
            vd.field_validated_block_sc("weight_multiplier", &mut sc, validate_modifiers_with_base);

            vd.req_field("localization_key");
            vd.field_item("localization_key", Item::Localization);
            vd.field_bool("fallback");
        });
    }
    fn validate_variant(&self, _key: &Token, block: &Block, data: &Everything, suffix: &Token) {
        for block in block.get_field_blocks("text") {
            if let Some(key) = block.get_field_value("localization_key") {
                let loca = format!("{key}{suffix}");
                data.verify_exists_implied(Item::Localization, &loca, suffix);
            }
        }
    }
}
