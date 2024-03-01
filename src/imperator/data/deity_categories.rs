use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DeityCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::DeityCategory, DeityCategory::add)
}

impl DeityCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DeityCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for DeityCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = key.as_str().to_uppercase();
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_value("icon");
    }
}
