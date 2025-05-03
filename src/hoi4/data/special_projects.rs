use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::validate::validate_equipment_bonus;
use crate::item::{Item, ItemLoader};
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_color, validate_modifiers_with_base};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SpecialProject {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::SpecialProject, SpecialProject::add)
}

impl SpecialProject {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SpecialProject, key, block, Box::new(Self {}));
    }
}

impl DbKind for SpecialProject {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        validate_narrative(key, data, &mut vd, true);

        if !vd.field_item("icon", Item::Sprite) {
            let sprite = format!("GFX_{key}");
            data.mark_used(Item::Sprite, &sprite);
            // There's a fallback icon GFX_PLACEHOLDER_sp_project_icon
        }

        vd.req_field("specialization");
        vd.field_item("specialization", Item::Specialization);

        vd.field_list_items("project_tags", Item::SpecialProjectTag);

        // TODO: "only tag, original_tag and has_dlc allowed"
        vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);

        // FROM: country
        vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::SpecialProject);
        // FROM: country
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::SpecialProject);

        vd.field_validated_block("breakthrough_cost", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_fields(Item::Specialization, |_, bv, data| match bv {
                BV::Value(value) => {
                    value.expect_integer();
                }
                BV::Block(block) => {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    validate_modifiers_with_base(block, data, &mut sc);
                }
            });
        });

        vd.field_item("blueprint_image", Item::Sprite);
        vd.req_field("prototype_time");
        vd.field_integer_range("prototype_time", 1..);
        vd.field_integer_range("complexity", 1..);

        vd.field_validated_block("resource_cost", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block("resources", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.validate_item_key_values(Item::Resource, |key, mut vd| {
                    if key.is("oil") {
                        let msg = "oil is not allowed here";
                        err(ErrorKey::Validation).strong().msg(msg).loc(key).push();
                    }
                    vd.integer();
                });
            });
        });

        let mut sc = ScopeContext::new(Scopes::SpecialProject, key);
        vd.field_validated_block_sc("empty_reward_weight", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);

        vd.field_list_items("special_project_parent", Item::SpecialProject);

        vd.field_validated_block("project_output", |block, data| {
            validate_project_output(block, data);
        });

        vd.field_validated_block("unique_prototype_rewards", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                validate_prototype_reward(key, block, data);
            });
        });
        vd.field_list_items("generic_prototype_rewards", Item::PrototypeReward);
    }
}

fn validate_project_output(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    // TODO: FROM = SpecialProject
    vd.field_effect_rooted("country_effects", Tooltipped::Yes, Scopes::Country);

    // TODO: FROM = SpecialProject
    vd.field_effect_rooted("facility_state_effects", Tooltipped::Yes, Scopes::State);

    // TODO: FROM = SpecialProject
    vd.field_effect_rooted("scientist_effects", Tooltipped::Yes, Scopes::Character);

    vd.field_validated_block("enable_equipments", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: only has_dlc is allowed
        vd.field_trigger_rooted("limit", Tooltipped::No, Scopes::None);
        for token in vd.values() {
            data.verify_exists(Item::Equipment, token);
        }
    });

    vd.field_validated_block("enable_equipment_modules", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: scope not mentioned in docs
        vd.field_trigger_rooted("limit", Tooltipped::No, Scopes::None);
        for token in vd.values() {
            data.verify_exists(Item::EquipmentModule, token);
        }
    });

    vd.field_validated_block("enable_subunits", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: scope not mentioned in docs
        vd.field_trigger_rooted("limit", Tooltipped::No, Scopes::None);
        for token in vd.values() {
            data.verify_exists(Item::SubUnit, token);
        }
    });

    vd.field_validated_block("sub_unit_bonus", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.validate_item_key_blocks(Item::SubUnit, |_, block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
                vd.numeric();
            });
        });
    });

    vd.field_validated_block("equipment_bonus", validate_equipment_bonus);
}

