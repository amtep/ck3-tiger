use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect_internal;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::lowercase::Lowercase;
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_color, ListType};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct State {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::State, State::add)
}

impl State {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if let Some(id) = block.get_field_value("id") {
            db.add(Item::State, id.clone(), block, Box::new(Self {}));
        } else {
            let msg = "state without id";
            err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for State {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_integer("id");
        vd.field_item("name", Item::Localization);

        vd.field_item("state_category", Item::StateCategory);
        vd.field_integer("manpower");
        vd.multi_field_validated_block("resources", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Resource, key);
                value.expect_number();
            });
        });

        vd.field_validated_block("history", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.validate_history_blocks(|_, _, block, data| {
                let vd = Validator::new(block, data);
                validate_state_history(key, block, data, vd);
            });
            validate_state_history(key, block, data, vd);
        });

        vd.field_list_items("provinces", Item::Province);
        vd.field_numeric("buildings_max_level_factor");
        vd.field_numeric("local_supplies");
        vd.field_bool("impassable");
    }
}

fn validate_state_history(key: &Token, block: &Block, data: &Everything, mut vd: Validator) {
    let mut sc = ScopeContext::new(Scopes::State, key);
    vd.field_item("owner", Item::CountryTag);
    vd.field_item("controller", Item::CountryTag);
    vd.multi_field_validated_block("buildings", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Building, key);
            value.expect_integer();
        });
        vd.unknown_block_fields(|key, block| {
            data.verify_exists(Item::Province, key);
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::Building, key);
                value.expect_integer();
            });
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::Building, key);
                let mut vd = Validator::new(block, data);
                vd.field_integer("level");
                vd.field_trigger_full("allowed", &mut sc, Tooltipped::No);
            });
        });
    });
    vd.multi_field_validated_block("victory_points", |block, data| {
        for (i, value) in block.iter_values().enumerate() {
            match i {
                0 => data.verify_exists(Item::Province, value),
                1 => {
                    value.expect_number();
                }
                2 => {
                    let msg = "too many values in victory points";
                    let info = "should be `{ province_id points }`";
                    err(ErrorKey::Validation).msg(msg).info(info).loc(value).push();
                }
                _ => (),
            }
        }
    });
    validate_effect_internal(
        Lowercase::empty(),
        ListType::None,
        block,
        data,
        &mut sc,
        &mut vd,
        Tooltipped::No,
    );
}

#[derive(Clone, Debug)]
pub struct StateCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::StateCategory, StateCategory::add)
}

impl StateCategory {
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("state_categories") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::StateCategory, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `state_categories`";
            err(ErrorKey::FieldMissing).msg(msg).info(info).loc(key).push();
        }
    }
}

impl DbKind for StateCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        vd.field_integer("local_building_slots");
        vd.field_validated_block("color", validate_color);
    }
}
