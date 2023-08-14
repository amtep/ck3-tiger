use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct StateRegion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::StateRegion, StateRegion::add)
}

impl StateRegion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::StateRegion, key, block, Box::new(Self {}));
    }
}

impl DbKind for StateRegion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.req_field("id");
        vd.field_integer("id"); // TODO: verify unique?
                                // TODO: check that it's actually a subsistence building
        vd.field_item("subsistence_building", Item::BuildingType);
        vd.req_field("provinces");
        vd.field_list_items("provinces", Item::Province);
        vd.field_list_items("prime_land", Item::Province);
        vd.field_list_items("impassable", Item::Province);
        vd.field_list_items("traits", Item::StateTrait);

        for hub in &["city", "port", "mine", "farm", "wood"] {
            // TODO verify that these provinces are in the state region
            if vd.field_item(hub, Item::Province) {
                let loca = format!("HUB_NAME_{key}_{hub}");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }

        vd.field_integer("arable_land");
        vd.field_list_items("arable_resources", Item::BuildingGroup);
        vd.field_validated_block("capped_resources", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::BuildingGroup, key);
                value.expect_integer();
            });
        });
        vd.field_validated_blocks("resource", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("type");
            vd.field_item("type", Item::BuildingGroup);
            vd.field_item("depleted_type", Item::BuildingGroup);
            vd.field_integer("discovered_amount");
            vd.field_integer("undiscovered_amount");
        });
        vd.field_value("naval_exit_id"); // TODO it's an id of the sea provinces
    }
}
