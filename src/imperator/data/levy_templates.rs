use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct LevyTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::LevyTemplate, LevyTemplate::add)
}

impl LevyTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LevyTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for LevyTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_bool("default");
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Unit, key);
            value.expect_number();
        });
    }
}
