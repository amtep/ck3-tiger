use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SubUnit {}
#[derive(Clone, Debug)]
pub struct SubUnitCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::SubUnit, SubUnit::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::SubUnitCategory, SubUnitCategory::add)
}

impl SubUnit {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("sub_units") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::SubUnit, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "only `sub_units` is expected here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
    }
}

impl SubUnitCategory {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("sub_unit_categories") {
            for value in block.iter_values_warn() {
                db.add_flag(Item::SubUnitCategory, value.clone());
            }
        } else {
            let msg = "unexpected key";
            let info = "only `sub_units_categories` is expected here";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
        }
        db.set_flag_validator(Item::SubUnitCategory, |flag, data| {
            data.verify_exists(Item::Localization, flag);
        });
    }
}

impl DbKind for SubUnit {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_value("abbreviation");
        // TODO: figure out what values this can take. It is not a Sprite name.
        vd.field_value("sprite");
        vd.field_integer("priority");
        vd.field_integer("ai_priority");
        vd.field_bool("active");
        vd.field_bool("special_forces");
        vd.field_bool("marines");
        vd.field_bool("can_be_parachuted");
        vd.field_bool("is_artillery_brigade");
        vd.field_bool("cavalry");
        vd.field_choice(
            "map_icon_category",
            &["infantry", "armored", "other", "ship", "transport", "uboat"],
        );
        vd.field_item("type", Item::EquipmentCategory);
        vd.field_validated_block("need", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_item_key_values(Item::EquipmentBonusType, |_, mut vd| {
                vd.integer();
            });
        });
        vd.field_choice(
            "group",
            &[
                "armor",
                "mobile",
                "support",
                "combat_support",
                "mobile_combat_support",
                "armor_combat_support",
            ],
        );
        vd.field_list_items("categories", Item::SubUnitCategory);
        vd.field_validated_block("battalion_mult", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("category", Item::SubUnitCategory);
            vd.field_bool("add");
            validate_combat_stats(&mut vd);
        });

        validate_combat_stats(&mut vd);

        vd.field_list_items("essential", Item::EquipmentBonusType);
        vd.field_item("transport", Item::EquipmentBonusType);

        vd.field_item("same_support_type", Item::SubUnit);

        vd.field_validated_block("fort", validate_terrain_values);
        vd.field_validated_block("river", validate_terrain_values);
        vd.field_validated_block("amphibious", validate_terrain_values);
        vd.validate_item_key_blocks(Item::Terrain, |_, block, data| {
            validate_terrain_values(block, data);
        });
    }
}

fn validate_terrain_values(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_numeric("attack");
    vd.field_numeric("movement");
    vd.field_numeric("defence");
}

fn validate_combat_stats(vd: &mut Validator) {
    vd.field_numeric("defense");
    vd.field_numeric("breakthrough");
    vd.field_numeric("soft_attack");
    vd.field_numeric("hard_attack");
    vd.field_numeric("air_attack");
    vd.field_integer("manpower");
    vd.advice_field("max_organization", "it's max_organisation with an s");
    vd.field_integer("max_organisation");
    vd.field_numeric("default_morale");
    vd.field_integer("training_time");
    vd.field_integer("max_strength");
    vd.field_integer("suppression");
    vd.field_integer("weight");
    vd.field_numeric("supply_consumption");
    vd.field_numeric("maximum_speed");

    vd.field_integer("mega_carrier_air_wing_size");
    vd.field_integer("carrier_air_wing_size");
    vd.field_integer("land_air_wing_size");

    vd.field_integer("combat_width");
    vd.field_numeric("entrenchment");
}
