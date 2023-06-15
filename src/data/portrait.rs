use crate::block::validator::Validator;
use crate::block::Block;
use crate::data::genes::Gene;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::warn;
use crate::everything::Everything;
use crate::item::Item;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_normal_trigger;
use crate::validate::validate_modifiers_with_base;

#[derive(Clone, Debug)]
pub struct PortraitModifierGroup {}

impl PortraitModifierGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitModifierGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitModifierGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("usage", &["customization", "game", "both", "none"]);
        vd.field_integer("interface_position");
        vd.field_integer("priority");
        vd.field_value("user_data");
        vd.field_choice("selection_behavior", &["weighted_random", "max"]);

        let mut caller = key.as_str();
        if let Some(token) = block.get_field_value("usage") {
            if token.is("game") || token.is("none") {
                caller = "";
            }
        }

        if !caller.is_empty() {
            let loca = format!("PORTRAIT_MODIFIER_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if let Some(token) = vd.field_value("fallback") {
            if !block.has_key(token.as_str()) {
                let msg = "portrait modifier not defined";
                warn(token, ErrorKey::MissingItem, msg);
            }
        }
        vd.field_validated_blocks("add_accessory_modifiers", |block, data| {
            validate_add_accessory_modifiers(block, data, caller);
        });
        for (key, bv) in vd.unknown_keys() {
            if let Some(block) = bv.expect_block() {
                validate_portrait_modifier(key, block, data, caller);
            }
        }
    }
}

fn validate_portrait_modifier(key: &Token, block: &Block, data: &Everything, mut caller: &str) {
    let mut vd = Validator::new(block, data);
    vd.field_choice("usage", &["customization", "game", "both"]);
    if let Some(token) = block.get_field_value("usage") {
        if token.is("game") {
            caller = "";
        }
    }
    if !caller.is_empty() {
        let loca = format!("PORTRAIT_MODIFIER_{caller}_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
    }
    vd.field_validated_block_rooted("is_valid_custom", Scopes::Character, |block, data, sc| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
    vd.field_list("outfit_tags"); // TODO
    vd.field_bool("require_outfit_tags");
    vd.field_bool("ignore_outfit_tags");

    vd.field_validated_blocks("dna_modifiers", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_blocks("morph", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", &["add", "replace", "modify", "modify_multiply"]);
            vd.field_item("gene", Item::GeneCategory);
            if let Some(category) = block.get_field_value("gene") {
                if let Some(template) = vd.field_value("template") {
                    Gene::verify_has_template(category.as_str(), template, data);
                }
            }
            vd.field_script_value_rooted("value", Scopes::Character);
            vd.field_validated_block("range", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(2);
            });
        });
        vd.field_validated_blocks("color", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", &["add", "replace", "modify", "modify_multiply"]);
            vd.field_item("gene", Item::GeneCategory);
            vd.field_numeric("x");
            vd.field_numeric("y");
        });
        vd.field_validated_blocks("accessory", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", &["add", "replace", "modify", "modify_multiply"]);
            vd.field_item("gene", Item::GeneCategory);
            if let Some(category) = block.get_field_value("gene") {
                if let Some(template) = vd.field_value("template") {
                    Gene::verify_has_template(category.as_str(), template, data);
                }
            }
            vd.field_numeric("value");
            vd.field_validated_block("range", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.req_tokens_numbers_exactly(2);
            });
            vd.field_item("accessory", Item::Accessory);
            vd.field_choice("type", &["male", "female", "boy", "girl"]);
        });
    });
    vd.field_validated_blocks_rooted("weight", Scopes::Character, validate_modifiers_with_base);
}

fn validate_add_accessory_modifiers(block: &Block, data: &Everything, caller: &str) {
    let mut vd = Validator::new(block, data);
    vd.field_item("gene", Item::GeneCategory);
    if let Some(category) = block.get_field_value("gene") {
        if let Some(template) = vd.field_value("template") {
            Gene::verify_has_template(category.as_str(), template, data);
            if !caller.is_empty() {
                data.database.validate_property_use(
                    Item::GeneCategory,
                    category,
                    data,
                    template,
                    caller,
                );
            }
        }
    }
    vd.field_validated_block_rooted("is_valid_custom", Scopes::Character, |block, data, sc| {
        validate_normal_trigger(block, data, sc, Tooltipped::No);
    });
    vd.field_validated_blocks_rooted("weight", Scopes::Character, validate_modifiers_with_base);
}
