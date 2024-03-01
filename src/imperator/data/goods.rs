use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TradeGood {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::TradeGood, TradeGood::add)
}

impl TradeGood {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TradeGood, key, block, Box::new(Self {}));
    }
}

impl DbKind for TradeGood {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}DESC");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("category");
        vd.req_field("gold");
        vd.req_field("province");
        vd.req_field("country");
        vd.req_field("color");

        vd.field_numeric("category");
        vd.field_numeric("gold");

        vd.multi_field_item("allow_unit_type", Item::Unit);

        vd.field_validated_block("province", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Province, vd);
        });

        vd.field_validated_block("country", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("color", validate_color);
    }
}
