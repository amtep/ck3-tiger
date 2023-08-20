use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PopNeed {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::PopNeed, PopNeed::add)
}

impl PopNeed {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PopNeed, key, block, Box::new(Self {}));
    }
}

impl DbKind for PopNeed {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        // TODO: verify that it's one of the goods in this PopNeed
        vd.field_item("default", Item::Goods);
        vd.multi_field_validated_block("entry", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("goods", Item::Goods);
            vd.field_numeric("weight");
            vd.field_numeric("max_weight");
            vd.field_numeric("min_weight");
        });
    }
}
