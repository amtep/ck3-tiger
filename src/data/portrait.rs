use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::data::genes::{AccessoryGene, Gene};
use crate::db::{Db, DbKind};
use crate::everything::Everything;
#[cfg(feature = "ck3")]
use crate::game::Game;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::pdxfile::PdxEncoding;
use crate::report::{err, warn, Confidence, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::{validate_modifiers_with_base, validate_numeric_range};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PortraitModifierGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::PortraitModifierGroup, PortraitModifierGroup::add)
}

impl PortraitModifierGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitModifierGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitModifierGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // TODO: could the root be Scopes::None here?
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("age", Scopes::Value, key);
        sc.define_name("culture", Scopes::Culture, key);
        sc.define_name("current_weight", Scopes::Value, key);
        #[cfg(feature = "ck3")]
        if Game::is_ck3() {
            sc.define_name("highest_held_title_tier", Scopes::Value, key);
            sc.define_name("faith", Scopes::Faith, key);
            sc.define_name("government", Scopes::GovernmentType, key);
            sc.define_name("prowess", Scopes::Value, key);
            sc.define_name("ruler_designer", Scopes::Bool, key);
        }
        sc.define_name("female", Scopes::Bool, key);
        sc.define_name("weight_for_portrait", Scopes::Value, key);
        sc.define_name("year_of_birth", Scopes::Value, key);

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
                warn(ErrorKey::MissingItem).msg(msg).loc(token).push();
            }
        }
        vd.multi_field_validated_block("add_accessory_modifiers", |block, data| {
            validate_add_accessory_modifiers(block, data, caller, &mut sc);
        });
        vd.unknown_block_fields(|key, block| {
            validate_portrait_modifier(key, block, data, caller, &mut sc);
        });
    }

    fn has_property(&self, _key: &Token, block: &Block, property: &str, data: &Everything) -> bool {
        if property == "fallback" || property == "add_accessory_modifiers" {
            false
        } else if block.get_field_block(property).is_some() {
            true
        } else {
            for block in block.get_field_blocks("add_accessory_modifiers") {
                if let Some(gene) = block.get_field_value("gene") {
                    if let Some(template) = block.get_field_value("template") {
                        if let Some((key, block)) =
                            data.get_key_block(Item::GeneCategory, gene.as_str())
                        {
                            if AccessoryGene::has_template_setting(
                                key,
                                block,
                                data,
                                template.as_str(),
                                property,
                            ) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
    }
}

fn validate_portrait_modifier(
    key: &Token,
    block: &Block,
    data: &Everything,
    mut caller: &str,
    sc: &mut ScopeContext,
) {
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
    vd.field_list("outfit_tags"); // TODO
    vd.field_bool("require_outfit_tags");
    vd.field_bool("ignore_outfit_tags");

    vd.field_validated_block("is_valid_custom", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });

    #[cfg(feature = "imperator")]
    let modes = &["add", "replace", "modify", "replace_template"];
    #[cfg(not(feature = "imperator"))]
    let modes = &["add", "replace", "modify", "modify_multiply"];

    vd.multi_field_validated_block("dna_modifiers", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.multi_field_validated_block("morph", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", modes);
            vd.field_item("gene", Item::GeneCategory);
            if let Some(category) = block.get_field_value("gene") {
                if let Some(template) = vd.field_value("template") {
                    Gene::verify_has_template(category.as_str(), template, data);
                }
            }
            vd.field_script_value("value", sc);
            vd.field_validated_block("range", |block, data| {
                validate_numeric_range(block, data, 0.0, 1.0, Severity::Warning, Confidence::Weak);
            });
        });
        vd.multi_field_validated_block("color", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", modes);
            vd.field_item("gene", Item::GeneCategory);
            vd.field_numeric("x");
            vd.field_numeric("y");
        });
        vd.multi_field_validated_block("accessory", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("mode", modes);
            vd.field_item("gene", Item::GeneCategory);
            if let Some(category) = block.get_field_value("gene") {
                if let Some(template) = vd.field_value("template") {
                    Gene::verify_has_template(category.as_str(), template, data);
                }
            }
            vd.field_script_value("value", sc);
            vd.field_validated_block("range", |block, data| {
                validate_numeric_range(block, data, 0.0, 1.0, Severity::Warning, Confidence::Weak);
            });
            vd.field_item("accessory", Item::Accessory);
            vd.field_choice("type", &["male", "female", "boy", "girl"]);
        });
    });
    vd.multi_field_validated_block_sc("weight", sc, validate_modifiers_with_base);
}

