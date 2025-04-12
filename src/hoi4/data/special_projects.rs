use crate::block::{Block, BV};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
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

        vd.field_validated_block("narrative", |block, data| {
            let mut vd = Validator::new(block, data);
            if !vd.field_item("name", Item::Localization) {
                data.verify_exists(Item::Localization, key);
            }
            if !vd.field_item("desc", Item::Localization) {
                let loca = format!("{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        });

        if !vd.field_item("icon", Item::Sprite) {
            let sprite = format!("GFX_{key}");
            data.verify_exists_implied(Item::Sprite, &sprite, key);
        }

        vd.req_field("specialization");
        vd.field_item("specialization", Item::Specialization);

        vd.field_list_items("project_tags", Item::ProjectTag);

        // TODO: "only tag, original_tag and has_dlc allowed"
        vd.field_trigger_full("allowed", Scopes::Country, Tooltipped::No);

        // FROM: country
        vd.field_trigger_full("available", Scopes::SpecialProject, Tooltipped::Yes);
        // FROM: country
        vd.field_trigger_full("visible", Scopes::SpecialProject, Tooltipped::No);

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

        vd.field_validated_block("unique_prototype_rewards", |_block, _data| {
            // TODO
        });
        vd.field_list_items("generic_prototype_rewards", Item::PrototypeReward);
    }
}

fn validate_project_output(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    // TODO: FROM = SpecialProject
    vd.field_effect_full("country_effects", Scopes::Country, Tooltipped::Yes);

    // TODO: FROM = SpecialProject
    vd.field_effect_full("facility_state_effects", Scopes::State, Tooltipped::Yes);

    // TODO: FROM = SpecialProject
    vd.field_effect_full("scientist_effects", Scopes::Character, Tooltipped::Yes);

    vd.field_validated_block("enable_equipments", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: only has_dlc is allowed
        vd.field_trigger_full("limit", Scopes::None, Tooltipped::No);
        for token in vd.values() {
            data.verify_exists(Item::Equipment, token);
        }
    });

    vd.field_validated_block("enable_equipment_modules", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: scope not mentioned in docs
        vd.field_trigger_full("limit", Scopes::None, Tooltipped::No);
        for token in vd.values() {
            data.verify_exists(Item::EquipmentModule, token);
        }
    });

    vd.field_validated_block("enable_subunits", |block, data| {
        let mut vd = Validator::new(block, data);
        // TODO: scope not mentioned in docs
        vd.field_trigger_full("limit", Scopes::None, Tooltipped::No);
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

    vd.field_validated_block("equipment_bonus", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.validate_item_key_blocks(Item::EquipmentBonusType, |_, block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("instant");
            vd.validate_item_key_values(Item::EquipmentStat, |_, mut vd| {
                vd.numeric();
            });
        });
    });
}