#[derive(Clone, Debug)]
pub struct SpecialProjectTag {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::SpecialProjectTag, SpecialProjectTag::add)
}

impl SpecialProjectTag {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("project_tags") {
            for value in block.iter_values_warn() {
                db.add_flag(Item::SpecialProjectTag, value.clone());
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `project_tags` here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
        db.set_flag_validator(Item::SpecialProjectTag, |flag, data| {
            data.verify_exists(Item::Localization, flag);
        });
    }
}

fn validate_narrative(key: &Token, data: &Everything, vd: &mut Validator, needs_desc: bool) {
    let mut has_name = false;
    let mut has_desc = false;
    vd.field_validated_block("narrative", |block, data| {
        let mut vd = Validator::new(block, data);
        has_name = vd.field_item("name", Item::Localization);
        if needs_desc {
            has_desc = vd.field_item("desc", Item::Localization);
        }
    });
    if !has_name {
        data.verify_exists(Item::Localization, key);
    }
    if needs_desc && !has_desc {
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
    }
}

#[derive(Clone, Debug)]
pub struct PrototypeReward {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::PrototypeReward, PrototypeReward::add)
}

impl PrototypeReward {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PrototypeReward, key, block, Box::new(Self {}));
    }
}

impl DbKind for PrototypeReward {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        validate_prototype_reward(key, block, data);
    }
}

fn validate_prototype_reward(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    validate_narrative(key, data, &mut vd, true);

    if !vd.field_item("icon", Item::Sprite) {
        let sprite = format!("GFX_{key}");
        data.mark_used(Item::Sprite, &sprite);
        // There's a fallback icon GFX_PLACEHOLDER_sp_project_picture
    }

    vd.field_bool("fire_only_once");
    vd.field_bool("force_reward_if_available");

    vd.field_validated_block("threshold", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_integer("min");
        vd.field_integer("max");
    });

    let mut sc = ScopeContext::new(Scopes::SpecialProject, key);
    vd.field_validated_block_sc("weight", &mut sc, validate_modifiers_with_base);
    vd.field_trigger_rooted("allowed", Tooltipped::No, Scopes::Country);

    vd.req_field("option");
    let mut seen_default = false;
    vd.multi_field_validated_block("option", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("token");
        vd.field_identifier("token", "token");

        vd.field_bool("default");
        if block.get_field_bool("default").unwrap_or(false) {
            if seen_default {
                let msg = "only one option should have default = yes";
                let key = block.get_key("default").unwrap();
                warn(ErrorKey::DuplicateField).msg(msg).loc(key).push();
            }
            seen_default = true;
        }

        if let Some(token) = block.get_field_value("token") {
            validate_narrative(token, data, &mut vd, false);
        } else {
            // token is mandatory, but here's a fallback
            vd.field_block("narrative");
        }

        vd.field_validated_block("iteration_output", |block, data| {
            validate_project_output(block, data);
        });
    });
}

#[derive(Clone, Debug)]
pub struct Specialization {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::Specialization, Specialization::add)
}

impl Specialization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Specialization, key, block, Box::new(Self {}));
    }
}

impl DbKind for Specialization {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        if !vd.field_item("icon", Item::Sprite) {
            let sprite = format!("GFX_{key}");
            data.mark_used(Item::Sprite, &sprite);
            // There's a fallback icon GFX_PLACEHOLDER_sp_specialization_icon
        }

        if !vd.field_item("blueprint_image", Item::Sprite) {
            let sprite = format!("GFX_{key}_blueprint");
            data.mark_used(Item::Sprite, &sprite);
            // There's a fallback icon GFX_PLACEHOLDER_sp_blueprint
        }

        if !vd.field_item("program_background", Item::Sprite) {
            let sprite = format!("GFX_{key}_program_item");
            data.mark_used(Item::Sprite, &sprite);
            // There's a fallback icon GFX_tiled_plain_bg
        }

        vd.field_validated_block("color", validate_color);
    }
}