fn validate_add_accessory_modifiers(
    block: &Block,
    data: &Everything,
    caller: &str,
    sc: &mut ScopeContext,
) {
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
    vd.field_validated_block("is_valid_custom", |block, data| {
        validate_trigger(block, data, sc, Tooltipped::No);
    });
    vd.multi_field_validated_block_sc("weight", sc, validate_modifiers_with_base);
}

#[derive(Clone, Debug)]
pub struct PortraitAnimation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::PortraitAnimation, PortraitAnimation::add)
}

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
                            warn(ErrorKey::Validation).msg("unknown body type").loc(token).push();
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

    vd.multi_field_validated_block("portrait_modifier", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
            validate_trigger(block, data, sc, Tooltipped::No);
        });
        validate_portrait_modifiers(block, data, vd);
    });

    vd.unknown_block_fields(|_, block| {
        let mut vd = Validator::new(block, data);
        vd.field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("head"); // TODO
            vd.field_value("torso"); // TODO
        });
        vd.field_validated_key_block("weight", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("age", Scopes::Value, key);
            sc.define_name("culture", Scopes::Culture, key);
            sc.define_name("current_weight", Scopes::Value, key);
            sc.define_name("ai_boldness", Scopes::Value, key);
            sc.define_name("ai_compassion", Scopes::Value, key);
            sc.define_name("ai_greed", Scopes::Value, key);
            sc.define_name("ai_honor", Scopes::Value, key);
            sc.define_name("ai_rationality", Scopes::Value, key);
            sc.define_name("ai_vengefulness", Scopes::Value, key);
            sc.define_name("ai_zeal", Scopes::Value, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_block("portrait_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_portrait_modifiers(block, data, vd);
        });
        vd.field_item("portrait_modifier_pack", Item::PortraitModifierPack);
    });
}

#[derive(Clone, Debug)]
pub struct PortraitModifierPack {}

inventory::submit! {
    ItemLoader::Full(GameFlags::all(), Item::PortraitModifierPack, PdxEncoding::Utf8Bom, ".modifierpack", false, PortraitModifierPack::add)
}

impl PortraitModifierPack {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PortraitModifierPack, key, block, Box::new(Self {}));
    }
}

impl DbKind for PortraitModifierPack {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.multi_field_validated_block("portrait_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block_rooted("trigger", Scopes::Character, |block, data, sc| {
                validate_trigger(block, data, sc, Tooltipped::No);
            });
            validate_portrait_modifiers(block, data, vd);
        });
    }
}

fn validate_portrait_modifiers(_block: &Block, data: &Everything, mut vd: Validator) {
    vd.unknown_value_fields(|key, value| {
        data.verify_exists(Item::PortraitModifierGroup, key);
        if !data.item_has_property(Item::PortraitModifierGroup, key.as_str(), value.as_str()) {
            let msg = format!("portrait modifier {value} not found in group {key}");
            err(ErrorKey::MissingItem).msg(msg).loc(value).push();
        }
    });
}

#[derive(Clone, Debug)]
pub struct PortraitCamera {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::all(), Item::PortraitCamera, PortraitCamera::add)
}

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
            vd.field_list_numeric_exactly("position", 3);
            vd.field_value("position_node"); // TODO

            vd.field_list_numeric_exactly("look_at", 3);
            vd.field_value("look_at_node"); // TODO

            vd.field_numeric("fov");
            vd.field_list_numeric_exactly("camera_near_far", 2);
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
