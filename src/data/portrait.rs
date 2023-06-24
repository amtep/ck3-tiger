use crate::block::validator::Validator;
use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::data::genes::Gene;
use crate::db::{Db, DbKind};
use crate::errorkey::ErrorKey;
use crate::errors::{error, warn};
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
        for (key, block) in vd.unknown_block_fields() {
            validate_portrait_modifier(key, block, data, caller);
        }
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "fallback" || property == "add_accessory_modifiers" {
            false
        } else {
            block.get_field_block(property).is_some()
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

#[derive(Clone, Debug)]
pub struct PortraitAnimation {}

impl PortraitAnimation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitAnimation, key, block, Box::new(Self {}));
    }
}

const TYPES: &[&str] = &["male", "female", "boy", "girl"];
impl DbKind for PortraitAnimation {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);

        for field in TYPES {
            vd.req_field(field);
            vd.field_validated(field, |bv, data| {
                match bv {
                    BV::Value(token) => {
                        // TODO: check that the chain eventually resolves to a block
                        if !TYPES.contains(&token.as_str()) {
                            warn(token, ErrorKey::Validation, "unknown body type");
                        }
                    }
                    BV::Block(block) => {
                        validate_animation(block, data);
                    }
                }
            });
        }
    }
}

fn validate_animation(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_validated_block("default", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_value("head"); // TODO
        vd.field_value("torso"); // TODO
    });

    vd.field_validated_blocks("portrait_modifier", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
            validate_normal_trigger(block, data, sc, Tooltipped::No);
        });
        validate_portrait_modifiers(block, data, vd);
    });

    for (_key, block) in vd.unknown_block_fields() {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("head"); // TODO
            vd.field_value("torso"); // TODO
        });
        vd.field_validated_key_block("weight", |key, block, data| {
            let mut sc = ScopeContext::new_root(Scopes::Character, key.clone());
            sc.define_name("age", Scopes::Value, key.clone());
            sc.define_name("current_weight", Scopes::Value, key.clone());
            sc.define_name("ai_boldness", Scopes::Value, key.clone());
            sc.define_name("ai_compassion", Scopes::Value, key.clone());
            sc.define_name("ai_greed", Scopes::Value, key.clone());
            sc.define_name("ai_honor", Scopes::Value, key.clone());
            sc.define_name("ai_rationality", Scopes::Value, key.clone());
            sc.define_name("ai_vengefulness", Scopes::Value, key.clone());
            sc.define_name("ai_zeal", Scopes::Value, key.clone());
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_block("portrait_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_portrait_modifiers(block, data, vd);
        });
        vd.field_item("portrait_modifier_pack", Item::PortraitModifierPack);
    }
}

#[derive(Clone, Debug)]
pub struct PortraitModifierPack {}

impl PortraitModifierPack {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitModifierPack, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitModifierPack {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_blocks("portrait_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
                validate_normal_trigger(block, data, sc, Tooltipped::No);
            });
            validate_portrait_modifiers(block, data, vd);
        });
    }
}

fn validate_portrait_modifiers(_block: &Block, data: &Everything, mut vd: Validator) {
    for (key, value) in vd.unknown_value_fields() {
        data.verify_exists(Item::PortraitModifierGroup, key);
        if !data.item_has_property(Item::PortraitModifierGroup, key.as_str(), value.as_str()) {
            let msg = format!("portrait modifier {value} not found in group {key}");
            error(value, ErrorKey::MissingItem, &msg);
        }
    }
}

#[derive(Clone, Debug)]
pub struct PortraitCamera {}

impl PortraitCamera {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitCamera, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitCamera {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("camera", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_integers_exactly("position", 3);
            vd.field_value("position_node"); // TODO

            vd.field_list_integers_exactly("look_at", 3);
            vd.field_value("look_at_node"); // TODO

            vd.field_integer("fov");
            vd.field_list_integers_exactly("camera_near_far", 2);
        });

        if let Some(token) = vd.field_value("unknown") {
            if token.as_str().contains('/') {
                data.verify_exists(Item::File, token);
            } else {
                data.verify_exists(Item::PortraitCamera, token);
            }
        }
    }
}
