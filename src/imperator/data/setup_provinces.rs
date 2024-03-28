use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SetupProvinces {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::SetupProvinces, SetupProvinces::add)
}

impl SetupProvinces {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SetupProvinces, key, block, Box::new(Self {}));
    }
}

impl DbKind for SetupProvinces {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("barbarian_power");
        vd.field_value("civilization_value");

        vd.field_validated_value("culture", |_key, mut vd| {
            vd.maybe_is("");
            vd.item(Item::Culture);
        });
        vd.field_validated_value("religion", |_key, mut vd| {
            vd.maybe_is("");
            vd.item(Item::Religion);
        });
        vd.field_validated_value("province_rank", |_key, mut vd| {
            vd.maybe_is("");
            vd.item(Item::ProvinceRank);
        });
        vd.field_validated_value("trade_goods", |_key, mut vd| {
            vd.maybe_is("");
            vd.item(Item::TradeGood);
        });

        vd.field_item("terrain", Item::Terrain);
        vd.field_item("holy_site", Item::Deity);

        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Building, key);
            value.expect_number();
        });

        vd.unknown_block_fields(|key, block| {
            data.verify_exists(Item::PopType, key);
            let mut vd = Validator::new(block, data);
            vd.field_item("culture", Item::Culture);
            vd.field_item("religion", Item::Religion);
            vd.field_integer("amount");
        });
    }
}
