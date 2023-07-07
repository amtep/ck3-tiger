use crate::block::validator::Validator;
use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_normal_effect;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{old_warn, ErrorKey};
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

        let mut sc;
        if let Some(token) = vd.field_value("type") {
            if token.is("all") {
                sc = ScopeContext::new(Scopes::all(), token);
            } else if let Some(scopes) = scope_from_snake_case(token.as_str()) {
                sc = ScopeContext::new(scopes, token);
            } else {
                old_warn(token, ErrorKey::Scopes, "unknown scope type");
                sc = ScopeContext::new(Scopes::all(), token);
            }
        } else {
            sc = ScopeContext::new(Scopes::all(), key);
        }
        // Scopes depend on the scopes available in the loca where Custom or Custom2 is called. TODO
        sc.set_strict_scopes(false);
        vd.field_bool("log_loc_errors");

        if block.has_key("parent") {
            vd.field_item("parent", Item::CustomLocalization);
            vd.req_field("suffix");
            vd.field_value("suffix");
            // Actual loca existence is checked in validate_custom_call
            return;
        }
        vd.req_field("type");

        vd.field_bool("random_valid");

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
            // Actual loca existence is checked in validate_custom_call
            vd.field_value("localization_key");
            vd.field_bool("fallback");
        });
    }
}

impl CustomLocalization {
    pub fn validate_custom_call(
        key: &Token,
        block: &Block,
        data: &Everything,
        caller: &Token,
        scopes: Scopes,
        lang: &'static str,
        suffix_str: &str,
        suffix_token: Option<&Token>,
    ) {
        if let Some(token) = block.get_field_value("type") {
            if let Some(this_scopes) = scope_from_snake_case(token.as_str()) {
                if !scopes.contains(this_scopes) {
                    let msg = format!(
                        "custom localization {key} is for {this_scopes} but context is {scopes}"
                    );
                    old_warn(caller, ErrorKey::Scopes, &msg);
                }
            }
        }

        if let Some(parent) = block.get_field_value("parent") {
            if let Some(suffix) = block.get_field_value("suffix") {
                if let Some((key, block)) = data
                    .database
                    .get_key_block(Item::CustomLocalization, parent.as_str())
                {
                    let suffix_str = format!("{suffix_str}{suffix}");
                    let suffix_token = if suffix_token.is_some() {
                        suffix_token
                    } else {
                        Some(suffix)
                    };
                    Self::validate_custom_call(
                        key,
                        block,
                        data,
                        caller,
                        scopes,
                        lang,
                        &suffix_str,
                        suffix_token,
                    );
                }
            }
        } else {
            for block in block.get_field_blocks("text") {
                if let Some(key) = block.get_field_value("localization_key") {
                    if let Some(token) = suffix_token {
                        let loca = format!("{key}{suffix_str}");
                        data.localization
                            .verify_exists_implied_lang(&loca, token, lang);
                    } else {
                        data.localization.verify_exists_lang(key, lang);
                    }
                }
            }
        }
    }
}
